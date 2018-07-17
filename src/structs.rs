extern crate byteorder;
extern crate flate2;
extern crate hyper;
extern crate hyperlocal;
extern crate rustc_serialize;
extern crate tar;
extern crate url;
extern crate serde;
extern crate serde_json;


#[cfg(feature = "ssl")]
use hyper::net::HttpsConnector;
#[cfg(feature = "ssl")]
use hyper_openssl::OpensslClient;
#[cfg(feature = "ssl")]
use openssl::ssl::{SslConnectorBuilder, SslMethod};
#[cfg(feature = "ssl")]
use openssl::x509::X509_FILETYPE_PEM;
#[cfg(feature = "ssl")]
use std::path::Path;

use reader::BufIterator;

pub use builder::*;

pub use errors::Error;
use errors::ErrorKind as EK;
/// Represents the result of all docker operations
pub use errors::Result;

use hyper::client::Body;
use hyper::header::ContentType;
use hyper::method::Method;
use hyper::{Client, Url};
use hyperlocal::UnixSocketConnector;
use serde::de::DeserializeOwned;
use serde_json::Value;

use tarball;

use rep::{
    Change, Container as ContainerRep, ContainerCreateInfo, ContainerDetails, Event, Exit, History,
    Image as ImageRep, ImageDetails, Info, NetworkCreateInfo, NetworkDetails as NetworkInfo,
    SearchResult, Stats, Status, Top, Version,
};

use std::borrow::Cow;
use std::env;
use std::io::Read;
use std::time::Duration;
use transport::{tar, Transport};
use tty::Tty;
use url::form_urlencoded;
use builder::ContainerArchiveOptions;

/// Entrypoint interface for communicating with docker daemon
pub struct Docker {
    transport: Transport,
}

/// Interface for accessing and manipulating a named docker image
pub struct Image<'a, 'b> {
    docker: &'a Docker,
    name: Cow<'b, str>,
}

impl<'a, 'b> Image<'a, 'b> {
    /// Exports an interface for operations that may be performed against a named image
    pub fn new<S>(docker: &'a Docker, name: S) -> Image<'a, 'b>
    where
        S: Into<Cow<'b, str>>,
    {
        Image {
            docker,
            name: name.into(),
        }
    }

    /// Inspects a named image's details
    pub fn inspect(&self) -> Result<ImageDetails> {
        let raw = self.docker.get(&format!("/images/{}/json", self.name)[..])?;
        ::serde_json::from_str::<ImageDetails>(&raw).map_err(Error::from)
    }

    /// Lists the history of the images set of changes
    pub fn history(&self) -> Result<Vec<History>> {
        let raw = self
            .docker
            .get(&format!("/images/{}/history", self.name)[..])?;
        ::serde_json::from_str::<Vec<History>>(&raw).map_err(Error::from)
    }

    /// Deletes an image
    pub fn delete(&self) -> Result<Vec<Status>> {
        let raw = self.docker.delete(&format!("/images/{}", self.name)[..])?;
        match ::serde_json::from_str(&raw)? {
            Value::Array(ref xs) => xs
                .iter()
                .map(|j| {
                    let obj = j
                        .as_object()
                        .ok_or_else(|| EK::JsonTypeError("<anonym>", "Object"))?;

                    if let Some(sha) = obj.get("Untagged") {
                        sha.as_str()
                            .map(|s| Status::Untagged(s.to_owned()))
                            .ok_or_else(|| EK::JsonTypeError("Untagged", "String"))
                    } else {
                        obj.get("Deleted")
                            .ok_or_else(|| EK::JsonFieldMissing("Deleted' or 'Untagged"))
                            .and_then(|sha| {
                                sha.as_str()
                                    .map(|s| Status::Deleted(s.to_owned()))
                                    .ok_or_else(|| EK::JsonTypeError("Deleted", "String"))
                            })
                    }
                })
                .map(|r| r.map_err(Error::from_kind)),

            _ => unreachable!(),
        }.collect()
    }

    /// Export this image to a tarball
    pub fn export(&self) -> Result<Box<Read>> {
        self.docker
            .stream_get(&format!("/images/{}/get", self.name)[..])
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
    pub fn build(&self, opts: &BuildOptions) -> Result<Vec<Value>> {
        let mut path = vec!["/build".to_owned()];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        let mut bytes = vec![];

        tarball::dir(&mut bytes, &opts.path[..])?;

        let body = Body::BufBody(&bytes[..], bytes.len());

        self.docker
            .stream_post(&path.join("?"), Some((body, tar())))
            .and_then(|r| ::serde_json::from_reader::<_, Vec<_>>(r).map_err(Error::from))
    }

    /// Lists the docker images on the current docker host
    pub fn list(&self, opts: &ImageListOptions) -> Result<Vec<ImageRep>> {
        let mut path = vec!["/images/json".to_owned()];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        let raw = self.docker.get(&path.join("?"))?;
        ::serde_json::from_str::<Vec<ImageRep>>(&raw).map_err(Error::from)
    }

    /// Returns a reference to a set of operations available for a named image
    pub fn get(&'a self, name: &'a str) -> Image {
        Image::new(self.docker, name)
    }

    /// Search for docker images by term
    pub fn search(&self, term: &str) -> Result<Vec<SearchResult>> {
        let query = form_urlencoded::serialize(vec![("term", term)]);
        let raw = self.docker.get(&format!("/images/search?{}", query)[..])?;

        ::serde_json::from_str::<Vec<SearchResult>>(&raw).map_err(Error::from)
    }

    /// Pull and create a new docker images from an existing image
    pub fn pull(&self, opts: &PullOptions) -> Result<BufIterator<Value>> {
        let mut path = vec!["/images/create".to_owned()];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        self.docker
            .bufreader_post(&path.join("?"), None as Option<(&'a str, ContentType)>)
    }

    /// exports a collection of named images,
    /// either by name, name:tag, or image id, into a tarball
    pub fn export(&self, names: Vec<&str>) -> Result<Box<Read>> {
        let params = names
            .iter()
            .map(|n| ("names", *n))
            .collect::<Vec<(&str, &str)>>();

        let query = form_urlencoded::serialize(params);

        self.docker
            .stream_get(&format!("/images/get?{}", query)[..])
    }

    // pub fn import(self, tarball: Box<Read>) -> Result<()> {
    //  self.docker.post
    // }
}

/// Interface for accessing and manipulating a docker container
pub struct Container<'a, 'b> {
    docker: &'a Docker,
    id: Cow<'b, str>,
}

impl<'a, 'b> Container<'a, 'b> {
    /// Exports an interface exposing operations against a container instance
    pub fn new<S>(docker: &'a Docker, id: S) -> Container<'a, 'b>
    where
        S: Into<Cow<'b, str>>,
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
    pub fn inspect(&self) -> Result<ContainerDetails> {
        let raw = self
            .docker
            .get(&format!("/containers/{}/json", self.id)[..])?;
        ::serde_json::from_str::<ContainerDetails>(&raw).map_err(Error::from)
    }

    /// Returns a `top` view of information about the container process
    pub fn top(&self, psargs: Option<&str>) -> Result<Top> {
        let mut path = vec![format!("/containers/{}/top", self.id)];

        if let Some(ref args) = psargs {
            let encoded = form_urlencoded::serialize(vec![("ps_args", args)]);
            path.push(encoded);
        }

        let raw = self.docker.get(&path.join("?"))?;

        ::serde_json::from_str::<Top>(&raw).map_err(Error::from)
    }

    /// Returns a stream of logs emitted but the container instance
    pub fn logs(&self, opts: &LogsOptions) -> Result<Box<Read>> {
        let mut path = vec![format!("/containers/{}/logs", self.id)];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        self.docker.stream_get(&path.join("?"))
    }

    /// Returns a set of changes made to the container instance
    pub fn changes(&self) -> Result<Vec<Change>> {
        let raw = self
            .docker
            .get(&format!("/containers/{}/changes", self.id)[..])?;

        ::serde_json::from_str::<Vec<Change>>(&raw).map_err(Error::from)
    }

    /// Exports the current docker container into a tarball
    pub fn export(&self) -> Result<Box<Read>> {
        self.docker
            .stream_get(&format!("/containers/{}/export", self.id)[..])
    }

    /// Returns a stream of stats specific to this container instance
    pub fn stats(&self) -> Result<BufIterator<Option<Stats>>> {
        self.docker
            .bufreader_get(&format!("/containers/{}/stats", self.id)[..])
    }

    /// Start the container instance
    pub fn start(&'a self) -> Result<()> {
        let s = &format!("/containers/{}/start", self.id)[..];
        self.docker
            .post(s, None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Stop the container instance
    pub fn stop(&self, wait: Option<Duration>) -> Result<()> {
        let mut path = vec![format!("/containers/{}/stop", self.id)];

        if let Some(w) = wait {
            let encoded = form_urlencoded::serialize(vec![("t", w.as_secs().to_string())]);
            path.push(encoded);
        }

        self.docker
            .post(&path.join("?"), None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Restart the container instance
    pub fn restart(&self, wait: Option<Duration>) -> Result<()> {
        let mut path = vec![format!("/containers/{}/restart", self.id)];

        if let Some(w) = wait {
            let encoded = form_urlencoded::serialize(vec![("t", w.as_secs().to_string())]);

            path.push(encoded);
        }

        self.docker
            .post(&path.join("?"), None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Kill the container instance
    pub fn kill(&self, signal: Option<&str>) -> Result<()> {
        let mut path = vec![format!("/containers/{}/kill", self.id)];

        if let Some(sig) = signal {
            let encoded = form_urlencoded::serialize(vec![("signal", sig.to_owned())]);
            path.push(encoded)
        }

        self.docker
            .post(&path.join("?"), None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Rename the container instance
    pub fn rename(&self, name: &str) -> Result<()> {
        let query = form_urlencoded::serialize(vec![("name", name)]);
        let s = &format!("/containers/{}/rename?{}", self.id, query)[..];

        self.docker
            .post(s, None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Pause the container instance
    pub fn pause(&self) -> Result<()> {
        let s = &format!("/containers/{}/pause", self.id)[..];

        self.docker
            .post(s, None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Unpause the container instance
    pub fn unpause(&self) -> Result<()> {
        let s = &format!("/containers/{}/unpause", self.id)[..];

        self.docker
            .post(s, None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Wait until the container stops
    pub fn wait(&self) -> Result<Exit> {
        let s = &format!("/containers/{}/wait", self.id)[..];
        let raw = self.docker.post(s, None as Option<(&'a str, ContentType)>)?;

        ::serde_json::from_str::<Exit>(&raw).map_err(Error::from)
    }

    /// Delete the container instance
    ///
    /// Use remove instead to use the force/v options.
    pub fn delete(&self) -> Result<()> {
        self.docker
            .delete(&format!("/containers/{}", self.id)[..])
            .map(|_| ())
    }

    /// Delete the container instance (todo: force/v)
    pub fn remove(&self, opts: RmContainerOptions) -> Result<()> {
        let mut path = vec![format!("/containers/{}", self.id)];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        self.docker.delete(&path.join("?"))?;
        Ok(())
    }

    /// Exec the specified command in the container
    pub fn exec(&self, opts: &ExecContainerOptions) -> Result<Tty> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();

        let s = &format!("/containers/{}/exec", self.id)[..];

        match self.docker.post(s, Some((&mut bytes, ContentType::json()))) {
            Err(e) => Err(e),
            Ok(res) => {
                let data = "{}";
                let mut bytes = data.as_bytes();
                let json = ::serde_json::from_str::<Value>(res.as_str())?;

                if let Value::Object(ref _obj) = json {
                    let id = json
                        .get("Id")
                        .ok_or_else(|| EK::JsonFieldMissing("Id"))
                        .map_err(Error::from_kind)?
                        .as_str()
                        .ok_or_else(|| EK::JsonTypeError("Id", "String"))
                        .map_err(Error::from_kind)?;

                    let post = &format!("/exec/{}/start", id);

                    self.docker
                        .stream_post(&post[..], Some((&mut bytes, ContentType::json())))
                        .map(|stream| Tty::new(stream))
                } else {
                    Err(Error::from_kind(EK::JsonTypeError("<anonymous>", "Object")))
                }
            }
        }
    }

    pub fn archive_put(&self, opts: &ContainerArchiveOptions) -> Result<()> {
        let mut path = vec![(&format!("/containers/{}/archive", self.id)).to_owned()];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        let mut bytes = vec![];

        tarball::dir(&mut bytes, &opts.local_path)?;

        let body = Body::BufBody(&bytes[..], bytes.len());

        self.docker
            .stream_put(&path.join("?"), Some((body, tar())))
            .map(|_| ())
    }

    // todo attach, attach/ws, copy, archive
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
    pub fn list(&self, opts: &ContainerListOptions) -> Result<Vec<ContainerRep>> {
        let mut path = vec!["/containers/json".to_owned()];

        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        let raw = self.docker.get(&path.join("?"))?;
        ::serde_json::from_str::<Vec<ContainerRep>>(&raw).map_err(Error::from)
    }

    /// Returns a reference to a set of operations available to a specific container instance
    pub fn get(&'a self, name: &'a str) -> Container {
        Container::new(self.docker, name)
    }

    /// Returns a builder interface for creating a new container instance
    pub fn create(&'a self, opts: &ContainerOptions) -> Result<ContainerCreateInfo> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();
        let mut path = vec!["/containers/create".to_owned()];

        if let Some(ref name) = opts.name {
            path.push(form_urlencoded::serialize(vec![("name", name)]));
        }

        let raw = self
            .docker
            .post(&path.join("?"), Some((&mut bytes, ContentType::json())))?;

        ::serde_json::from_str::<ContainerCreateInfo>(&raw).map_err(Error::from)
    }
}

/// Interface for docker network
pub struct Networks<'a> {
    docker: &'a Docker,
}

impl<'a> Networks<'a> {
    /// Exports an interface for interacting with docker Networks
    pub fn new(docker: &'a Docker) -> Networks<'a> {
        Networks { docker }
    }

    /// List the docker networks on the current docker host
    pub fn list(&self, opts: &NetworkListOptions) -> Result<Vec<NetworkInfo>> {
        let mut path = vec!["/networks".to_owned()];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        let raw = self.docker.get(&path.join("?"))?;
        ::serde_json::from_str::<Vec<NetworkInfo>>(&raw).map_err(Error::from)
    }

    /// Returns a reference to a set of operations available to a specific network instance
    pub fn get(&'a self, id: &'a str) -> Network {
        Network::new(self.docker, id)
    }

    pub fn create(&'a self, opts: &NetworkCreateOptions) -> Result<NetworkCreateInfo> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();
        let path = vec!["/networks/create".to_owned()];

        let raw = self
            .docker
            .post(&path.join("?"), Some((&mut bytes, ContentType::json())))?;

        ::serde_json::from_str::<NetworkCreateInfo>(&raw).map_err(Error::from)
    }
}

/// Interface for accessing and manipulating a docker network
pub struct Network<'a, 'b> {
    docker: &'a Docker,
    id: Cow<'b, str>,
}

impl<'a, 'b> Network<'a, 'b> {
    /// Exports an interface exposing operations against a network instance
    pub fn new<S>(docker: &'a Docker, id: S) -> Network<'a, 'b>
    where
        S: Into<Cow<'b, str>>,
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
    pub fn inspect(&self) -> Result<NetworkInfo> {
        let raw = self.docker.get(&format!("/networks/{}", self.id)[..])?;
        ::serde_json::from_str::<NetworkInfo>(&raw).map_err(Error::from)
    }

    /// Delete the network instance
    pub fn delete(&self) -> Result<()> {
        self.docker
            .delete(&format!("/networks/{}", self.id)[..])
            .map(|_| ())
    }

    /// Connect container to network
    pub fn connect(&self, opts: &ContainerConnectionOptions) -> Result<()> {
        self.do_connection("connect", opts)
    }

    /// Disconnect container to network
    pub fn disconnect(&self, opts: &ContainerConnectionOptions) -> Result<()> {
        self.do_connection("disconnect", opts)
    }

    fn do_connection(&self, segment: &str, opts: &ContainerConnectionOptions) -> Result<()> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();

        let s = &format!("/networks/{}/{}", self.id, segment)[..];

        self.docker
            .post(s, Some((&mut bytes, ContentType::json())))
            .map(|_| ())
    }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
    /// constructs a new Docker instance for a docker host listening at a url specified by an env var `DOCKER_HOST`,
    /// falling back on unix:///var/run/docker.sock
    pub fn new(host: Option<String>) -> Result<Docker> {
        host
            .ok_or(env::var("DOCKER_HOST"))
            .or_else(|_| Ok("unix:///var/run/docker.sock".to_owned()))
            .and_then(|h| Url::parse(&h).map_err(Error::from))
            .and_then(Docker::host)
    }

    /// constructs a new Docker instance for docker host listening at the given host url
    pub fn host(host: Url) -> Result<Docker> {
        match host.scheme().as_ref() {
            "unix" => Ok(Docker {
                transport: Transport::Unix {
                    client: Client::with_connector(UnixSocketConnector),
                    path: host.path().to_owned(),
                },
            }),

            _ => {
                #[cfg(not(feature = "ssl"))]
                let client = Client::new();

                #[cfg(feature = "ssl")]
                let client = if let Some(ref certs) = env::var("DOCKER_CERT_PATH").ok() {
                    // https://github.com/hyperium/hyper/blob/master/src/net.rs#L427-L428
                    let mut connector = SslConnectorBuilder::new(SslMethod::tls())?;

                    connector.set_cipher_list("DEFAULT")?;

                    let cert = &format!("{}/cert.pem", certs);
                    let key = &format!("{}/key.pem", certs);

                    connector.set_certificate_file(&Path::new(cert), X509_FILETYPE_PEM)?;

                    connector.set_private_key_file(&Path::new(key), X509_FILETYPE_PEM)?;

                    if let Some(_) = env::var("DOCKER_TLS_VERIFY").ok() {
                        let ca = &format!("{}/ca.pem", certs);
                        connector.set_ca_file(&Path::new(ca))?;
                    }

                    let ssl = OpensslClient::from(connector.build());
                    Client::with_connector(HttpsConnector::new(ssl))
                } else {
                    Client::new()
                };

                let hoststr = host
                    .host_str()
                    .ok_or_else(|| EK::NoHostString)
                    .map_err(Error::from_kind)?
                    .to_owned();

                let port = host
                    .port_or_known_default()
                    .ok_or_else(|| EK::NoPort)
                    .map_err(Error::from_kind)?
                    .to_owned();

                let host = format!("{}://{}:{}", host.scheme(), hoststr, port);

                let d = Docker {
                    transport: Transport::Tcp { client, host },
                };

                Ok(d)
            }
        }
    }

    /// Exports an interface for interacting with docker images
    pub fn images<'a>(&'a self) -> Images {
        Images::new(self)
    }

    /// Exports an interface for interacting with docker containers
    pub fn containers<'a>(&'a self) -> Containers {
        Containers::new(self)
    }

    pub fn networks<'a>(&'a self) -> Networks {
        Networks::new(self)
    }

    /// Returns version information associated with the docker daemon
    pub fn version(&self) -> Result<Version> {
        let raw = self.get("/version")?;
        ::serde_json::from_str::<Version>(&raw).map_err(Error::from)
    }

    /// Returns information associated with the docker daemon
    pub fn info(&self) -> Result<Info> {
        let raw = self.get("/info")?;
        ::serde_json::from_str::<Info>(&raw).map_err(Error::from)
    }

    /// Returns a simple ping response indicating the docker daemon is accessible
    pub fn ping(&self) -> Result<String> {
        self.get("/_ping")
    }

    /// Returns an interator over streamed docker events
    pub fn events(&self, opts: &EventsOptions) -> Result<BufIterator<Event>> {
        let mut path = vec!["/events".to_owned()];

        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        self.bufreader_get(&path.join("?")[..])
    }

    fn get<'a>(&self, endpoint: &str) -> Result<String> {
        self.transport.request(
            Method::Get,
            endpoint,
            None as Option<(&'a str, ContentType)>,
        )
    }

    fn post<'a, B>(&'a self, endpoint: &str, body: Option<(B, ContentType)>) -> Result<String>
    where
        B: Into<Body<'a>>,
    {
        self.transport.request(Method::Post, endpoint, body)
    }

    fn delete<'a>(&self, endpoint: &str) -> Result<String> {
        self.transport.request(
            Method::Delete,
            endpoint,
            None as Option<(&'a str, ContentType)>,
        )
    }

    fn stream_post<'a, B>(
        &'a self,
        endpoint: &str,
        body: Option<(B, ContentType)>,
    ) -> Result<Box<Read>>
    where
        B: Into<Body<'a>>,
    {
        self.transport.stream(Method::Post, endpoint, body)
    }

    fn stream_put<'a, B>(
        &'a self,
        endpoint: &str,
        body: Option<(B, ContentType)>,
    ) -> Result<Box<Read>>
        where
            B: Into<Body<'a>>,
    {
        self.transport.stream(Method::Put, endpoint, body)
    }

    fn bufreader_post<'a, B, T>(
        &'a self,
        endpoint: &str,
        body: Option<(B, ContentType)>,
    ) -> Result<BufIterator<T>>
    where
        B: Into<Body<'a>>,
        T: DeserializeOwned,
    {
        self.transport.bufreader(Method::Post, endpoint, body)
    }

    fn stream_get<'a>(&self, endpoint: &str) -> Result<Box<Read>> {
        self.transport.stream(
            Method::Get,
            endpoint,
            None as Option<(&'a str, ContentType)>,
        )
    }

    fn bufreader_get<'a, T>(&self, endpoint: &str) -> Result<BufIterator<T>>
    where
        T: DeserializeOwned,
    {
        self.transport.bufreader(
            Method::Get,
            endpoint,
            None as Option<(&'a str, ContentType)>,
        )
    }
}
