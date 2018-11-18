//! Interfaces for building various structures

// Std lib
use std::cmp::Eq;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::iter::IntoIterator;
use std::iter::Peekable;

// Third party
use serde::Serialize;
use serde_json::{self, map::Map, Value};
use url::form_urlencoded;

// Ours
use errors::Error;
use Result;

#[derive(Default)]
pub struct PullOptions {
    params: HashMap<&'static str, String>,
}

impl PullOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> PullOptionsBuilder {
        PullOptionsBuilder::default()
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

#[derive(Default)]
pub struct PullOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl PullOptionsBuilder {
    pub fn image<I>(
        &mut self,
        img: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.params.insert("fromImage", img.into());
        self
    }

    pub fn src<S>(
        &mut self,
        s: S,
    ) -> &mut Self
    where
        S: Into<String>,
    {
        self.params.insert("fromSrc", s.into());
        self
    }

    pub fn repo<R>(
        &mut self,
        r: R,
    ) -> &mut Self
    where
        R: Into<String>,
    {
        self.params.insert("repo", r.into());
        self
    }

    pub fn tag<T>(
        &mut self,
        t: T,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        self.params.insert("tag", t.into());
        self
    }

    pub fn build(&self) -> PullOptions {
        PullOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Default)]
pub struct BuildOptions {
    pub path: String,
    params: HashMap<&'static str, String>,
}

impl BuildOptions {
    /// return a new instance of a builder for options
    /// path is expected to be a file path to a directory containing a Dockerfile
    /// describing how to build a Docker image
    pub fn builder<S>(path: S) -> BuildOptionsBuilder
    where
        S: Into<String>,
    {
        BuildOptionsBuilder::new(path)
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

#[derive(Default)]
pub struct BuildOptionsBuilder {
    path: String,
    params: HashMap<&'static str, String>,
}

impl BuildOptionsBuilder {
    /// path is expected to be a file path to a directory containing a Dockerfile
    /// describing how to build a Docker image
    pub(crate) fn new<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        BuildOptionsBuilder {
            path: path.into(),
            ..Default::default()
        }
    }

    /// set the name of the docker file. defaults to "DockerFile"
    pub fn dockerfile<P>(
        &mut self,
        path: P,
    ) -> &mut Self
    where
        P: Into<String>,
    {
        self.params.insert("dockerfile", path.into());
        self
    }

    /// tag this image with a name after building it
    pub fn tag<T>(
        &mut self,
        t: T,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        self.params.insert("t", t.into());
        self
    }

    pub fn remote<R>(
        &mut self,
        r: R,
    ) -> &mut Self
    where
        R: Into<String>,
    {
        self.params.insert("remote", r.into());
        self
    }

    /// don't use the image cache when building image
    pub fn nocache<R>(
        &mut self,
        nc: bool,
    ) -> &mut Self {
        self.params.insert("nocache", nc.to_string());
        self
    }

    pub fn rm(
        &mut self,
        r: bool,
    ) -> &mut Self {
        self.params.insert("rm", r.to_string());
        self
    }

    pub fn forcerm(
        &mut self,
        fr: bool,
    ) -> &mut Self {
        self.params.insert("forcerm", fr.to_string());
        self
    }

    /// `bridge`, `host`, `none`, `container:<name|id>`, or a custom network name.
    pub fn network_mode<T>(
        &mut self,
        t: T,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        self.params.insert("networkmode", t.into());
        self
    }

    // todo: memory
    // todo: memswap
    // todo: cpushares
    // todo: cpusetcpus
    // todo: cpuperiod
    // todo: cpuquota
    // todo: buildargs

    pub fn build(&self) -> BuildOptions {
        BuildOptions {
            path: self.path.clone(),
            params: self.params.clone(),
        }
    }
}

/// Options for filtering container list results
#[derive(Default)]
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
#[derive(Serialize)]
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

    pub fn volumes(
        &mut self,
        volumes: Vec<&str>,
    ) -> &mut Self {
        self.params.insert("HostConfig.Binds", json!(volumes));
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

        /* The idea here is to go thought the 'old' port binds
         * and to apply them to the local 'binding' variable,
         * add the bind we want and replace the 'old' value */
        let mut binding: HashMap<String, Value> = HashMap::new();
        for (key, val) in self
            .params
            .get("HostConfig.PortBindings")
            .unwrap_or(&mut json!(null))
            .as_object()
            .unwrap_or(&mut Map::new())
            .iter()
        {
            binding.insert(key.to_string(), json!(val));
        }
        binding.insert(
            format!("{}/{}", srcport, protocol),
            json!(vec![exposedport]),
        );

        self.params
            .insert("HostConfig.PortBindings", json!(binding));
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

    pub fn labels(
        &mut self,
        labels: &HashMap<&str, &str>,
    ) -> &mut Self {
        self.params.insert("Labels", json!(labels));
        self
    }

    /// Whether to attach to `stdin`.
    pub fn attach_stdin(&mut self, attach: bool) -> &mut Self {
        self.params.insert("AttachStdin", json!(attach));
        self.params.insert("OpenStdin", json!(attach));
        self
    }

    /// Whether to attach to `stdout`.
    pub fn attach_stdout(&mut self, attach: bool) -> &mut Self {
        self.params.insert("AttachStdout", json!(attach));
        self
    }

    /// Whether to attach to `stderr`.
    pub fn attach_stderr(&mut self, attach: bool) -> &mut Self {
        self.params.insert("AttachStderr", json!(attach));
        self
    }

    /// Whether standard streams should be attached to a TTY.
    pub fn tty(&mut self, tty: bool) -> &mut Self {
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

    pub fn env(
        &mut self,
        envs: Vec<&str>,
    ) -> &mut Self {
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

    pub fn build(&self) -> ContainerOptions {
        ContainerOptions {
            name: self.name.clone(),
            params: self.params.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ExecContainerOptions {
    params: HashMap<&'static str, Vec<String>>,
    params_bool: HashMap<&'static str, bool>,
}

impl ExecContainerOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> ExecContainerOptionsBuilder {
        ExecContainerOptionsBuilder::default()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        let mut body = serde_json::Map::new();

        for (k, v) in &self.params {
            body.insert(
                k.to_string(),
                serde_json::to_value(v).map_err(Error::SerdeJsonError)?,
            );
        }

        for (k, v) in &self.params_bool {
            body.insert(
                k.to_string(),
                serde_json::to_value(v).map_err(Error::SerdeJsonError)?,
            );
        }

        serde_json::to_string(&body).map_err(Error::from)
    }
}

#[derive(Default)]
pub struct ExecContainerOptionsBuilder {
    params: HashMap<&'static str, Vec<String>>,
    params_bool: HashMap<&'static str, bool>,
}

impl ExecContainerOptionsBuilder {
    /// Command to run, as an array of strings
    pub fn cmd(
        &mut self,
        cmds: Vec<&str>,
    ) -> &mut Self {
        for cmd in cmds {
            self.params
                .entry("Cmd")
                .or_insert_with(Vec::new)
                .push(cmd.to_owned());
        }
        self
    }

    /// A list of environment variables in the form "VAR=value"
    pub fn env(
        &mut self,
        envs: Vec<&str>,
    ) -> &mut Self {
        for env in envs {
            self.params
                .entry("Env")
                .or_insert_with(Vec::new)
                .push(env.to_owned());
        }
        self
    }

    /// Attach to stdout of the exec command
    pub fn attach_stdout(
        &mut self,
        stdout: bool,
    ) -> &mut Self {
        self.params_bool.insert("AttachStdout", stdout);
        self
    }

    /// Attach to stderr of the exec command
    pub fn attach_stderr(
        &mut self,
        stderr: bool,
    ) -> &mut Self {
        self.params_bool.insert("AttachStderr", stderr);
        self
    }

    pub fn build(&self) -> ExecContainerOptions {
        ExecContainerOptions {
            params: self.params.clone(),
            params_bool: self.params_bool.clone(),
        }
    }
}

/// Options for filtering streams of Docker events
#[derive(Default)]
pub struct EventsOptions {
    params: HashMap<&'static str, String>,
}

impl EventsOptions {
    pub fn builder() -> EventsOptionsBuilder {
        EventsOptionsBuilder::default()
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

#[derive(Copy, Clone)]
pub enum EventFilterType {
    Container,
    Image,
    Volume,
    Network,
    Daemon,
}

fn event_filter_type_to_string(filter: EventFilterType) -> &'static str {
    match filter {
        EventFilterType::Container => "container",
        EventFilterType::Image => "image",
        EventFilterType::Volume => "volume",
        EventFilterType::Network => "network",
        EventFilterType::Daemon => "daemon",
    }
}

/// Filter options for image listings
pub enum EventFilter {
    Container(String),
    Event(String),
    Image(String),
    Label(String),
    Type(EventFilterType),
    Volume(String),
    Network(String),
    Daemon(String),
}

/// Builder interface for `EventOptions`
#[derive(Default)]
pub struct EventsOptionsBuilder {
    params: HashMap<&'static str, String>,
    events: Vec<String>,
    containers: Vec<String>,
    images: Vec<String>,
    labels: Vec<String>,
    volumes: Vec<String>,
    networks: Vec<String>,
    daemons: Vec<String>,
    types: Vec<String>,
}

impl EventsOptionsBuilder {
    /// Filter events since a given timestamp
    pub fn since(
        &mut self,
        ts: &u64,
    ) -> &mut Self {
        self.params.insert("since", ts.to_string());
        self
    }

    /// Filter events until a given timestamp
    pub fn until(
        &mut self,
        ts: &u64,
    ) -> &mut Self {
        self.params.insert("until", ts.to_string());
        self
    }

    pub fn filter(
        &mut self,
        filters: Vec<EventFilter>,
    ) -> &mut Self {
        let mut params = HashMap::new();
        for f in filters {
            match f {
                EventFilter::Container(n) => {
                    self.containers.push(n);
                    params.insert("container", self.containers.clone())
                }
                EventFilter::Event(n) => {
                    self.events.push(n);
                    params.insert("event", self.events.clone())
                }
                EventFilter::Image(n) => {
                    self.images.push(n);
                    params.insert("image", self.images.clone())
                }
                EventFilter::Label(n) => {
                    self.labels.push(n);
                    params.insert("label", self.labels.clone())
                }
                EventFilter::Volume(n) => {
                    self.volumes.push(n);
                    params.insert("volume", self.volumes.clone())
                }
                EventFilter::Network(n) => {
                    self.networks.push(n);
                    params.insert("network", self.networks.clone())
                }
                EventFilter::Daemon(n) => {
                    self.daemons.push(n);
                    params.insert("daemon", self.daemons.clone())
                }
                EventFilter::Type(n) => {
                    let event_type = event_filter_type_to_string(n).to_string();
                    self.types.push(event_type);
                    params.insert("type", self.types.clone())
                }
            };
        }
        self.params
            .insert("filters", serde_json::to_string(&params).unwrap());
        self
    }

    pub fn build(&self) -> EventsOptions {
        EventsOptions {
            params: self.params.clone(),
        }
    }
}

/// Options for controlling log request results
#[derive(Default)]
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

    pub fn build(&self) -> LogsOptions {
        LogsOptions {
            params: self.params.clone(),
        }
    }
}

/// Filter options for image listings
pub enum ImageFilter {
    Dangling,
    LabelName(String),
    Label(String, String),
}

/// Options for filtering image list results
#[derive(Default)]
pub struct ImageListOptions {
    params: HashMap<&'static str, String>,
}

impl ImageListOptions {
    pub fn builder() -> ImageListOptionsBuilder {
        ImageListOptionsBuilder::default()
    }
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

/// Builder interface for `ImageListOptions`
#[derive(Default)]
pub struct ImageListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl ImageListOptionsBuilder {
    pub fn digests(
        &mut self,
        d: bool,
    ) -> &mut Self {
        self.params.insert("digests", d.to_string());
        self
    }

    pub fn all(
        &mut self,
        a: bool,
    ) -> &mut Self {
        self.params.insert("all", a.to_string());
        self
    }

    pub fn filter_name(
        &mut self,
        name: &str,
    ) -> &mut Self {
        self.params.insert("filter", name.to_owned());
        self
    }

    pub fn filter(
        &mut self,
        filters: Vec<ImageFilter>,
    ) -> &mut Self {
        let mut param = HashMap::new();
        for f in filters {
            match f {
                ImageFilter::Dangling => param.insert("dangling", vec![true.to_string()]),
                ImageFilter::LabelName(n) => param.insert("label", vec![n]),
                ImageFilter::Label(n, v) => param.insert("label", vec![format!("{}={}", n, v)]),
            };
        }
        // structure is a a json encoded object mapping string keys to a list
        // of string values
        self.params
            .insert("filters", serde_json::to_string(&param).unwrap());
        self
    }

    pub fn build(&self) -> ImageListOptions {
        ImageListOptions {
            params: self.params.clone(),
        }
    }
}

/// Options for controlling log request results
#[derive(Default)]
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

/// Options for filtering networks list results
#[derive(Default)]
pub struct NetworkListOptions {
    params: HashMap<&'static str, String>,
}

impl NetworkListOptions {
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

/// Interface for creating new docker network
#[derive(Serialize)]
pub struct NetworkCreateOptions {
    pub name: Option<String>,
    params: HashMap<&'static str, String>,
    params_hash: HashMap<String, Vec<HashMap<String, String>>>,
}

impl NetworkCreateOptions {
    /// return a new instance of a builder for options
    pub fn builder(name: &str) -> NetworkCreateOptionsBuilder {
        NetworkCreateOptionsBuilder::new(name)
    }

    fn to_json(&self) -> Value {
        let mut body = serde_json::Map::new();
        self.parse_from(&self.params, &mut body);
        self.parse_from(&self.params_hash, &mut body);
        Value::Object(body)
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(&self.to_json()).map_err(Error::from)
    }

    pub fn parse_from<'a, K, V>(
        &self,
        params: &'a HashMap<K, V>,
        body: &mut serde_json::Map<String, Value>,
    ) where
        &'a HashMap<K, V>: IntoIterator,
        K: ToString + Eq + Hash,
        V: Serialize,
    {
        for (k, v) in params.iter() {
            let key = k.to_string();
            let value = serde_json::to_value(v).unwrap();

            body.insert(key, value);
        }
    }
}

#[derive(Default)]
pub struct NetworkCreateOptionsBuilder {
    name: Option<String>,
    params: HashMap<&'static str, String>,
    params_hash: HashMap<String, Vec<HashMap<String, String>>>,
}

impl NetworkCreateOptionsBuilder {
    pub(crate) fn new(name: &str) -> Self {
        let mut params = HashMap::new();
        let params_hash = HashMap::new();

        params.insert("Name", name.to_owned());
        NetworkCreateOptionsBuilder {
            name: None,
            params,
            params_hash,
        }
    }

    pub fn driver(
        &mut self,
        name: &str,
    ) -> &mut Self {
        if !name.is_empty() {
            self.params.insert("Driver", name.to_owned());
        }
        self
    }

    pub fn label(
        &mut self,
        labels: Vec<HashMap<String, String>>,
    ) -> &mut Self {
        for l in labels {
            self.params_hash
                .entry("Labels".to_string())
                .or_insert_with(Vec::new)
                .push(l)
        }
        self
    }

    pub fn build(&self) -> NetworkCreateOptions {
        NetworkCreateOptions {
            name: self.name.clone(),
            params: self.params.clone(),
            params_hash: self.params_hash.clone(),
        }
    }
}

/// Interface for connect container to network
#[derive(Serialize)]
pub struct ContainerConnectionOptions {
    pub container: Option<String>,
    params: HashMap<&'static str, String>,
}

impl ContainerConnectionOptions {
    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Error::from)
    }

    pub fn parse_from<'a, K, V>(
        &self,
        params: &'a HashMap<K, V>,
        body: &mut BTreeMap<String, Value>,
    ) where
        &'a HashMap<K, V>: IntoIterator,
        K: ToString + Eq + Hash,
        V: Serialize,
    {
        for (k, v) in params.iter() {
            let key = k.to_string();
            let value = serde_json::to_value(v).unwrap();

            body.insert(key, value);
        }
    }

    pub fn new(container_id: &str) -> ContainerConnectionOptions {
        let mut params = HashMap::new();
        params.insert("Container", container_id.to_owned());
        ContainerConnectionOptions {
            container: None,
            params: params.clone(),
        }
    }

    pub fn force(&mut self) -> ContainerConnectionOptions {
        self.params.insert("Force", "true".to_owned());
        ContainerConnectionOptions {
            container: None,
            params: self.params.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContainerOptionsBuilder;

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
    fn container_options_host_config() {
        let options = ContainerOptionsBuilder::new("test_image")
            .network_mode("host")
            .build();

        assert_eq!(
            r#"{"HostConfig":{"NetworkMode":"host"},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
    }

    #[test]
    fn container_options_expose() {
        let options = ContainerOptionsBuilder::new("test_image")
            .expose(80, "tcp", 8080)
            .build();
        assert_eq!(
            r#"{"HostConfig":{"PortBindings":{"80/tcp":[{"HostPort":"8080"}]}},"Image":"test_image"}"#,
            options.serialize().unwrap()
        );
        // try exposing two
        let options = ContainerOptionsBuilder::new("test_image")
            .expose(80, "tcp", 8080)
            .expose(81, "tcp", 8081)
            .build();
        assert_eq!(
            r#"{"HostConfig":{"PortBindings":{"80/tcp":[{"HostPort":"8080"}],"81/tcp":[{"HostPort":"8081"}]}},"Image":"test_image"}"#,
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
}
