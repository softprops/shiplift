//! Create and manage containers.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Container>

use std::{collections::HashMap, hash::Hash, io, iter::Peekable, path::Path, time::Duration};

use futures_util::{
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
    TryStreamExt,
};
use hyper::Body;
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use url::form_urlencoded;

use crate::{
    docker::Docker,
    errors::{Error, Result},
    exec::{Exec, ExecContainerOptions},
    image::Config,
    network::{NetworkInfo, NetworkSettings},
    transport::Payload,
    tty::{self, Multiplexer as TtyMultiPlexer},
};

#[cfg(feature = "chrono")]
use crate::datetime::datetime_from_unix_timestamp;
#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

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

/// Options for filtering container list results
#[derive(Default, Debug)]
pub struct ContainerListOptions {
    params: HashMap<&'static str, String>,
}

impl ContainerListOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> ContainerListOptionsBuilder {
        ContainerListOptionsBuilder::default()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(&self.params)
                    .finish(),
            )
        }
    }
}

/// Filter options for container listings
pub enum ContainerFilter {
    ExitCode(u64),
    Status(String),
    LabelName(String),
    Label(String, String),
}

/// Builder interface for `ContainerListOptions`
#[derive(Default)]
pub struct ContainerListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl ContainerListOptionsBuilder {
    pub fn filter(
        &mut self,
        filters: Vec<ContainerFilter>,
    ) -> &mut Self {
        let mut param = HashMap::new();
        for f in filters {
            match f {
                ContainerFilter::ExitCode(c) => param.insert("exit", vec![c.to_string()]),
                ContainerFilter::Status(s) => param.insert("status", vec![s]),
                ContainerFilter::LabelName(n) => param.insert("label", vec![n]),
                ContainerFilter::Label(n, v) => param.insert("label", vec![format!("{}={}", n, v)]),
            };
        }
        // structure is a a json encoded object mapping string keys to a list
        // of string values
        self.params
            .insert("filters", serde_json::to_string(&param).unwrap());
        self
    }

    pub fn all(&mut self) -> &mut Self {
        self.params.insert("all", "true".to_owned());
        self
    }

    pub fn since(
        &mut self,
        since: &str,
    ) -> &mut Self {
        self.params.insert("since", since.to_owned());
        self
    }

    pub fn before(
        &mut self,
        before: &str,
    ) -> &mut Self {
        self.params.insert("before", before.to_owned());
        self
    }

    pub fn sized(&mut self) -> &mut Self {
        self.params.insert("size", "true".to_owned());
        self
    }

    pub fn build(&self) -> ContainerListOptions {
        ContainerListOptions {
            params: self.params.clone(),
        }
    }
}

/// Interface for building a new docker container from an existing image
#[derive(Serialize, Debug)]
pub struct ContainerOptions {
    pub name: Option<String>,
    params: HashMap<&'static str, Value>,
}

/// Function to insert a JSON value into a tree where the desired
/// location of the value is given as a path of JSON keys.
fn insert<'a, I, V>(
    key_path: &mut Peekable<I>,
    value: &V,
    parent_node: &mut Value,
) where
    V: Serialize,
    I: Iterator<Item = &'a str>,
{
    let local_key = key_path.next().unwrap();

    if key_path.peek().is_some() {
        let node = parent_node
            .as_object_mut()
            .unwrap()
            .entry(local_key.to_string())
            .or_insert(Value::Object(Map::new()));

        insert(key_path, value, node);
    } else {
        parent_node
            .as_object_mut()
            .unwrap()
            .insert(local_key.to_string(), serde_json::to_value(value).unwrap());
    }
}

impl ContainerOptions {
    /// return a new instance of a builder for options
    pub fn builder(name: &str) -> ContainerOptionsBuilder {
        ContainerOptionsBuilder::new(name)
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(&self.to_json()).map_err(Error::from)
    }

    fn to_json(&self) -> Value {
        let mut body_members = Map::new();
        // The HostConfig element gets initialized to an empty object,
        // for backward compatibility.
        body_members.insert("HostConfig".to_string(), Value::Object(Map::new()));
        let mut body = Value::Object(body_members);
        self.parse_from(&self.params, &mut body);
        body
    }

    pub fn parse_from<'a, K, V>(
        &self,
        params: &'a HashMap<K, V>,
        body: &mut Value,
    ) where
        &'a HashMap<K, V>: IntoIterator,
        K: ToString + Eq + Hash,
        V: Serialize,
    {
        for (k, v) in params.iter() {
            let key_string = k.to_string();
            insert(&mut key_string.split('.').peekable(), v, body)
        }
    }
}

#[derive(Default)]
pub struct ContainerOptionsBuilder {
    name: Option<String>,
    params: HashMap<&'static str, Value>,
}

impl ContainerOptionsBuilder {
    pub(crate) fn new(image: &str) -> Self {
        let mut params = HashMap::new();

        params.insert("Image", Value::String(image.to_owned()));
        ContainerOptionsBuilder { name: None, params }
    }

    pub fn name(
        &mut self,
        name: &str,
    ) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Specify the working dir (corresponds to the `-w` docker cli argument)
    pub fn working_dir(
        &mut self,
        working_dir: &str,
    ) -> &mut Self {
        self.params.insert("WorkingDir", json!(working_dir));
        self
    }

    /// Specify any bind mounts, taking the form of `/some/host/path:/some/container/path`
    pub fn volumes(
        &mut self,
        volumes: Vec<&str>,
    ) -> &mut Self {
        self.params.insert("HostConfig.Binds", json!(volumes));
        self
    }

    /// enable all exposed ports on the container to be mapped to random, available, ports on the host
    pub fn publish_all_ports(&mut self) -> &mut Self {
        self.params
            .insert("HostConfig.PublishAllPorts", json!(true));
        self
    }

    pub fn expose(
        &mut self,
        srcport: u32,
        protocol: &str,
        hostport: u32,
    ) -> &mut Self {
        let mut exposedport: HashMap<String, String> = HashMap::new();
        exposedport.insert("HostPort".to_string(), hostport.to_string());

        // The idea here is to go thought the 'old' port binds and to apply them to the local
        // 'port_bindings' variable, add the bind we want and replace the 'old' value
        let mut port_bindings: HashMap<String, Value> = HashMap::new();
        for (key, val) in self
            .params
            .get("HostConfig.PortBindings")
            .unwrap_or(&json!(null))
            .as_object()
            .unwrap_or(&Map::new())
            .iter()
        {
            port_bindings.insert(key.to_string(), json!(val));
        }
        port_bindings.insert(
            format!("{}/{}", srcport, protocol),
            json!(vec![exposedport]),
        );

        self.params
            .insert("HostConfig.PortBindings", json!(port_bindings));

        // Replicate the port bindings over to the exposed ports config
        let mut exposed_ports: HashMap<String, Value> = HashMap::new();
        let empty_config: HashMap<String, Value> = HashMap::new();
        for key in port_bindings.keys() {
            exposed_ports.insert(key.to_string(), json!(empty_config));
        }

        self.params.insert("ExposedPorts", json!(exposed_ports));

        self
    }

    /// Publish a port in the container without assigning a port on the host
    pub fn publish(
        &mut self,
        srcport: u32,
        protocol: &str,
    ) -> &mut Self {
        /* The idea here is to go thought the 'old' port binds
         * and to apply them to the local 'exposedport_bindings' variable,
         * add the bind we want and replace the 'old' value */
        let mut exposed_port_bindings: HashMap<String, Value> = HashMap::new();
        for (key, val) in self
            .params
            .get("ExposedPorts")
            .unwrap_or(&json!(null))
            .as_object()
            .unwrap_or(&Map::new())
            .iter()
        {
            exposed_port_bindings.insert(key.to_string(), json!(val));
        }
        exposed_port_bindings.insert(format!("{}/{}", srcport, protocol), json!({}));

        // Replicate the port bindings over to the exposed ports config
        let mut exposed_ports: HashMap<String, Value> = HashMap::new();
        let empty_config: HashMap<String, Value> = HashMap::new();
        for key in exposed_port_bindings.keys() {
            exposed_ports.insert(key.to_string(), json!(empty_config));
        }

        self.params.insert("ExposedPorts", json!(exposed_ports));

        self
    }

    pub fn links(
        &mut self,
        links: Vec<&str>,
    ) -> &mut Self {
        self.params.insert("HostConfig.Links", json!(links));
        self
    }

    pub fn memory(
        &mut self,
        memory: u64,
    ) -> &mut Self {
        self.params.insert("HostConfig.Memory", json!(memory));
        self
    }

    /// Total memory limit (memory + swap) in bytes. Set to -1 (default) to enable unlimited swap.
    pub fn memory_swap(
        &mut self,
        memory_swap: i64,
    ) -> &mut Self {
        self.params
            .insert("HostConfig.MemorySwap", json!(memory_swap));
        self
    }

    /// CPU quota in units of 10<sup>-9</sup> CPUs. Set to 0 (default) for there to be no limit.
    ///
    /// For example, setting `nano_cpus` to `500_000_000` results in the container being allocated
    /// 50% of a single CPU, while `2_000_000_000` results in the container being allocated 2 CPUs.
    pub fn nano_cpus(
        &mut self,
        nano_cpus: u64,
    ) -> &mut Self {
        self.params.insert("HostConfig.NanoCpus", json!(nano_cpus));
        self
    }

    /// CPU quota in units of CPUs. This is a wrapper around `nano_cpus` to do the unit conversion.
    ///
    /// See [`nano_cpus`](#method.nano_cpus).
    pub fn cpus(
        &mut self,
        cpus: f64,
    ) -> &mut Self {
        self.nano_cpus((1_000_000_000.0 * cpus) as u64)
    }

    /// Sets an integer value representing the container's relative CPU weight versus other
    /// containers.
    pub fn cpu_shares(
        &mut self,
        cpu_shares: u32,
    ) -> &mut Self {
        self.params
            .insert("HostConfig.CpuShares", json!(cpu_shares));
        self
    }

    pub fn labels(
        &mut self,
        labels: &HashMap<&str, &str>,
    ) -> &mut Self {
        self.params.insert("Labels", json!(labels));
        self
    }

    /// Whether to attach to `stdin`.
    pub fn attach_stdin(
        &mut self,
        attach: bool,
    ) -> &mut Self {
        self.params.insert("AttachStdin", json!(attach));
        self.params.insert("OpenStdin", json!(attach));
        self
    }

    /// Whether to attach to `stdout`.
    pub fn attach_stdout(
        &mut self,
        attach: bool,
    ) -> &mut Self {
        self.params.insert("AttachStdout", json!(attach));
        self
    }

    /// Whether to attach to `stderr`.
    pub fn attach_stderr(
        &mut self,
        attach: bool,
    ) -> &mut Self {
        self.params.insert("AttachStderr", json!(attach));
        self
    }

    /// Whether standard streams should be attached to a TTY.
    pub fn tty(
        &mut self,
        tty: bool,
    ) -> &mut Self {
        self.params.insert("Tty", json!(tty));
        self
    }

    pub fn extra_hosts(
        &mut self,
        hosts: Vec<&str>,
    ) -> &mut Self {
        self.params.insert("HostConfig.ExtraHosts", json!(hosts));
        self
    }

    pub fn volumes_from(
        &mut self,
        volumes: Vec<&str>,
    ) -> &mut Self {
        self.params.insert("HostConfig.VolumesFrom", json!(volumes));
        self
    }

    pub fn network_mode(
        &mut self,
        network: &str,
    ) -> &mut Self {
        self.params.insert("HostConfig.NetworkMode", json!(network));
        self
    }

    pub fn env<E, S>(
        &mut self,
        envs: E,
    ) -> &mut Self
    where
        S: AsRef<str> + Serialize,
        E: AsRef<[S]> + Serialize,
    {
        self.params.insert("Env", json!(envs));
        self
    }

    pub fn cmd(
        &mut self,
        cmds: Vec<&str>,
    ) -> &mut Self {
        self.params.insert("Cmd", json!(cmds));
        self
    }

    pub fn entrypoint(
        &mut self,
        entrypoint: &str,
    ) -> &mut Self {
        self.params.insert("Entrypoint", json!(entrypoint));
        self
    }

    pub fn capabilities(
        &mut self,
        capabilities: Vec<&str>,
    ) -> &mut Self {
        self.params.insert("HostConfig.CapAdd", json!(capabilities));
        self
    }

    pub fn devices(
        &mut self,
        devices: Vec<HashMap<String, String>>,
    ) -> &mut Self {
        self.params.insert("HostConfig.Devices", json!(devices));
        self
    }

    pub fn log_driver(
        &mut self,
        log_driver: &str,
    ) -> &mut Self {
        self.params
            .insert("HostConfig.LogConfig.Type", json!(log_driver));
        self
    }

    pub fn restart_policy(
        &mut self,
        name: &str,
        maximum_retry_count: u64,
    ) -> &mut Self {
        self.params
            .insert("HostConfig.RestartPolicy.Name", json!(name));
        if name == "on-failure" {
            self.params.insert(
                "HostConfig.RestartPolicy.MaximumRetryCount",
                json!(maximum_retry_count),
            );
        }
        self
    }

    pub fn auto_remove(
        &mut self,
        set: bool,
    ) -> &mut Self {
        self.params.insert("HostConfig.AutoRemove", json!(set));
        self
    }

    /// Signal to stop a container as a string. Default is "SIGTERM".
    pub fn stop_signal(
        &mut self,
        sig: &str,
    ) -> &mut Self {
        self.params.insert("StopSignal", json!(sig));
        self
    }

    /// Signal to stop a container as an integer. Default is 15 (SIGTERM).
    pub fn stop_signal_num(
        &mut self,
        sig: u64,
    ) -> &mut Self {
        self.params.insert("StopSignal", json!(sig));
        self
    }

    /// Timeout to stop a container. Only seconds are counted. Default is 10s
    pub fn stop_timeout(
        &mut self,
        timeout: Duration,
    ) -> &mut Self {
        self.params.insert("StopTimeout", json!(timeout.as_secs()));
        self
    }

    pub fn userns_mode(
        &mut self,
        mode: &str,
    ) -> &mut Self {
        self.params.insert("HostConfig.UsernsMode", json!(mode));
        self
    }

    pub fn privileged(
        &mut self,
        set: bool,
    ) -> &mut Self {
        self.params.insert("HostConfig.Privileged", json!(set));
        self
    }

    pub fn user(
        &mut self,
        user: &str,
    ) -> &mut Self {
        self.params.insert("User", json!(user));
        self
    }

    pub fn build(&self) -> ContainerOptions {
        ContainerOptions {
            name: self.name.clone(),
            params: self.params.clone(),
        }
    }
}

/// Options for controlling log request results
#[derive(Default, Debug)]
pub struct LogsOptions {
    params: HashMap<&'static str, String>,
}

impl LogsOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> LogsOptionsBuilder {
        LogsOptionsBuilder::default()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(&self.params)
                    .finish(),
            )
        }
    }
}

/// Builder interface for `LogsOptions`
#[derive(Default)]
pub struct LogsOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl LogsOptionsBuilder {
    pub fn follow(
        &mut self,
        f: bool,
    ) -> &mut Self {
        self.params.insert("follow", f.to_string());
        self
    }

    pub fn stdout(
        &mut self,
        s: bool,
    ) -> &mut Self {
        self.params.insert("stdout", s.to_string());
        self
    }

    pub fn stderr(
        &mut self,
        s: bool,
    ) -> &mut Self {
        self.params.insert("stderr", s.to_string());
        self
    }

    pub fn timestamps(
        &mut self,
        t: bool,
    ) -> &mut Self {
        self.params.insert("timestamps", t.to_string());
        self
    }

    /// how_many can either be "all" or a to_string() of the number
    pub fn tail(
        &mut self,
        how_many: &str,
    ) -> &mut Self {
        self.params.insert("tail", how_many.to_owned());
        self
    }

    #[cfg(feature = "chrono")]
    pub fn since<Tz>(
        &mut self,
        timestamp: &chrono::DateTime<Tz>,
    ) -> &mut Self
    where
        Tz: chrono::TimeZone,
    {
        self.params
            .insert("since", timestamp.timestamp().to_string());
        self
    }

    #[cfg(not(feature = "chrono"))]
    pub fn since(
        &mut self,
        timestamp: i64,
    ) -> &mut Self {
        self.params.insert("since", timestamp.to_string());
        self
    }

    pub fn build(&self) -> LogsOptions {
        LogsOptions {
            params: self.params.clone(),
        }
    }
}

/// Options for controlling log request results
#[derive(Default, Debug)]
pub struct RmContainerOptions {
    params: HashMap<&'static str, String>,
}

impl RmContainerOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> RmContainerOptionsBuilder {
        RmContainerOptionsBuilder::default()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(&self.params)
                    .finish(),
            )
        }
    }
}

/// Builder interface for `LogsOptions`
#[derive(Default)]
pub struct RmContainerOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl RmContainerOptionsBuilder {
    pub fn force(
        &mut self,
        f: bool,
    ) -> &mut Self {
        self.params.insert("force", f.to_string());
        self
    }

    pub fn volumes(
        &mut self,
        s: bool,
    ) -> &mut Self {
        self.params.insert("v", s.to_string());
        self
    }

    pub fn build(&self) -> RmContainerOptions {
        RmContainerOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerInfo {
    #[cfg(feature = "chrono")]
    #[serde(deserialize_with = "datetime_from_unix_timestamp")]
    pub created: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created: u64,
    pub command: String,
    pub id: String,
    pub image: String,
    #[serde(rename = "ImageID")]
    pub image_id: String,
    pub labels: HashMap<String, String>,
    pub names: Vec<String>,
    pub ports: Vec<Port>,
    pub state: String,
    pub status: String,
    pub size_rw: Option<u64>,
    pub size_root_fs: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerDetails {
    pub app_armor_profile: String,
    pub args: Vec<String>,
    pub config: Config,
    #[cfg(feature = "chrono")]
    pub created: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created: String,
    pub driver: String,
    // pub ExecIDs: ??
    pub host_config: HostConfig,
    pub hostname_path: String,
    pub hosts_path: String,
    pub log_path: String,
    pub id: String,
    pub image: String,
    pub mount_label: String,
    pub name: String,
    pub network_settings: NetworkSettings,
    pub path: String,
    pub process_label: String,
    pub resolv_conf_path: String,
    pub restart_count: u64,
    pub state: State,
    pub mounts: Vec<Mount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub mode: String,
    #[serde(rename = "RW")]
    pub rw: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct State {
    pub error: String,
    pub exit_code: u64,
    #[cfg(feature = "chrono")]
    pub finished_at: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub finished_at: String,
    #[serde(rename = "OOMKilled")]
    pub oom_killed: bool,
    pub paused: bool,
    pub pid: u64,
    pub restarting: bool,
    pub running: bool,
    #[cfg(feature = "chrono")]
    pub started_at: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub started_at: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HostConfig {
    pub cgroup_parent: Option<String>,
    #[serde(rename = "ContainerIDFile")]
    pub container_id_file: String,
    pub cpu_shares: Option<u64>,
    pub cpuset_cpus: Option<String>,
    pub memory: Option<u64>,
    pub memory_swap: Option<i64>,
    pub network_mode: String,
    pub pid_mode: Option<String>,
    pub port_bindings: Option<HashMap<String, Vec<HashMap<String, String>>>>,
    pub privileged: bool,
    pub publish_all_ports: bool,
    pub readonly_rootfs: Option<bool>, /* pub RestartPolicy: ???
                                        * pub SecurityOpt: Option<???>,
                                        * pub Ulimits: Option<???>
                                        * pub VolumesFrom: Option<??/> */
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Port {
    pub ip: Option<String>,
    pub private_port: u64,
    pub public_port: Option<u64>,
    #[serde(rename = "Type")]
    pub typ: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub read: String,
    pub networks: HashMap<String, NetworkInfo>,
    pub memory_stats: MemoryStats,
    pub blkio_stats: BlkioStats,
    pub cpu_stats: CpuStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryStats {
    pub max_usage: u64,
    pub usage: u64,
    pub failcnt: Option<u64>,
    pub limit: u64,
    pub stats: MemoryStat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryStat {
    pub total_pgmajfault: u64,
    pub cache: u64,
    pub mapped_file: u64,
    pub total_inactive_file: u64,
    pub pgpgout: u64,
    pub rss: u64,
    pub total_mapped_file: u64,
    pub writeback: u64,
    pub unevictable: u64,
    pub pgpgin: u64,
    pub total_unevictable: u64,
    pub pgmajfault: u64,
    pub total_rss: u64,
    pub total_rss_huge: u64,
    pub total_writeback: u64,
    pub total_inactive_anon: u64,
    pub rss_huge: u64,
    pub hierarchical_memory_limit: u64,
    pub hierarchical_memsw_limit: u64,
    pub total_pgfault: u64,
    pub total_active_file: u64,
    pub active_anon: u64,
    pub total_active_anon: u64,
    pub total_pgpgout: u64,
    pub total_cache: u64,
    pub inactive_anon: u64,
    pub active_file: u64,
    pub pgfault: u64,
    pub inactive_file: u64,
    pub total_pgpgin: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CpuStats {
    pub cpu_usage: CpuUsage,
    pub system_cpu_usage: u64,
    pub throttling_data: ThrottlingData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CpuUsage {
    pub percpu_usage: Vec<u64>,
    pub usage_in_usermode: u64,
    pub total_usage: u64,
    pub usage_in_kernelmode: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThrottlingData {
    pub periods: u64,
    pub throttled_periods: u64,
    pub throttled_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlkioStats {
    pub io_service_bytes_recursive: Vec<BlkioStat>,
    pub io_serviced_recursive: Vec<BlkioStat>,
    pub io_queue_recursive: Vec<BlkioStat>,
    pub io_service_time_recursive: Vec<BlkioStat>,
    pub io_wait_time_recursive: Vec<BlkioStat>,
    pub io_merged_recursive: Vec<BlkioStat>,
    pub io_time_recursive: Vec<BlkioStat>,
    pub sectors_recursive: Vec<BlkioStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlkioStat {
    pub major: u64,
    pub minor: u64,
    pub op: String,
    pub value: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Change {
    pub kind: u64,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Top {
    pub titles: Vec<String>,
    pub processes: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerCreateInfo {
    pub id: String,
    pub warnings: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Exit {
    pub status_code: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_options_simple() {
        let builder = ContainerOptionsBuilder::new("test_image");
        let options = builder.build();

        assert_eq!(
            r#"{"HostConfig":{},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    #[test]
    fn container_options_env() {
        let options = ContainerOptionsBuilder::new("test_image")
            .env(vec!["foo", "bar"])
            .build();

        assert_eq!(
            r#"{"Env":["foo","bar"],"HostConfig":{},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    #[test]
    fn container_options_env_dynamic() {
        let env: Vec<String> = ["foo", "bar", "baz"]
            .iter()
            .map(|s| String::from(*s))
            .collect();

        let options = ContainerOptionsBuilder::new("test_image").env(&env).build();

        assert_eq!(
            r#"{"Env":["foo","bar","baz"],"HostConfig":{},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    #[test]
    fn container_options_user() {
        let options = ContainerOptionsBuilder::new("test_image")
            .user("alice")
            .build();

        assert_eq!(
            r#"{"HostConfig":{},"Image":"test_image","User":"alice"}"#,
            options.serialize().unwrap()
        );
    }

    #[test]
    fn container_options_host_config() {
        let options = ContainerOptionsBuilder::new("test_image")
            .network_mode("host")
            .auto_remove(true)
            .privileged(true)
            .build();

        assert_eq!(
            r#"{"HostConfig":{"AutoRemove":true,"NetworkMode":"host","Privileged":true},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    #[test]
    fn container_options_expose() {
        let options = ContainerOptionsBuilder::new("test_image")
            .expose(80, "tcp", 8080)
            .build();
        assert_eq!(
            r#"{"ExposedPorts":{"80/tcp":{}},"HostConfig":{"PortBindings":{"80/tcp":[{"HostPort":"8080"}]}},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
        // try exposing two
        let options = ContainerOptionsBuilder::new("test_image")
            .expose(80, "tcp", 8080)
            .expose(81, "tcp", 8081)
            .build();
        assert_eq!(
            r#"{"ExposedPorts":{"80/tcp":{},"81/tcp":{}},"HostConfig":{"PortBindings":{"80/tcp":[{"HostPort":"8080"}],"81/tcp":[{"HostPort":"8081"}]}},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    #[test]
    fn container_options_publish() {
        let options = ContainerOptionsBuilder::new("test_image")
            .publish(80, "tcp")
            .build();
        assert_eq!(
            r#"{"ExposedPorts":{"80/tcp":{}},"HostConfig":{},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
        // try exposing two
        let options = ContainerOptionsBuilder::new("test_image")
            .publish(80, "tcp")
            .publish(81, "tcp")
            .build();
        assert_eq!(
            r#"{"ExposedPorts":{"80/tcp":{},"81/tcp":{}},"HostConfig":{},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    /// Test container option PublishAllPorts
    #[test]
    fn container_options_publish_all_ports() {
        let options = ContainerOptionsBuilder::new("test_image")
            .publish_all_ports()
            .build();

        assert_eq!(
            r#"{"HostConfig":{"PublishAllPorts":true},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    /// Test container options that are nested 3 levels deep.
    #[test]
    fn container_options_nested() {
        let options = ContainerOptionsBuilder::new("test_image")
            .log_driver("fluentd")
            .build();

        assert_eq!(
            r#"{"HostConfig":{"LogConfig":{"Type":"fluentd"}},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    /// Test the restart policy settings
    #[test]
    fn container_options_restart_policy() {
        let mut options = ContainerOptionsBuilder::new("test_image")
            .restart_policy("on-failure", 10)
            .build();

        assert_eq!(
            r#"{"HostConfig":{"RestartPolicy":{"MaximumRetryCount":10,"Name":"on-failure"}},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );

        options = ContainerOptionsBuilder::new("test_image")
            .restart_policy("always", 0)
            .build();

        assert_eq!(
            r#"{"HostConfig":{"RestartPolicy":{"Name":"always"}},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn logs_options() {
        let timestamp = chrono::NaiveDateTime::from_timestamp(2_147_483_647, 0);
        let since = chrono::DateTime::<chrono::Utc>::from_utc(timestamp, chrono::Utc);

        let options = LogsOptionsBuilder::default()
            .follow(true)
            .stdout(true)
            .stderr(true)
            .timestamps(true)
            .tail("all")
            .since(&since)
            .build();

        let serialized = options.serialize().unwrap();

        assert!(serialized.contains("follow=true"));
        assert!(serialized.contains("stdout=true"));
        assert!(serialized.contains("stderr=true"));
        assert!(serialized.contains("timestamps=true"));
        assert!(serialized.contains("tail=all"));
        assert!(serialized.contains("since=2147483647"));
    }

    #[cfg(not(feature = "chrono"))]
    #[test]
    fn logs_options() {
        let options = LogsOptionsBuilder::default()
            .follow(true)
            .stdout(true)
            .stderr(true)
            .timestamps(true)
            .tail("all")
            .since(2_147_483_647)
            .build();

        let serialized = options.serialize().unwrap();

        assert!(serialized.contains("follow=true"));
        assert!(serialized.contains("stdout=true"));
        assert!(serialized.contains("stderr=true"));
        assert!(serialized.contains("timestamps=true"));
        assert!(serialized.contains("tail=all"));
        assert!(serialized.contains("since=2147483647"));
    }
}
