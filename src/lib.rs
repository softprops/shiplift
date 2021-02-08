//! Shiplift is a multi-transport utility for maneuvering [docker](https://www.docker.com/) containers
//!
//! # examples
//!
//! ```no_run
//! # async {
//! let docker = shiplift::Docker::new();
//!
//! match docker.images().list(&Default::default()).await {
//!     Ok(images) => {
//!         for image in images {
//!             println!("{:?}", image.repo_tags);
//!         }
//!     },
//!     Err(e) => eprintln!("Something bad happened! {}", e),
//! }
//! # };
//! ```

pub mod builder;
pub mod errors;
pub mod rep;
pub mod transport;
pub mod tty;

mod tarball;

pub use crate::{
    builder::{
        BuildOptions, ContainerConnectionOptions, ContainerFilter, ContainerListOptions,
        ContainerOptions, EventsOptions, ExecContainerOptions, ExecResizeOptions, ImageFilter,
        ImageListOptions, LogsOptions, NetworkCreateOptions, NetworkListOptions, PullOptions,
        RegistryAuth, RmContainerOptions, TagOptions, VolumeCreateOptions,
    },
    errors::Error,
};
use crate::{
    rep::{
        Change, Container as ContainerRep, ContainerCreateInfo, ContainerDetails, Event,
        ExecDetails, Exit, History, Image as ImageRep, ImageDetails, Info, NetworkCreateInfo,
        NetworkDetails as NetworkInfo, SearchResult, Service as ServiceRep,
        Services as ServicesRep, Stats, Status, Top, Version, Volume as VolumeRep,
        VolumeCreateInfo, Volumes as VolumesRep,
    },
    transport::{tar, Transport},
    tty::Multiplexer as TtyMultiPlexer,
};
use futures_util::{
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
    TryFutureExt, TryStreamExt,
};
// use futures::{future::Either, Future, IntoFuture, Stream};
pub use hyper::Uri;
use hyper::{client::HttpConnector, Body, Client, Method};
#[cfg(feature = "tls")]
use hyper_openssl::HttpsConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::UnixConnector;
use mime::Mime;
#[cfg(feature = "tls")]
use openssl::ssl::{SslConnector, SslFiletype, SslMethod};
use serde_json::Value;
use std::{env, io, io::Read, iter, path::Path, time::Duration};
use url::form_urlencoded;

/// Represents the result of all docker operations
pub type Result<T> = std::result::Result<T, Error>;

/// Entrypoint interface for communicating with docker daemon
#[derive(Clone)]
pub struct Docker {
    transport: Transport,
}

/// Interface for accessing and manipulating a named docker image
pub struct Image<'a> {
    docker: &'a Docker,
    name: String,
}

impl<'a> Image<'a> {
    /// Exports an interface for operations that may be performed against a named image
    pub fn new<S>(
        docker: &Docker,
        name: S,
    ) -> Image
    where
        S: Into<String>,
    {
        Image {
            docker,
            name: name.into(),
        }
    }

    /// Inspects a named image's details
    pub async fn inspect(&self) -> Result<ImageDetails> {
        self.docker
            .get_json(&format!("/images/{}/json", self.name)[..])
            .await
    }

    /// Lists the history of the images set of changes
    pub async fn history(&self) -> Result<Vec<History>> {
        self.docker
            .get_json(&format!("/images/{}/history", self.name)[..])
            .await
    }

    /// Deletes an image
    pub async fn delete(&self) -> Result<Vec<Status>> {
        self.docker
            .delete_json::<Vec<Status>>(&format!("/images/{}", self.name)[..])
            .await
    }

    /// Export this image to a tarball
    pub fn export(&self) -> impl Stream<Item = Result<Vec<u8>>> + Unpin + 'a {
        Box::pin(
            self.docker
                .stream_get(format!("/images/{}/get", self.name))
                .map_ok(|c| c.to_vec()),
        )
    }

    /// Adds a tag to an image
    pub async fn tag(
        &self,
        opts: &TagOptions,
    ) -> Result<()> {
        let mut path = vec![format!("/images/{}/tag", self.name)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        let _ = self.docker.post(&path.join("?"), None).await?;
        Ok(())
    }
}

/// Interface for docker images
pub struct Images<'a> {
    docker: &'a Docker,
}

impl<'a> Images<'a> {
    /// Exports an interface for interacting with docker images
    pub fn new(docker: &'a Docker) -> Images<'a> {
        Images { docker }
    }

    /// Builds a new image build by reading a Dockerfile in a target directory
    pub fn build(
        &'a self,
        opts: &'a BuildOptions,
    ) -> impl Stream<Item = Result<Value>> + Unpin + 'a {
        Box::pin(
            async move {
                let mut path = vec!["/build".to_owned()];
                if let Some(query) = opts.serialize() {
                    path.push(query)
                }

                let mut bytes = Vec::default();

                tarball::dir(&mut bytes, &opts.path[..])?;

                let value_stream = self.docker.stream_post_into_values(
                    path.join("?"),
                    Some((Body::from(bytes), tar())),
                    None::<iter::Empty<_>>,
                );

                Ok(value_stream)
            }
            .try_flatten_stream(),
        )
    }

    /// Lists the docker images on the current docker host
    pub async fn list(
        &self,
        opts: &ImageListOptions,
    ) -> Result<Vec<ImageRep>> {
        let mut path = vec!["/images/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        self.docker.get_json::<Vec<ImageRep>>(&path.join("?")).await
    }

    /// Returns a reference to a set of operations available for a named image
    pub fn get<S>(
        &self,
        name: S,
    ) -> Image<'a>
    where
        S: Into<String>,
    {
        Image::new(self.docker, name)
    }

    /// Search for docker images by term
    pub async fn search(
        &self,
        term: &str,
    ) -> Result<Vec<SearchResult>> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("term", term)
            .finish();
        self.docker
            .get_json::<Vec<SearchResult>>(&format!("/images/search?{}", query)[..])
            .await
    }

    /// Pull and create a new docker images from an existing image
    pub fn pull(
        &self,
        opts: &PullOptions,
    ) -> impl Stream<Item = Result<Value>> + Unpin + 'a {
        let mut path = vec!["/images/create".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let headers = opts
            .auth_header()
            .map(|a| iter::once(("X-Registry-Auth", a)));

        Box::pin(
            self.docker
                .stream_post_into_values(path.join("?"), None, headers),
        )
    }

    /// exports a collection of named images,
    /// either by name, name:tag, or image id, into a tarball
    pub fn export(
        &self,
        names: Vec<&str>,
    ) -> impl Stream<Item = Result<Vec<u8>>> + 'a {
        let params = names.iter().map(|n| ("names", *n));
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();
        self.docker
            .stream_get(format!("/images/get?{}", query))
            .map_ok(|c| c.to_vec())
    }

    /// imports an image or set of images from a given tarball source
    /// source can be uncompressed on compressed via gzip, bzip2 or xz
    pub fn import<R>(
        self,
        mut tarball: R,
    ) -> impl Stream<Item = Result<Value>> + Unpin + 'a
    where
        R: Read + Send + 'a,
    {
        Box::pin(
            async move {
                let mut bytes = Vec::default();

                tarball.read_to_end(&mut bytes)?;

                let value_stream = self.docker.stream_post_into_values(
                    "/images/load",
                    Some((Body::from(bytes), tar())),
                    None::<iter::Empty<_>>,
                );
                Ok(value_stream)
            }
            .try_flatten_stream(),
        )
    }
}

/// Interface for accessing and manipulating a docker container
pub struct Container<'a> {
    docker: &'a Docker,
    id: String,
}

impl<'a> Container<'a> {
    /// Exports an interface exposing operations against a container instance
    pub fn new<S>(
        docker: &'a Docker,
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
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + Unpin + 'a {
        let mut path = vec![format!("/containers/{}/logs", self.id)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        let stream = Box::pin(self.docker.stream_get(path.join("?")));

        Box::pin(tty::decode(stream))
    }

    /// Attaches a multiplexed TCP stream to the container that can be used to read Stdout, Stderr and write Stdin.
    async fn attach_raw(&self) -> Result<impl AsyncRead + AsyncWrite + Send + 'a> {
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
    pub async fn attach(&self) -> Result<TtyMultiPlexer<'a>> {
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
    pub fn export(&self) -> impl Stream<Item = Result<Vec<u8>>> + 'a {
        self.docker
            .stream_get(format!("/containers/{}/export", self.id))
            .map_ok(|c| c.to_vec())
    }

    /// Returns a stream of stats specific to this container instance
    pub fn stats(&'a self) -> impl Stream<Item = Result<Stats>> + Unpin + 'a {
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
            .post_json(
                format!("/containers/{}/wait", self.id),
                Option::<(Body, Mime)>::None,
            )
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
        &'a self,
        opts: &'a ExecContainerOptions,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + Unpin + 'a {
        Box::pin(
            async move {
                let id = Exec::create_id(&self.docker, &self.id, opts).await?;
                Ok(Exec::_start(&self.docker, &id))
            }
            .try_flatten_stream(),
        )
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
    ) -> impl Stream<Item = Result<Vec<u8>>> + 'a {
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
pub struct Containers<'a> {
    docker: &'a Docker,
}

impl<'a> Containers<'a> {
    /// Exports an interface for interacting with docker containers
    pub fn new(docker: &'a Docker) -> Containers<'a> {
        Containers { docker }
    }

    /// Lists the container instances on the docker host
    pub async fn list(
        &self,
        opts: &ContainerListOptions,
    ) -> Result<Vec<ContainerRep>> {
        let mut path = vec!["/containers/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker
            .get_json::<Vec<ContainerRep>>(&path.join("?"))
            .await
    }

    /// Returns a reference to a set of operations available to a specific container instance
    pub fn get<S>(
        &self,
        name: S,
    ) -> Container
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
/// Interface for docker exec instance
pub struct Exec<'a> {
    docker: &'a Docker,
    id: String,
}

impl<'a> Exec<'a> {
    fn new<S>(
        docker: &'a Docker,
        id: S,
    ) -> Exec<'a>
    where
        S: Into<String>,
    {
        Exec {
            docker,
            id: id.into(),
        }
    }

    /// Creates an exec instance in docker and returns its id
    pub(crate) async fn create_id(
        docker: &'a Docker,
        container_id: &str,
        opts: &ExecContainerOptions,
    ) -> Result<String> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Response {
            id: String,
        }

        let body: Body = opts.serialize()?.into();

        docker
            .post_json(
                &format!("/containers/{}/exec", container_id)[..],
                Some((body, mime::APPLICATION_JSON)),
            )
            .await
            .map(|resp: Response| resp.id)
    }

    /// Starts an exec instance with id exec_id
    pub(crate) fn _start(
        docker: &'a Docker,
        exec_id: &str,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + 'a {
        let bytes: &[u8] = b"{}";

        let stream = Box::pin(docker.stream_post(
            format!("/exec/{}/start", &exec_id),
            Some((bytes.into(), mime::APPLICATION_JSON)),
            None::<iter::Empty<_>>,
        ));

        tty::decode(stream)
    }

    /// Creates a new exec instance that will be executed in a container with id == container_id
    pub async fn create(
        docker: &'a Docker,
        container_id: &str,
        opts: &ExecContainerOptions,
    ) -> Result<Exec<'a>> {
        Ok(Exec::new(
            docker,
            Exec::create_id(docker, container_id, opts).await?,
        ))
    }

    /// Get a reference to a set of operations available to an already created exec instance.
    ///
    /// It's in callers responsibility to ensure that exec instance with specified id actually
    /// exists. Use [Exec::create](Exec::create) to ensure that the exec instance is created
    /// beforehand.
    pub async fn get<S>(
        docker: &'a Docker,
        id: S,
    ) -> Exec<'a>
    where
        S: Into<String>,
    {
        Exec::new(docker, id)
    }

    /// Starts this exec instance returning a multiplexed tty stream
    pub fn start(&'a self) -> impl Stream<Item = Result<tty::TtyChunk>> + 'a {
        Box::pin(
            async move {
                let bytes: &[u8] = b"{}";

                let stream = Box::pin(self.docker.stream_post(
                    format!("/exec/{}/start", &self.id),
                    Some((bytes.into(), mime::APPLICATION_JSON)),
                    None::<iter::Empty<_>>,
                ));

                Ok(tty::decode(stream))
            }
            .try_flatten_stream(),
        )
    }

    /// Inspect this exec instance to aquire detailed information
    pub async fn inspect(&self) -> Result<ExecDetails> {
        self.docker
            .get_json(&format!("/exec/{}/json", &self.id)[..])
            .await
    }

    pub async fn resize(
        &self,
        opts: &ExecResizeOptions,
    ) -> Result<()> {
        let body: Body = opts.serialize()?.into();

        self.docker
            .post_json(
                &format!("/exec/{}/resize", &self.id)[..],
                Some((body, mime::APPLICATION_JSON)),
            )
            .await
    }
}

/// Interface for docker network
pub struct Networks<'a> {
    docker: &'a Docker,
}

impl<'a> Networks<'a> {
    /// Exports an interface for interacting with docker Networks
    pub fn new(docker: &Docker) -> Networks {
        Networks { docker }
    }

    /// List the docker networks on the current docker host
    pub async fn list(
        &self,
        opts: &NetworkListOptions,
    ) -> Result<Vec<NetworkInfo>> {
        let mut path = vec!["/networks".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        self.docker.get_json(&path.join("?")).await
    }

    /// Returns a reference to a set of operations available to a specific network instance
    pub fn get<S>(
        &self,
        id: S,
    ) -> Network
    where
        S: Into<String>,
    {
        Network::new(self.docker, id)
    }

    /// Create a new Network instance
    pub async fn create(
        &self,
        opts: &NetworkCreateOptions,
    ) -> Result<NetworkCreateInfo> {
        let body: Body = opts.serialize()?.into();
        let path = vec!["/networks/create".to_owned()];

        self.docker
            .post_json(&path.join("?"), Some((body, mime::APPLICATION_JSON)))
            .await
    }
}

/// Interface for accessing and manipulating a docker network
pub struct Network<'a> {
    docker: &'a Docker,
    id: String,
}

impl<'a> Network<'a> {
    /// Exports an interface exposing operations against a network instance
    pub fn new<S>(
        docker: &Docker,
        id: S,
    ) -> Network
    where
        S: Into<String>,
    {
        Network {
            docker,
            id: id.into(),
        }
    }

    /// a getter for the Network id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Inspects the current docker network instance's details
    pub async fn inspect(&self) -> Result<NetworkInfo> {
        self.docker
            .get_json(&format!("/networks/{}", self.id)[..])
            .await
    }

    /// Delete the network instance
    pub async fn delete(&self) -> Result<()> {
        self.docker
            .delete(&format!("/networks/{}", self.id)[..])
            .await?;
        Ok(())
    }

    /// Connect container to network
    pub async fn connect(
        &self,
        opts: &ContainerConnectionOptions,
    ) -> Result<()> {
        self.do_connection("connect", opts).await
    }

    /// Disconnect container to network
    pub async fn disconnect(
        &self,
        opts: &ContainerConnectionOptions,
    ) -> Result<()> {
        self.do_connection("disconnect", opts).await
    }

    async fn do_connection(
        &self,
        segment: &str,
        opts: &ContainerConnectionOptions,
    ) -> Result<()> {
        let body: Body = opts.serialize()?.into();

        self.docker
            .post(
                &format!("/networks/{}/{}", self.id, segment)[..],
                Some((body, mime::APPLICATION_JSON)),
            )
            .await?;
        Ok(())
    }
}

/// Interface for docker volumes
pub struct Volumes<'a> {
    docker: &'a Docker,
}

impl<'a> Volumes<'a> {
    /// Exports an interface for interacting with docker volumes
    pub fn new(docker: &Docker) -> Volumes {
        Volumes { docker }
    }

    pub async fn create(
        &self,
        opts: &VolumeCreateOptions,
    ) -> Result<VolumeCreateInfo> {
        let body: Body = opts.serialize()?.into();
        let path = vec!["/volumes/create".to_owned()];

        self.docker
            .post_json(&path.join("?"), Some((body, mime::APPLICATION_JSON)))
            .await
    }

    /// Lists the docker volumes on the current docker host
    pub async fn list(&self) -> Result<Vec<VolumeRep>> {
        let path = vec!["/volumes".to_owned()];

        let volumes_rep = self.docker.get_json::<VolumesRep>(&path.join("?")).await?;
        Ok(match volumes_rep.volumes {
            Some(volumes) => volumes,
            None => vec![],
        })
    }

    /// Returns a reference to a set of operations available for a named volume
    pub fn get(
        &self,
        name: &str,
    ) -> Volume {
        Volume::new(self.docker, name)
    }
}

/// Interface for accessing and manipulating a named docker volume
pub struct Volume<'a> {
    docker: &'a Docker,
    name: String,
}

impl<'a> Volume<'a> {
    /// Exports an interface for operations that may be performed against a named volume
    pub fn new<S>(
        docker: &Docker,
        name: S,
    ) -> Volume
    where
        S: Into<String>,
    {
        Volume {
            docker,
            name: name.into(),
        }
    }

    /// Deletes a volume
    pub async fn delete(&self) -> Result<()> {
        self.docker
            .delete(&format!("/volumes/{}", self.name)[..])
            .await?;
        Ok(())
    }
}

/// Interface for docker services
pub struct Services<'a> {
    docker: &'a Docker,
}

impl<'a> Services<'a> {
    /// Exports an interface for interacting with docker services
    pub fn new(docker: &Docker) -> Services {
        Services { docker }
    }

    /// Lists the docker services on the current docker host
    pub async fn list(&self) -> Result<ServicesRep> {
        let path = vec!["/services".to_owned()];

        self.docker.get_json::<ServicesRep>(&path.join("?")).await
    }

    /// Returns a reference to a set of operations available for a named service
    pub fn get(
        &self,
        name: &str,
    ) -> Service {
        Service::new(self.docker, name)
    }
}

/// Interface for accessing and manipulating a named docker volume
pub struct Service<'a> {
    docker: &'a Docker,
    name: String,
}

impl<'a> Service<'a> {
    /// Exports an interface for operations that may be performed against a named service
    pub fn new<S>(
        docker: &Docker,
        name: S,
    ) -> Service
    where
        S: Into<String>,
    {
        Service {
            docker,
            name: name.into(),
        }
    }
}

fn get_http_connector() -> HttpConnector {
    let mut http = HttpConnector::new();
    http.enforce_http(false);

    http
}

#[cfg(feature = "tls")]
fn get_docker_for_tcp(tcp_host_str: String) -> Docker {
    let http = get_http_connector();
    if let Ok(ref certs) = env::var("DOCKER_CERT_PATH") {
        // fixme: don't unwrap before you know what's in the box
        // https://github.com/hyperium/hyper/blob/master/src/net.rs#L427-L428
        let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
        connector.set_cipher_list("DEFAULT").unwrap();
        let cert = &format!("{}/cert.pem", certs);
        let key = &format!("{}/key.pem", certs);
        connector
            .set_certificate_file(&Path::new(cert), SslFiletype::PEM)
            .unwrap();
        connector
            .set_private_key_file(&Path::new(key), SslFiletype::PEM)
            .unwrap();
        if env::var("DOCKER_TLS_VERIFY").is_ok() {
            let ca = &format!("{}/ca.pem", certs);
            connector.set_ca_file(&Path::new(ca)).unwrap();
        }

        // If we are attempting to connec to the docker daemon via tcp
        // we need to convert the scheme to `https` to let hyper connect.
        // Otherwise, hyper will reject the connection since it does not
        // recongnize `tcp` as a valid `http` scheme.
        let tcp_host_str = if tcp_host_str.contains("tcp://") {
            tcp_host_str.replace("tcp://", "https://")
        } else {
            tcp_host_str
        };

        Docker {
            transport: Transport::EncryptedTcp {
                client: Client::builder()
                    .build(HttpsConnector::with_connector(http, connector).unwrap()),
                host: tcp_host_str,
            },
        }
    } else {
        Docker {
            transport: Transport::Tcp {
                client: Client::builder().build(http),
                host: tcp_host_str,
            },
        }
    }
}

#[cfg(not(feature = "tls"))]
fn get_docker_for_tcp(tcp_host_str: String) -> Docker {
    let http = get_http_connector();
    Docker {
        transport: Transport::Tcp {
            client: Client::builder().build(http),
            host: tcp_host_str,
        },
    }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
    /// constructs a new Docker instance for a docker host listening at a url specified by an env var `DOCKER_HOST`,
    /// falling back on unix:///var/run/docker.sock
    pub fn new() -> Docker {
        match env::var("DOCKER_HOST").ok() {
            Some(host) => {
                #[cfg(feature = "unix-socket")]
                if let Some(path) = host.strip_prefix("unix://") {
                    return Docker::unix(path);
                }
                let host = host.parse().expect("invalid url");
                Docker::host(host)
            }
            #[cfg(feature = "unix-socket")]
            None => Docker::unix("/var/run/docker.sock"),
            #[cfg(not(feature = "unix-socket"))]
            None => panic!("Unix socket support is disabled"),
        }
    }

    /// Creates a new docker instance for a docker host
    /// listening on a given Unix socket.
    #[cfg(feature = "unix-socket")]
    pub fn unix<S>(socket_path: S) -> Docker
    where
        S: Into<String>,
    {
        Docker {
            transport: Transport::Unix {
                client: Client::builder()
                    .pool_max_idle_per_host(0)
                    .build(UnixConnector),
                path: socket_path.into(),
            },
        }
    }

    /// constructs a new Docker instance for docker host listening at the given host url
    pub fn host(host: Uri) -> Docker {
        let tcp_host_str = format!(
            "{}://{}:{}",
            host.scheme_str().unwrap(),
            host.host().unwrap().to_owned(),
            host.port_u16().unwrap_or(80)
        );

        match host.scheme_str() {
            #[cfg(feature = "unix-socket")]
            Some("unix") => Docker {
                transport: Transport::Unix {
                    client: Client::builder().build(UnixConnector),
                    path: host.path().to_owned(),
                },
            },

            #[cfg(not(feature = "unix-socket"))]
            Some("unix") => panic!("Unix socket support is disabled"),

            _ => get_docker_for_tcp(tcp_host_str),
        }
    }

    /// Exports an interface for interacting with docker images
    pub fn images(&self) -> Images {
        Images::new(self)
    }

    /// Exports an interface for interacting with docker containers
    pub fn containers(&self) -> Containers {
        Containers::new(self)
    }

    /// Exports an interface for interacting with docker services
    pub fn services(&self) -> Services {
        Services::new(self)
    }

    pub fn networks(&self) -> Networks {
        Networks::new(self)
    }

    pub fn volumes(&self) -> Volumes {
        Volumes::new(self)
    }

    /// Returns version information associated with the docker daemon
    pub async fn version(&self) -> Result<Version> {
        self.get_json("/version").await
    }

    /// Returns information associated with the docker daemon
    pub async fn info(&self) -> Result<Info> {
        self.get_json("/info").await
    }

    /// Returns a simple ping response indicating the docker daemon is accessible
    pub async fn ping(&self) -> Result<String> {
        self.get("/_ping").await
    }

    /// Returns a stream of docker events
    pub fn events<'a>(
        &'a self,
        opts: &EventsOptions,
    ) -> impl Stream<Item = Result<Event>> + Unpin + 'a {
        let mut path = vec!["/events".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let reader = Box::pin(
            self.stream_get(path.join("?"))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
        )
        .into_async_read();

        let codec = futures_codec::LinesCodec {};

        Box::pin(
            futures_codec::FramedRead::new(reader, codec)
                .map_err(Error::IO)
                .and_then(|s: String| async move {
                    serde_json::from_str(&s).map_err(Error::SerdeJsonError)
                }),
        )
    }

    //
    // Utility functions to make requests
    //

    async fn get(
        &self,
        endpoint: &str,
    ) -> Result<String> {
        self.transport
            .request(Method::GET, endpoint, Option::<(Body, Mime)>::None)
            .await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T> {
        let raw_string = self
            .transport
            .request(Method::GET, endpoint, Option::<(Body, Mime)>::None)
            .await?;

        Ok(serde_json::from_str::<T>(&raw_string)?)
    }

    async fn post(
        &self,
        endpoint: &str,
        body: Option<(Body, Mime)>,
    ) -> Result<String> {
        self.transport.request(Method::POST, endpoint, body).await
    }

    async fn put(
        &self,
        endpoint: &str,
        body: Option<(Body, Mime)>,
    ) -> Result<String> {
        self.transport.request(Method::PUT, endpoint, body).await
    }

    async fn post_json<T, B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: Into<Body>,
    {
        let string = self.transport.request(Method::POST, endpoint, body).await?;

        Ok(serde_json::from_str::<T>(&string)?)
    }

    async fn delete(
        &self,
        endpoint: &str,
    ) -> Result<String> {
        self.transport
            .request(Method::DELETE, endpoint, Option::<(Body, Mime)>::None)
            .await
    }

    async fn delete_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T> {
        let string = self
            .transport
            .request(Method::DELETE, endpoint, Option::<(Body, Mime)>::None)
            .await?;

        Ok(serde_json::from_str::<T>(&string)?)
    }

    /// Send a streaming post request.
    ///
    /// Use stream_post_into_values if the endpoint returns JSON values
    fn stream_post<'a, H>(
        &'a self,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(Body, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<hyper::body::Bytes>> + 'a
    where
        H: IntoIterator<Item = (&'static str, String)> + 'a,
    {
        self.transport
            .stream_chunks(Method::POST, endpoint, body, headers)
    }

    /// Send a streaming post request that returns a stream of JSON values
    ///
    /// Assumes that each received chunk contains one or more JSON values
    fn stream_post_into_values<'a, H>(
        &'a self,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(Body, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<Value>> + 'a
    where
        H: IntoIterator<Item = (&'static str, String)> + 'a,
    {
        self.stream_post(endpoint, body, headers)
            .and_then(|chunk| async move {
                let stream = futures_util::stream::iter(
                    serde_json::Deserializer::from_slice(&chunk)
                        .into_iter()
                        .collect::<Vec<_>>(),
                )
                .map_err(Error::from);

                Ok(stream)
            })
            .try_flatten()
    }

    fn stream_get<'a>(
        &'a self,
        endpoint: impl AsRef<str> + Unpin + 'a,
    ) -> impl Stream<Item = Result<hyper::body::Bytes>> + 'a {
        let headers = Some(Vec::default());
        self.transport
            .stream_chunks(Method::GET, endpoint, Option::<(Body, Mime)>::None, headers)
    }

    async fn stream_post_upgrade<'a>(
        &'a self,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(Body, Mime)>,
    ) -> Result<impl futures_util::io::AsyncRead + futures_util::io::AsyncWrite + 'a> {
        self.transport
            .stream_upgrade(Method::POST, endpoint, body)
            .await
    }
}

impl Default for Docker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "unix-socket")]
    #[test]
    fn unix_host_env() {
        use super::Docker;
        use std::env;
        env::set_var("DOCKER_HOST", "unix:///docker.sock");
        let d = Docker::new();
        match d.transport {
            crate::transport::Transport::Unix { path, .. } => {
                assert_eq!(path, "/docker.sock");
            }
            _ => {
                panic!("Expected transport to be unix.");
            }
        }
        env::set_var("DOCKER_HOST", "http://localhost:8000");
        let d = Docker::new();
        match d.transport {
            crate::transport::Transport::Tcp { host, .. } => {
                assert_eq!(host, "http://localhost:8000");
            }
            _ => {
                panic!("Expected transport to be http.");
            }
        }
    }
}
