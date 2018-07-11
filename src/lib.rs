//! Shiplift is a multi-transport utility for maneuvering [docker](https://www.docker.com/) containers
//!
//! # examples
//!
//! ```no_run
//! extern crate shiplift;
//!
//! let docker = shiplift::Docker::new(None).unwrap();
//! let images = docker.images().list(&Default::default()).unwrap();
//! println!("docker images in stock");
//! for i in images {
//!   println!("{:?}", i.RepoTags);
//! }
//! ```

#[macro_use]
extern crate log;
extern crate hyper;
extern crate hyperlocal;
extern crate flate2;
extern crate jed;
extern crate rustc_serialize;
extern crate url;
extern crate tar;
extern crate byteorder;
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[cfg(feature = "ssl")]
extern crate hyper_openssl;
#[cfg(feature = "ssl")]
extern crate openssl;

pub mod builder;
pub mod rep;
pub mod transport;
pub mod errors;
pub mod tty;

mod tarball;

pub use builder::{BuildOptions, ContainerConnectionOptions, ContainerFilter,
                  ContainerListOptions, ContainerOptions, EventsOptions,
                  ExecContainerOptions, ImageFilter, ImageListOptions,
                  LogsOptions, NetworkCreateOptions, NetworkListOptions,
                  PullOptions, RmContainerOptions};
pub use errors::Error;
use hyper::{Client, Url};
use hyper::client::Body;
use hyper::header::ContentType;
use hyper::method::Method;

#[cfg(feature = "ssl")]
use hyper::net::HttpsConnector;
#[cfg(feature = "ssl")]
use hyper_openssl::OpensslClient;
use hyperlocal::UnixSocketConnector;
#[cfg(feature = "ssl")]
use openssl::ssl::{SslConnectorBuilder, SslMethod};
#[cfg(feature = "ssl")]
use openssl::x509::X509_FILETYPE_PEM;
#[cfg(feature = "ssl")]
use std::path::Path;

use rep::{Change, Container as ContainerRep, ContainerCreateInfo,
          ContainerDetails, Event, Exit, History, ImageDetails, Info,
          SearchResult, Stats, Status, Top, Version};
use rep::{NetworkCreateInfo, NetworkDetails as NetworkInfo};
use rep::Image as ImageRep;
use rustc_serialize::json::{self, Json};
use std::borrow::Cow;
use std::env;
use std::io::Read;
use std::iter::IntoIterator;
use std::time::Duration;
use transport::{Transport, tar};
use tty::Tty;
use url::form_urlencoded;


#[cfg(feature = "ssl")]
const CERT_ENV_VARIABLE: &str = "DOCKER_CERT_PATH";

const HOST_ENV_VARIABLE: &str = "DOCKER_HOST";
const DEFAULT_UNIX_SOCKET: &str = "unix:///var/run/docker.sock";

/// Represents the result of all docker operations
pub type Result<T> = std::result::Result<T, Error>;

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
        Ok(json::decode::<ImageDetails>(&raw)?)
    }

    /// Lists the history of the images set of changes
    pub fn history(&self) -> Result<Vec<History>> {
        let raw = self.docker.get(
            &format!("/images/{}/history", self.name)[..],
        )?;
        Ok(json::decode::<Vec<History>>(&raw)?)
    }

    /// Delete's an image
    pub fn delete(&self) -> Result<Vec<Status>> {
        let raw = self.docker.delete(&format!("/images/{}", self.name)[..])?;
        Ok(
            match Json::from_str(&raw)? {
                Json::Array(ref xs) => {
                    xs.iter().map(|j| {
                        let obj = j.as_object().expect("expected json object");
                        obj.get("Untagged")
                            .map(|sha| {
                                Status::Untagged(
                                    sha.as_string()
                                        .expect(
                                            "expected Untagged to be a string",
                                        )
                                        .to_owned(),
                                )
                            })
                            .or(obj.get("Deleted").map(|sha| {
                                Status::Deleted(
                                    sha.as_string()
                                        .expect(
                                            "expected Deleted to be a string",
                                        )
                                        .to_owned(),
                                )
                            }))
                            .expect("expected Untagged or Deleted")
                    })
                }
                _ => unreachable!(),
            }.collect(),
        )
    }

    /// Export this image to a tarball
    pub fn export(&self) -> Result<Box<Read>> {
        self.docker.stream_get(
            &format!("/images/{}/get", self.name)[..],
        )
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
        &self,
        opts: &BuildOptions,
    ) -> Result<Box<Iterator<Item = Json>>> {
        let mut path = vec!["/build".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        let mut bytes = vec![];

        tarball::dir(&mut bytes, &opts.path[..])?;

        let raw = self.docker.stream_post(
            &path.join("?"),
            Some(
                (Body::BufBody(&bytes[..], bytes.len()), tar()),
            ),
        )?;
        let it = jed::Iter::new(raw).into_iter();
        Ok(Box::new(it))
    }

    /// Lists the docker images on the current docker host
    pub fn list(&self, opts: &ImageListOptions) -> Result<Vec<ImageRep>> {
        let mut path = vec!["/images/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let raw = self.docker.get(&path.join("?"))?;
        Ok(json::decode::<Vec<ImageRep>>(&raw)?)
    }

    /// Returns a reference to a set of operations available for a named image
    pub fn get(&'a self, name: &'a str) -> Image {
        Image::new(self.docker, name)
    }

    /// Search for docker images by term
    pub fn search(&self, term: &str) -> Result<Vec<SearchResult>> {
        let query = form_urlencoded::serialize(vec![("term", term)]);
        let raw = self.docker.get(&format!("/images/search?{}", query)[..])?;
        Ok(json::decode::<Vec<SearchResult>>(&raw)?)
    }

    /// Pull and create a new docker images from an existing image
    pub fn pull(
        &self,
        opts: &PullOptions,
    ) -> Result<Box<Iterator<Item = Json>>> {
        let mut path = vec!["/images/create".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let raw = self.docker.stream_post(
            &path.join("?"),
            None as Option<(&'a str, ContentType)>,
        )?;
        let it = jed::Iter::new(raw).into_iter();
        Ok(Box::new(it))
    }

    /// exports a collection of named images,
    /// either by name, name:tag, or image id, into a tarball
    pub fn export(&self, names: Vec<&str>) -> Result<Box<Read>> {
        let params = names
            .iter()
            .map(|n| ("names", *n))
            .collect::<Vec<(&str, &str)>>();
        let query = form_urlencoded::serialize(params);
        self.docker.stream_get(
            &format!("/images/get?{}", query)[..],
        )
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
        let raw = self.docker.get(
            &format!("/containers/{}/json", self.id)[..],
        )?;
        Ok(json::decode::<ContainerDetails>(&raw)?)
    }

    /// Returns a `top` view of information about the container process
    pub fn top(&self, psargs: Option<&str>) -> Result<Top> {
        let mut path = vec![format!("/containers/{}/top", self.id)];
        if let Some(ref args) = psargs {
            let encoded = form_urlencoded::serialize(vec![("ps_args", args)]);
            path.push(encoded)
        }
        let raw = self.docker.get(&path.join("?"))?;

        Ok(json::decode::<Top>(&raw)?)
    }

    /// Returns a stream of logs emitted but the container instance
    pub fn logs(&self, opts: &LogsOptions) -> Result<Box<Read>> {
        let mut path = vec![format!("/containers/{}/logs", self.id)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker.stream_get(&path.join("?"))
    }

    /// Returns a set of changes made to the container instance
    pub fn changes(&self) -> Result<Vec<Change>> {
        let raw = self.docker.get(
            &format!("/containers/{}/changes", self.id)[..],
        )?;
        Ok(json::decode::<Vec<Change>>(&raw)?)
    }

    /// Exports the current docker container into a tarball
    pub fn export(&self) -> Result<Box<Read>> {
        self.docker.stream_get(
            &format!("/containers/{}/export", self.id)[..],
        )
    }

    /// Returns a stream of stats specific to this container instance
    pub fn stats(&self) -> Result<Box<Iterator<Item = Stats>>> {
        let raw = self.docker.stream_get(
            &format!("/containers/{}/stats", self.id)[..],
        )?;
        let it = jed::Iter::new(raw).into_iter().map(|j| {
            // fixme: better error handling
            debug!("{:?}", j);
            let s = json::encode(&j).unwrap();
            json::decode::<Stats>(&s).unwrap()
        });
        Ok(Box::new(it))
    }

    /// Start the container instance
    pub fn start(&'a self) -> Result<()> {
        self.docker
            .post(
                &format!("/containers/{}/start", self.id)[..],
                None as Option<(&'a str, ContentType)>,
            )
            .map(|_| ())
    }

    /// Stop the container instance
    pub fn stop(&self, wait: Option<Duration>) -> Result<()> {
        let mut path = vec![format!("/containers/{}/stop", self.id)];
        if let Some(w) = wait {
            let encoded = form_urlencoded::serialize(
                vec![("t", w.as_secs().to_string())],
            );
            path.push(encoded)
        }
        self.docker
            .post(&path.join("?"), None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Restart the container instance
    pub fn restart(&self, wait: Option<Duration>) -> Result<()> {
        let mut path = vec![format!("/containers/{}/restart", self.id)];
        if let Some(w) = wait {
            let encoded = form_urlencoded::serialize(
                vec![("t", w.as_secs().to_string())],
            );
            path.push(encoded)
        }
        self.docker
            .post(&path.join("?"), None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Kill the container instance
    pub fn kill(&self, signal: Option<&str>) -> Result<()> {
        let mut path = vec![format!("/containers/{}/kill", self.id)];
        if let Some(sig) = signal {
            let encoded =
                form_urlencoded::serialize(vec![("signal", sig.to_owned())]);
            path.push(encoded)
        }
        self.docker
            .post(&path.join("?"), None as Option<(&'a str, ContentType)>)
            .map(|_| ())
    }

    /// Rename the container instance
    pub fn rename(&self, name: &str) -> Result<()> {
        let query = form_urlencoded::serialize(vec![("name", name)]);
        self.docker
            .post(
                &format!("/containers/{}/rename?{}", self.id, query)[..],
                None as Option<(&'a str, ContentType)>,
            )
            .map(|_| ())
    }

    /// Pause the container instance
    pub fn pause(&self) -> Result<()> {
        self.docker
            .post(
                &format!("/containers/{}/pause", self.id)[..],
                None as Option<(&'a str, ContentType)>,
            )
            .map(|_| ())
    }

    /// Unpause the container instance
    pub fn unpause(&self) -> Result<()> {
        self.docker
            .post(
                &format!("/containers/{}/unpause", self.id)[..],
                None as Option<(&'a str, ContentType)>,
            )
            .map(|_| ())
    }

    /// Wait until the container stops
    pub fn wait(&self) -> Result<Exit> {
        let raw = self.docker.post(
            &format!("/containers/{}/wait", self.id)[..],
            None as Option<(&'a str, ContentType)>,
        )?;
        Ok(json::decode::<Exit>(&raw)?)
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
            path.push(query)
        }
        self.docker.delete(&path.join("?"))?;
        Ok(())
    }

    /// Exec the specified command in the container
    pub fn exec(&self, opts: &ExecContainerOptions) -> Result<Tty> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();
        match self.docker.post(
            &format!("/containers/{}/exec", self.id)[..],
            Some((&mut bytes, ContentType::json())),
        ) {
            Err(e) => Err(e),
            Ok(res) => {
                let data = "{}";
                let mut bytes = data.as_bytes();
                self.docker
                    .stream_post(
                        &format!(
                            "/exec/{}/start",
                            Json::from_str(res.as_str())
                                .unwrap()
                                .search("Id")
                                .unwrap()
                                .as_string()
                                .unwrap()
                        )
                            [..],
                        Some((&mut bytes, ContentType::json())),
                    )
                    .map(|stream| Tty::new(stream))
            }
        }
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
        Containers { docker: docker }
    }

    /// Lists the container instances on the docker host
    pub fn list(
        &self,
        opts: &ContainerListOptions,
    ) -> Result<Vec<ContainerRep>> {
        let mut path = vec!["/containers/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        let raw = self.docker.get(&path.join("?"))?;
        Ok(json::decode::<Vec<ContainerRep>>(&raw)?)
    }

    /// Returns a reference to a set of operations available to a specific container instance
    pub fn get(&'a self, name: &'a str) -> Container {
        Container::new(self.docker, name)
    }

    /// Returns a builder interface for creating a new container instance
    pub fn create(
        &'a self,
        opts: &ContainerOptions,
    ) -> Result<ContainerCreateInfo> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();
        let mut path = vec!["/containers/create".to_owned()];

        if let Some(ref name) = opts.name {
            path.push(form_urlencoded::serialize(vec![("name", name)]));
        }

        let raw = self.docker.post(
            &path.join("?"),
            Some((&mut bytes, ContentType::json())),
        )?;
        Ok(json::decode::<ContainerCreateInfo>(&raw)?)
    }
}

/// Interface for docker network
pub struct Networks<'a> {
    docker: &'a Docker,
}

impl<'a> Networks<'a> {
    /// Exports an interface for interacting with docker Networks
    pub fn new(docker: &'a Docker) -> Networks<'a> {
        Networks { docker: docker }
    }

    /// List the docker networks on the current docker host
    pub fn list(&self, opts: &NetworkListOptions) -> Result<Vec<NetworkInfo>> {
        let mut path = vec!["/networks".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let raw = self.docker.get(&path.join("?"))?;
        Ok(json::decode::<Vec<NetworkInfo>>(&raw)?)
    }

    /// Returns a reference to a set of operations available to a specific network instance
    pub fn get(&'a self, id: &'a str) -> Network {
        Network::new(self.docker, id)
    }

    pub fn create(
        &'a self,
        opts: &NetworkCreateOptions,
    ) -> Result<NetworkCreateInfo> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();
        let path = vec!["/networks/create".to_owned()];

        let raw = self.docker.post(
            &path.join("?"),
            Some((&mut bytes, ContentType::json())),
        )?;
        Ok(json::decode::<NetworkCreateInfo>(&raw)?)
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
            docker: docker,
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
        Ok(json::decode::<NetworkInfo>(&raw)?)
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

    fn do_connection(
        &self,
        segment: &str,
        opts: &ContainerConnectionOptions,
    ) -> Result<()> {
        let data = opts.serialize()?;
        let mut bytes = data.as_bytes();

        self.docker
            .post(
                &format!("/networks/{}/{}", self.id, segment)[..],
                Some((&mut bytes, ContentType::json())),
            )
            .map(|_| ())
    }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
    /// constructs a new Docker instance for a docker host listening at a url specified by an env var `DOCKER_HOST`,
    /// falling back on unix:///var/run/docker.sock
    pub fn new(host: Option<String>) -> Result<Docker> {
        let env = env::var(HOST_ENV_VARIABLE);

        let host = host
            .as_ref()
            .map(String::as_str)
            .unwrap_or(env
                .as_ref()
                .map(String::as_str)
                .unwrap_or(DEFAULT_UNIX_SOCKET)
            );


        let host = Url::parse(host)
                .map_err( |h |
                    Error::Fault {
                        code: hyper::status::StatusCode::BadRequest,
                        message: format!("Couldn't parse invalid host: {:?}\n", &h)
                    }
                )?;

        Docker::host(host)
    }

    /// constructs a new Docker instance for docker host listening at the given host url
    pub fn host(host: Url) -> Result<Docker> {
        match host.scheme() {
            "unix" => {
                Ok(Docker {
                    transport: Transport::Unix {
                        client: Client::with_connector(UnixSocketConnector),
                        path: host.path().to_owned(),
                    },
                })
            }
            _ => {
                #[cfg(not(feature = "ssl"))]
                let client = Client::new();

                #[cfg(feature = "ssl")]
                let client = if let Ok(ref certs) =
                        env::var(CERT_ENV_VARIABLE.to_owned()) {
                    // fixme: don't unwrap before you know what's in the box
                    // https://github.com/hyperium/hyper/blob/master/src/net.rs#L427-L428
                    let mut connector =
                        SslConnectorBuilder::new(SslMethod::tls())?;

                    connector.set_cipher_list("DEFAULT")?;

                    let cert = &format!("{}/cert.pem", certs);
                    let key = &format!("{}/key.pem", certs);

                    connector
                        .set_certificate_file(
                            &Path::new(cert),
                            X509_FILETYPE_PEM,
                        )?;
                    connector
                        .set_private_key_file(
                            &Path::new(key),
                            X509_FILETYPE_PEM,
                        )?;

                    if let Ok(_) = env::var("DOCKER_TLS_VERIFY") {
                        let ca = &format!("{}/ca.pem", certs);
                        connector

                            .set_ca_file(&Path::new(ca))?
                    }

                    let ssl = OpensslClient::from(connector.build());
                    Client::with_connector(HttpsConnector::new(ssl))
                } else {
                    Client::new()
                };

                let hostname = host.host_str().ok_or(
                    Error::Message("Wrong host name\n".to_string()))?;
                let port = host.port_or_known_default().ok_or(
                    Error::Message("Wrong port\n".to_string()))?;

                Ok(Docker {
                    transport: Transport::Tcp {
                        client,
                        host: format!(
                            "{}://{}:{}",
                            host.scheme(),
                            hostname,
                            port
                        ),
                    },
                })
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
        Ok(json::decode::<Version>(&raw)?)
    }

    /// Returns information associated with the docker daemon
    pub fn info(&self) -> Result<Info> {
        let raw = self.get("/info")?;
        Ok(json::decode::<Info>(&raw)?)
    }

    /// Returns a simple ping response indicating the docker daemon is accessible
    pub fn ping(&self) -> Result<String> {
        self.get("/_ping")
    }

    /// Returns an iterator over streamed docker events
    pub fn events(
        &self,
        opts: &EventsOptions,
    ) -> Result<Box<Iterator<Item = Event>>> {
        let mut path = vec!["/events".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let raw = self.stream_get(&path.join("?")[..])?;
        let it = jed::Iter::new(raw).into_iter().map(|j| {
            debug!("{:?}", j);
            // fixme: better error handling
            let s = json::encode(&j).unwrap();
            json::decode::<Event>(&s).unwrap()
        });
        Ok(Box::new(it))
    }

    fn get<'a>(&self, endpoint: &str) -> Result<String> {
        self.transport.request(
            Method::Get,
            endpoint,
            None as Option<(&'a str, ContentType)>,
        )
    }

    fn post<'a, B>(
        &'a self,
        endpoint: &str,
        body: Option<(B, ContentType)>,
    ) -> Result<String>
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

    fn stream_get<'a>(&self, endpoint: &str) -> Result<Box<Read>> {
        self.transport.stream(
            Method::Get,
            endpoint,
            None as Option<(&'a str, ContentType)>,
        )
    }
}
