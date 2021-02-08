//! Create and manage containers.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Container>

use std::{io, path::Path, time::Duration};

use futures_util::{
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
    TryStreamExt,
};
use hyper::Body;
use mime::Mime;
use url::form_urlencoded;

use crate::{
    builder::{
        ContainerListOptions, ContainerOptions, ExecContainerOptions, LogsOptions,
        RmContainerOptions,
    },
    docker::Docker,
    errors::{Error, Result},
    exec::Exec,
    rep::{
        Change, Container as ContainerInfo, ContainerCreateInfo, ContainerDetails, Exit, Stats, Top,
    },
    transport::Payload,
    tty::{self, Multiplexer as TtyMultiPlexer},
};

/// Interface for accessing and manipulating a docker container
pub struct Container<'docker> {
    docker: &'docker Docker,
    id: String,
}

impl<'docker> Container<'docker> {
    /// Exports an interface exposing operations against a container instance
    pub fn new<S>(
        docker: &'docker Docker,
        id: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Container {
            docker,
            id: id.into(),
        }
    }

    /// a getter for the container id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Inspects the current docker container instance's details
    pub async fn inspect(&self) -> Result<ContainerDetails> {
        self.docker
            .get_json::<ContainerDetails>(&format!("/containers/{}/json", self.id)[..])
            .await
    }

    /// Returns a `top` view of information about the container process
    pub async fn top(
        &self,
        psargs: Option<&str>,
    ) -> Result<Top> {
        let mut path = vec![format!("/containers/{}/top", self.id)];
        if let Some(ref args) = psargs {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("ps_args", args)
                .finish();
            path.push(encoded)
        }
        self.docker.get_json(&path.join("?")).await
    }

    /// Returns a stream of logs emitted but the container instance
    pub fn logs(
        &self,
        opts: &LogsOptions,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + Unpin + 'docker {
        let mut path = vec![format!("/containers/{}/logs", self.id)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        let stream = Box::pin(self.docker.stream_get(path.join("?")));

        Box::pin(tty::decode(stream))
    }

    /// Attaches a multiplexed TCP stream to the container that can be used to read Stdout, Stderr and write Stdin.
    async fn attach_raw(&self) -> Result<impl AsyncRead + AsyncWrite + Send + 'docker> {
        self.docker
            .stream_post_upgrade(
                format!(
                    "/containers/{}/attach?stream=1&stdout=1&stderr=1&stdin=1",
                    self.id
                ),
                None,
            )
            .await
    }

    /// Attaches a `[TtyMultiplexer]` to the container.
    ///
    /// The `[TtyMultiplexer]` implements Stream for returning Stdout and Stderr chunks. It also implements `[AsyncWrite]` for writing to Stdin.
    ///
    /// The multiplexer can be split into its read and write halves with the `[split](TtyMultiplexer::split)` method
    pub async fn attach(&self) -> Result<TtyMultiPlexer<'docker>> {
        let tcp_stream = self.attach_raw().await?;

        Ok(TtyMultiPlexer::new(tcp_stream))
    }

    /// Returns a set of changes made to the container instance
    pub async fn changes(&self) -> Result<Vec<Change>> {
        self.docker
            .get_json::<Vec<Change>>(&format!("/containers/{}/changes", self.id)[..])
            .await
    }

    /// Exports the current docker container into a tarball
    pub fn export(&self) -> impl Stream<Item = Result<Vec<u8>>> + 'docker {
        self.docker
            .stream_get(format!("/containers/{}/export", self.id))
            .map_ok(|c| c.to_vec())
    }

    /// Returns a stream of stats specific to this container instance
    pub fn stats(&self) -> impl Stream<Item = Result<Stats>> + Unpin + 'docker {
        let codec = futures_codec::LinesCodec {};

        let reader = Box::pin(
            self.docker
                .stream_get(format!("/containers/{}/stats", self.id))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
        )
        .into_async_read();

        Box::pin(
            futures_codec::FramedRead::new(reader, codec)
                .map_err(Error::IO)
                .and_then(|s: String| async move {
                    serde_json::from_str(&s).map_err(Error::SerdeJsonError)
                }),
        )
    }

    /// Start the container instance
    pub async fn start(&self) -> Result<()> {
        self.docker
            .post(&format!("/containers/{}/start", self.id)[..], None)
            .await?;
        Ok(())
    }

    /// Stop the container instance
    pub async fn stop(
        &self,
        wait: Option<Duration>,
    ) -> Result<()> {
        let mut path = vec![format!("/containers/{}/stop", self.id)];
        if let Some(w) = wait {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("t", &w.as_secs().to_string())
                .finish();

            path.push(encoded)
        }
        self.docker.post(&path.join("?"), None).await?;
        Ok(())
    }

    /// Restart the container instance
    pub async fn restart(
        &self,
        wait: Option<Duration>,
    ) -> Result<()> {
        let mut path = vec![format!("/containers/{}/restart", self.id)];
        if let Some(w) = wait {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("t", &w.as_secs().to_string())
                .finish();
            path.push(encoded)
        }
        self.docker.post(&path.join("?"), None).await?;
        Ok(())
    }

    /// Kill the container instance
    pub async fn kill(
        &self,
        signal: Option<&str>,
    ) -> Result<()> {
        let mut path = vec![format!("/containers/{}/kill", self.id)];
        if let Some(sig) = signal {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("signal", &sig.to_owned())
                .finish();
            path.push(encoded)
        }
        self.docker.post(&path.join("?"), None).await?;
        Ok(())
    }

    /// Rename the container instance
    pub async fn rename(
        &self,
        name: &str,
    ) -> Result<()> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("name", name)
            .finish();
        self.docker
            .post(
                &format!("/containers/{}/rename?{}", self.id, query)[..],
                None,
            )
            .await?;
        Ok(())
    }

    /// Pause the container instance
    pub async fn pause(&self) -> Result<()> {
        self.docker
            .post(&format!("/containers/{}/pause", self.id)[..], None)
            .await?;
        Ok(())
    }

    /// Unpause the container instance
    pub async fn unpause(&self) -> Result<()> {
        self.docker
            .post(&format!("/containers/{}/unpause", self.id)[..], None)
            .await?;
        Ok(())
    }

    /// Wait until the container stops
    pub async fn wait(&self) -> Result<Exit> {
        self.docker
            .post_json(format!("/containers/{}/wait", self.id), Payload::None)
            .await
    }

    /// Delete the container instance
    ///
    /// Use remove instead to use the force/v options.
    pub async fn delete(&self) -> Result<()> {
        self.docker
            .delete(&format!("/containers/{}", self.id)[..])
            .await?;
        Ok(())
    }

    /// Delete the container instance (todo: force/v)
    pub async fn remove(
        &self,
        opts: RmContainerOptions,
    ) -> Result<()> {
        let mut path = vec![format!("/containers/{}", self.id)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker.delete(&path.join("?")).await?;
        Ok(())
    }

    /// Execute a command in this container
    pub fn exec(
        &self,
        opts: &ExecContainerOptions,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + Unpin + 'docker {
        Exec::create_and_start(self.docker, &self.id, opts)
    }

    /// Copy a file/folder from the container.  The resulting stream is a tarball of the extracted
    /// files.
    ///
    /// If `path` is not an absolute path, it is relative to the container’s root directory. The
    /// resource specified by `path` must exist. To assert that the resource is expected to be a
    /// directory, `path` should end in `/` or `/`. (assuming a path separator of `/`). If `path`
    /// ends in `/.`  then this indicates that only the contents of the path directory should be
    /// copied.  A symlink is always resolved to its target.
    pub fn copy_from(
        &self,
        path: &Path,
    ) -> impl Stream<Item = Result<Vec<u8>>> + 'docker {
        let path_arg = form_urlencoded::Serializer::new(String::new())
            .append_pair("path", &path.to_string_lossy())
            .finish();

        let endpoint = format!("/containers/{}/archive?{}", self.id, path_arg);
        self.docker.stream_get(endpoint).map_ok(|c| c.to_vec())
    }

    /// Copy a byte slice as file into (see `bytes`) the container.
    ///
    /// The file will be copied at the given location (see `path`) and will be owned by root
    /// with access mask 644.
    pub async fn copy_file_into<P: AsRef<Path>>(
        &self,
        path: P,
        bytes: &[u8],
    ) -> Result<()> {
        let path = path.as_ref();

        let mut ar = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o0644);
        ar.append_data(
            &mut header,
            path.to_path_buf()
                .iter()
                .skip(1)
                .collect::<std::path::PathBuf>(),
            bytes,
        )
        .unwrap();
        let data = ar.into_inner().unwrap();

        self.copy_to(Path::new("/"), data.into()).await?;
        Ok(())
    }

    /// Copy a tarball (see `body`) to the container.
    ///
    /// The tarball will be copied to the container and extracted at the given location (see `path`).
    pub async fn copy_to(
        &self,
        path: &Path,
        body: Body,
    ) -> Result<()> {
        let path_arg = form_urlencoded::Serializer::new(String::new())
            .append_pair("path", &path.to_string_lossy())
            .finish();

        let mime = "application/x-tar".parse::<Mime>().unwrap();

        self.docker
            .put(
                &format!("/containers/{}/archive?{}", self.id, path_arg),
                Some((body, mime)),
            )
            .await?;
        Ok(())
    }
}

/// Interface for docker containers
pub struct Containers<'docker> {
    docker: &'docker Docker,
}

impl<'docker> Containers<'docker> {
    /// Exports an interface for interacting with docker containers
    pub fn new(docker: &'docker Docker) -> Self {
        Containers { docker }
    }

    /// Lists the container instances on the docker host
    pub async fn list(
        &self,
        opts: &ContainerListOptions,
    ) -> Result<Vec<ContainerInfo>> {
        let mut path = vec!["/containers/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker
            .get_json::<Vec<ContainerInfo>>(&path.join("?"))
            .await
    }

    /// Returns a reference to a set of operations available to a specific container instance
    pub fn get<S>(
        &self,
        name: S,
    ) -> Container<'docker>
    where
        S: Into<String>,
    {
        Container::new(self.docker, name)
    }

    /// Returns a builder interface for creating a new container instance
    pub async fn create(
        &self,
        opts: &ContainerOptions,
    ) -> Result<ContainerCreateInfo> {
        let body: Body = opts.serialize()?.into();
        let mut path = vec!["/containers/create".to_owned()];

        if let Some(ref name) = opts.name {
            path.push(
                form_urlencoded::Serializer::new(String::new())
                    .append_pair("name", name)
                    .finish(),
            );
        }

        self.docker
            .post_json(&path.join("?"), Some((body, mime::APPLICATION_JSON)))
            .await
    }
}
