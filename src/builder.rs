//! Interfaces for building various structures

use crate::{errors::Error, Result};
use serde::Serialize;
use serde_json::{self, json, map::Map, Value};
use std::{
    cmp::Eq,
    collections::{BTreeMap, HashMap},
    hash::Hash,
    iter::{IntoIterator, Peekable},
};
use url::form_urlencoded;

#[derive(Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum RegistryAuth {
    Password {
        username: String,
        password: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,

        #[serde(rename = "serveraddress")]
        #[serde(skip_serializing_if = "Option::is_none")]
        server_address: Option<String>,
    },
    Token {
        #[serde(rename = "identitytoken")]
        identity_token: String,
    },
}

impl RegistryAuth {
    /// return a new instance with token authentication
    pub fn token<S>(token: S) -> RegistryAuth
    where
        S: Into<String>,
    {
        RegistryAuth::Token {
            identity_token: token.into(),
        }
    }

    /// return a new instance of a builder for authentication
    pub fn builder() -> RegistryAuthBuilder {
        RegistryAuthBuilder::default()
    }

    /// serialize authentication as JSON in base64
    pub fn serialize(&self) -> String {
        serde_json::to_string(self)
            .map(|c| base64::encode(&c))
            .unwrap()
    }
}

#[derive(Default)]
pub struct RegistryAuthBuilder {
    username: Option<String>,
    password: Option<String>,
    email: Option<String>,
    server_address: Option<String>,
}

impl RegistryAuthBuilder {
    pub fn username<I>(
        &mut self,
        username: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.username = Some(username.into());
        self
    }

    pub fn password<I>(
        &mut self,
        password: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.password = Some(password.into());
        self
    }

    pub fn email<I>(
        &mut self,
        email: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.email = Some(email.into());
        self
    }

    pub fn server_address<I>(
        &mut self,
        server_address: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.server_address = Some(server_address.into());
        self
    }

    pub fn build(&self) -> RegistryAuth {
        RegistryAuth::Password {
            username: self.username.clone().unwrap_or_else(String::new),
            password: self.password.clone().unwrap_or_else(String::new),
            email: self.email.clone(),
            server_address: self.server_address.clone(),
        }
    }
}

#[derive(Default, Debug)]
pub struct TagOptions {
    pub params: HashMap<&'static str, String>,
}

impl TagOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> TagOptionsBuilder {
        TagOptionsBuilder::default()
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
pub struct TagOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl TagOptionsBuilder {
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

    pub fn build(&self) -> TagOptions {
        TagOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Default, Debug)]
pub struct PullOptions {
    auth: Option<RegistryAuth>,
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

    pub(crate) fn auth_header(&self) -> Option<String> {
        self.auth.clone().map(|a| a.serialize())
    }
}

#[derive(Default)]
pub struct PullOptionsBuilder {
    auth: Option<RegistryAuth>,
    params: HashMap<&'static str, String>,
}

impl PullOptionsBuilder {
    ///  Name of the image to pull. The name may include a tag or digest.
    /// This parameter may only be used when pulling an image.
    /// If an untagged value is provided and no `tag` is provided, _all_
    /// tags will be pulled
    /// The pull is cancelled if the HTTP connection is closed.
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

    /// Repository name given to an image when it is imported. The repo may include a tag.
    /// This parameter may only be used when importing an image.
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

    /// Tag or digest. If empty when pulling an image,
    /// this causes all tags for the given image to be pulled.
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

    pub fn auth(
        &mut self,
        auth: RegistryAuth,
    ) -> &mut Self {
        self.auth = Some(auth);
        self
    }

    pub fn build(&mut self) -> PullOptions {
        PullOptions {
            auth: self.auth.take(),
            params: self.params.clone(),
        }
    }
}

#[derive(Default, Debug)]
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
    pub fn nocache(
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

    pub fn memory(
        &mut self,
        memory: u64,
    ) -> &mut Self {
        self.params.insert("memory", memory.to_string());
        self
    }

    pub fn cpu_shares(
        &mut self,
        cpu_shares: u32,
    ) -> &mut Self {
        self.params.insert("cpushares", cpu_shares.to_string());
        self
    }

    // todo: memswap
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

        /* The idea here is to go thought the 'old' port binds
         * and to apply them to the local 'port_bindings' variable,
         * add the bind we want and replace the 'old' value */
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

    /// Sets an integer value representing the container's
    /// relative CPU weight versus other containers.
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

    pub fn auto_remove(
        &mut self,
        set: bool,
    ) -> &mut Self {
        self.params.insert("HostConfig.AutoRemove", json!(set));
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

    pub fn build(&self) -> ContainerOptions {
        ContainerOptions {
            name: self.name.clone(),
            params: self.params.clone(),
        }
    }
}

#[derive(Serialize, Debug)]
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
#[derive(Default, Debug)]
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

/// Filter options for image listings
pub enum ImageFilter {
    Dangling,
    LabelName(String),
    Label(String, String),
}

/// Options for filtering image list results
#[derive(Default, Debug)]
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

    pub fn all(&mut self) -> &mut Self {
        self.params.insert("all", "true".to_owned());
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

/// Options for filtering networks list results
#[derive(Default, Debug)]
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
#[derive(Serialize, Debug)]
pub struct NetworkCreateOptions {
    params: HashMap<&'static str, Value>,
}

impl NetworkCreateOptions {
    /// return a new instance of a builder for options
    pub fn builder(name: &str) -> NetworkCreateOptionsBuilder {
        NetworkCreateOptionsBuilder::new(name)
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(&self.params).map_err(Error::from)
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
    params: HashMap<&'static str, Value>,
}

impl NetworkCreateOptionsBuilder {
    pub(crate) fn new(name: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("Name", json!(name));
        NetworkCreateOptionsBuilder { params }
    }

    pub fn driver(
        &mut self,
        name: &str,
    ) -> &mut Self {
        if !name.is_empty() {
            self.params.insert("Driver", json!(name));
        }
        self
    }

    pub fn label(
        &mut self,
        labels: HashMap<String, String>,
    ) -> &mut Self {
        self.params.insert("Labels", json!(labels));
        self
    }

    pub fn build(&self) -> NetworkCreateOptions {
        NetworkCreateOptions {
            params: self.params.clone(),
        }
    }
}

/// Interface for connect container to network
#[derive(Serialize, Debug)]
pub struct ContainerConnectionOptions {
    params: HashMap<&'static str, Value>,
}

impl ContainerConnectionOptions {
    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(&self.params).map_err(Error::from)
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

    /// return a new instance of a builder for options
    pub fn builder(container_id: &str) -> ContainerConnectionOptionsBuilder {
        ContainerConnectionOptionsBuilder::new(container_id)
    }
}

#[derive(Default)]
pub struct ContainerConnectionOptionsBuilder {
    params: HashMap<&'static str, Value>,
}

impl ContainerConnectionOptionsBuilder {
    pub(crate) fn new(container_id: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("Container", json!(container_id));
        ContainerConnectionOptionsBuilder { params }
    }

    pub fn aliases(
        &mut self,
        aliases: Vec<&str>,
    ) -> &mut Self {
        self.params
            .insert("EndpointConfig", json!({ "Aliases": json!(aliases) }));
        self
    }

    pub fn force(&mut self) -> &mut Self {
        self.params.insert("Force", json!(true));
        self
    }

    pub fn build(&self) -> ContainerConnectionOptions {
        ContainerConnectionOptions {
            params: self.params.clone(),
        }
    }
}

/// Interface for creating volumes
#[derive(Serialize, Debug)]
pub struct VolumeCreateOptions {
    params: HashMap<&'static str, Value>,
}

impl VolumeCreateOptions {
    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(&self.params).map_err(Error::from)
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

    /// return a new instance of a builder for options
    pub fn builder() -> VolumeCreateOptionsBuilder {
        VolumeCreateOptionsBuilder::new()
    }
}

#[derive(Default)]
pub struct VolumeCreateOptionsBuilder {
    params: HashMap<&'static str, Value>,
}

impl VolumeCreateOptionsBuilder {
    pub(crate) fn new() -> Self {
        let params = HashMap::new();
        VolumeCreateOptionsBuilder { params }
    }

    pub fn name(
        &mut self,
        name: &str,
    ) -> &mut Self {
        self.params.insert("Name", json!(name));
        self
    }

    pub fn labels(
        &mut self,
        labels: &HashMap<&str, &str>,
    ) -> &mut Self {
        self.params.insert("Labels", json!(labels));
        self
    }

    pub fn build(&self) -> VolumeCreateOptions {
        VolumeCreateOptions {
            params: self.params.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ContainerOptionsBuilder, LogsOptionsBuilder, RegistryAuth};

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

    /// Test registry auth with token
    #[test]
    fn registry_auth_token() {
        let options = RegistryAuth::token("abc");
        assert_eq!(
            base64::encode(r#"{"identitytoken":"abc"}"#),
            options.serialize()
        );
    }

    /// Test registry auth with username and password
    #[test]
    fn registry_auth_password_simple() {
        let options = RegistryAuth::builder()
            .username("user_abc")
            .password("password_abc")
            .build();
        assert_eq!(
            base64::encode(r#"{"username":"user_abc","password":"password_abc"}"#),
            options.serialize()
        );
    }

    /// Test registry auth with all fields
    #[test]
    fn registry_auth_password_all() {
        let options = RegistryAuth::builder()
            .username("user_abc")
            .password("password_abc")
            .email("email_abc")
            .server_address("https://example.org")
            .build();
        assert_eq!(
            base64::encode(
                r#"{"username":"user_abc","password":"password_abc","email":"email_abc","serveraddress":"https://example.org"}"#
            ),
            options.serialize()
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
