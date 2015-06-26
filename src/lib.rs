//! Shiplift is a multi-transport utility for maneuvering [docker](https://www.docker.com/) containers
//!
//! # examples
//!
//! ```
//! extern crate shiplift;
//!
//! let mut docker = shiplift::Docker::new();
//! let mut images = docker.images().list().unwrap();
//! println!("docker images in stock");
//! for i in images {
//!   println!("{:?}", i.RepoTags);
//! }
//! ```

extern crate hyper;
extern crate jed;
extern crate openssl;
extern crate rustc_serialize;
extern crate unix_socket;
extern crate url;

pub mod builder;
pub mod rep;
pub mod transport;

use builder::{ ContainerBuilder, ContainerListBuilder, Events };
use hyper::{ Client, Url };
use hyper::method::Method;
use openssl::x509::X509FileType;
use rep::Image as ImageRep;
use rep::{
  Change, ContainerDetails, Exit, History,
  ImageDetails, Info, SearchResult, Stats, Status,
  Top, Version
};
use rustc_serialize::json::{ self, Json };
use std::env::{ self, VarError };
use std::io::{ Read, Result };
use std::iter::IntoIterator;
use std::path::Path;
use transport::{ Body, Transport };
use unix_socket::UnixStream;
use url::{ Host, RelativeSchemeData, SchemeData };

/// Entrypoint interface for communicating with docker daemon
pub struct Docker {
  transport: Box<Transport>
}

/// Interface for accessing and manipulating a named docker image
pub struct Image<'a, 'b> {
  docker: &'a mut Docker,
  name: &'b str
}

impl<'a, 'b> Image<'a, 'b> {
  /// Exports an interface for operations that may be performed against a named image
  pub fn new(docker: &'a mut Docker, name: &'b str) -> Image<'a, 'b> {
    Image { docker: docker, name: name }
  }

  /// Inspects a named image's details
  pub fn inspect(self) -> Result<ImageDetails> {
    let raw = try!(self.docker.get(&format!("/images/{}/json", self.name)[..]));
    Ok(json::decode::<ImageDetails>(&raw).unwrap())
  }

  /// Lists the history of the images set of changes
  pub fn history(self) -> Result<Vec<History>> {
    let raw = try!(self.docker.get(&format!("/images/{}/history", self.name)[..]));
    Ok(json::decode::<Vec<History>>(&raw).unwrap())
  }

  /// Delete's an image
  pub fn delete(self) -> Result<Vec<Status>> {
    let raw = try!(self.docker.delete(&format!("/images/{}", self.name)[..]));
    Ok(match Json::from_str(&raw).unwrap() {
      Json::Array(ref xs) => xs.iter().map(|j| {
        let obj = j.as_object().unwrap();
        obj.get("Untagged").map(|sha| Status::Untagged(sha.as_string().unwrap().to_string()))
           .or(obj.get("Deleted").map(|sha| Status::Deleted(sha.as_string().unwrap().to_string())))
           .unwrap()
      }),
      _ => unreachable!("")
    }.collect())
  }

  /// Export this image to a tarball
  pub fn export(self) -> Result<Box<Read>> {
    self.docker.stream_get(&format!("/images/{}/get", self.name)[..])
  }
}

/// Interface for docker images
pub struct Images<'a> {
  docker: &'a mut Docker
}

impl<'a> Images<'a> {
  /// Exports an interface for interacting with docker images
  pub fn new(docker: &'a mut Docker) -> Images<'a> {
    Images { docker: docker }
  }

  /// Lists the docker images on the current docker host
  pub fn list(self) -> Result<Vec<ImageRep>> {
    let raw = try!(self.docker.get("/images/json"));
    Ok(json::decode::<Vec<ImageRep>>(&raw).unwrap())
  }

  /// Returns a reference to a set of operations available for a named image
  pub fn get(&'a mut self, name: &'a str) -> Image {
    Image::new(self.docker, name)
  }

  /// Search for docker images by term
  pub fn search(self, term: &str) -> Result<Vec<SearchResult>> {
    let raw = try!(self.docker.get(&format!("/images/search?term={}", term)[..]));
    Ok(json::decode::<Vec<SearchResult>>(&raw).unwrap())
  }

  /// Create a new docker images from an existing image
  pub fn create(self, from: &str) -> Result<Box<Read>> {
    self.docker.stream_post(&format!("/images/create?fromImage={}", from)[..])
  }

  /// exports a collection of named images,
  /// either by name, name:tag, or image id, into a tarball
  pub fn export(self, names: Vec<&str>) -> Result<Box<Read>> {
    let query = names.iter()
      .map(|n| format!("names={}", n))
      .collect::<Vec<String>>()
      .connect("&");
    self.docker.stream_get(&format!("/images/get?{}", query)[..])
  }

  //pub fn import(self, tarball: Box<Read>) -> Result<()> {
  //  self.docker.post
  //}
}

/// Interface for accessing and manipulating a docker container
pub struct Container<'a, 'b> {
  docker: &'a mut Docker,
  id: &'b str
}

impl<'a, 'b> Container<'a, 'b> {
  /// Exports an interface exposing operations against a container instance
  pub fn new(docker: &'a mut Docker, id: &'b str) -> Container<'a, 'b> {
    Container { docker: docker, id: id }
  }

  /// Inspects the current docker container instance's details
  pub fn inspect(self) -> Result<ContainerDetails> {
    let raw = try!(self.docker.get(&format!("/containers/{}/json", self.id)[..]));
    Ok(json::decode::<ContainerDetails>(&raw).unwrap())
  }

  /// Returns a `top` view of information about the container process
  pub fn top(self) -> Result<Top> {
    let raw = try!(self.docker.get(&format!("/containers/{}/top", self.id)[..]));
    Ok(json::decode::<Top>(&raw).unwrap())
  }

  /// Returns a stream of logs emitted but the container instance
  pub fn logs(self) -> Result<Box<Read>> {
    let query = format!(
      "follow={}&stdout={}&stderr={}&timestamps={}&tail={}",
      true, true, true, true, "all");
    self.docker.stream_get(&format!("/containers/{}/logs?{}", self.id, query)[..])
  }

  /// Returns a set of changes made to the container instance
  pub fn changes(self) -> Result<Vec<Change>> {
    let raw = try!(self.docker.get(&format!("/containers/{}/changes", self.id)[..]));
    Ok(json::decode::<Vec<Change>>(&raw).unwrap())
  }

  /// Exports the current docker container into a tarball
  pub fn export(self) -> Result<Box<Read>> {
    self.docker.stream_get(&format!("/containers/{}/export", self.id)[..])
  }

  /// Returns a stream of stats specific to this container instance
  pub fn stats(self) -> Result<Box<Iterator<Item=Stats>>> {
    let raw = try!(self.docker.stream_get(&format!("/containers/{}/stats", self.id)[..]));
    let it = jed::Iter::new(raw).into_iter().map(|j| {
      let s = json::encode(&j).unwrap();
      json::decode::<Stats>(&s).unwrap()
    });
    Ok(Box::new(it))
  }

  /// Start the container instance
  pub fn start(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/start", self.id)[..], None).map(|_| ())
  }

  /// Stop the container instance
  pub fn stop(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/stop", self.id)[..], None).map(|_| ())
  }

  /// Restart the container instance
  pub fn restart(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/restart", self.id)[..], None).map(|_| ())
  }

  /// Kill the container instance
  pub fn kill(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/kill", self.id)[..], None).map(|_| ())
  }

  /// Rename the container instance
  pub fn rename(self, name: &str) -> Result<()> {
    self.docker.post(&format!("/containers/{}/rename?name={}", self.id, name)[..], None).map(|_| ())
  }

  /// Pause the container instance
  pub fn pause(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/pause", self.id)[..], None).map(|_| ())
  }

  /// Unpause the container instance
  pub fn unpause(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/unpause", self.id)[..], None).map(|_| ())
  }

  /// Wait until the container stops
  pub fn wait(self) -> Result<Exit> {
    let raw = try!(self.docker.post(&format!("/containers/{}/wait", self.id)[..], None));
    Ok(json::decode::<Exit>(&raw).unwrap())
  }

  /// Delete the container instance
  pub fn delete(self) -> Result<()> {
    self.docker.delete(&format!("/containers/{}", self.id)[..]).map(|_| ())
  }

  // todo attach, attach/ws,
}

/// Interface for docker containers
pub struct Containers<'a> {
  docker: &'a mut Docker
}

impl<'a> Containers<'a> {
  /// Exports an interface for interacting with docker containers
  pub fn new(docker: &'a mut Docker) -> Containers<'a> {
    Containers { docker: docker }
  }

  /// Lists the container instances on the docker host
  pub fn list(self) -> ContainerListBuilder<'a> {
    ContainerListBuilder::new(self.docker)
  }

  /// Returns a reference to a set of operations available to a specific container instance
  pub fn get(&'a mut self, name: &'a str) -> Container {
    Container::new(self.docker, name)
  }

  /// Returns a builder interface for creating a new container instance
  pub fn create(&'a mut self, image: &'a str) -> ContainerBuilder {
    ContainerBuilder::new(self.docker, image)
  }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
  /// constructs a new Docker instance for a docker host listening at a url specified by an env var `DOCKER_HOST`,
  /// falling back on unix:///var/run/docker.sock
  pub fn new() -> Docker {
    let fallback: std::result::Result<String, VarError> =
      Ok("unix:///var/run/docker.sock".to_string());
    let host = env::var("DOCKER_HOST")
        .or(fallback)
        .map(|h| Url::parse(&h).ok()
             .expect("invalid url"))
        .ok()
        .expect("expected host");
    Docker::host(host)
  }

  /// constructs a new Docker instance for docker host listening at the given host url
  pub fn host(host: Url) -> Docker {
    let domain = match host.scheme_data {
        SchemeData::NonRelative(s) => s,
        SchemeData::Relative(RelativeSchemeData { host: host, .. }) =>
          match host {
              Host::Domain(s) => s,
              Host::Ipv6(a)   => a.to_string()
          }
    };
    match &host.scheme[..] {
      "unix" => {
        let stream =
          match UnixStream::connect(domain) {
            Err(_) => panic!("failed to connect to socket"),
            Ok(s) => s
          };
        Docker { transport: Box::new(stream) }
      },
          _  => {
        let mut client = Client::new();
        client.set_ssl_verifier(Box::new(|ssl_ctx| {
          match env::var("DOCKER_CERT_PATH").ok() {
            Some(ref certs) => {
              let cert = &format!("{}/cert.pem", certs);
              let key = &format!("{}/key.pem", certs);
              ssl_ctx.set_certificate_file(&Path::new(cert), X509FileType::PEM);
              ssl_ctx.set_private_key_file(&Path::new(key), X509FileType::PEM);
              match env::var("DOCKER_TLS_VERIFY").ok() {
                Some(_) => {
                  let ca = &format!("{}/ca.pem", certs);
                  ssl_ctx.set_CA_file(&Path::new(ca));
                }, _ => ()
              };
              ()
            },  _ => ()
          }
        }));
        let tup = (client, format!("https:{}", domain.to_string()));
        Docker { transport: Box::new(tup) }
      }
    }
  }

  /// Exports an interface for interacting with docker images
  pub fn images<'a>(&'a mut self) -> Images {
    Images::new(self)
  }

  /// Exports an interface for interacting with docker containers
  pub fn containers<'a>(&'a mut self) -> Containers {
    Containers::new(self)
  }

  /// Returns version information associated with the docker daemon
  pub fn version(&mut self) -> Result<Version> {
    let raw = try!(self.get("/version"));
    Ok(json::decode::<Version>(&raw).unwrap())
  }

  /// Returns information associated with the docker daemon
  pub fn info(&mut self) -> Result<Info> {
    let raw = try!(self.get("/info"));
    Ok(json::decode::<Info>(&raw).unwrap())
  }

  /// Returns a simple ping response indicating the docker daemon is accessible
  pub fn ping(&mut self) -> Result<String> {
    self.get("/_ping")
  }

  /// Retruns a stream of events ocurring on the current docker host
  pub fn events(&mut self) -> Events {
    Events::new(self)
  }

  fn get(&mut self, endpoint: &str) -> Result<String> {
    (*self.transport).request(Method::Get, endpoint, None)
  }

  fn post(&mut self, endpoint: &str, body: Option<Body>) -> Result<String> {
    (*self.transport).request(Method::Post, endpoint, body)
  }

  fn delete(&mut self, endpoint: &str) -> Result<String> {
    (*self.transport).request(Method::Delete, endpoint, None)
  }

  fn stream_post(&mut self, endpoint: &str) -> Result<Box<Read>> {
    (*self.transport).stream(Method::Post, endpoint, None)
  }

  fn stream_get(&mut self, endpoint: &str) -> Result<Box<Read>> {
    (*self.transport).stream(Method::Get, endpoint, None)
  }
}
