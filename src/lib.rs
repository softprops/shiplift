//! Shiplift is a multi-transport for utility for maneuvering [docker](https://www.docker.com/) containers
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

use builder::ContainerBuilder;
use hyper::{ Client, Url };
use hyper::method::Method;
use openssl::x509::X509FileType;
use rep::Image as ImageRep;
use rep::Container as ContainerRep;
use rep::{
  Change, ContainerDetails, Event, Exit, History,
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
  pub fn new(docker: &'a mut Docker, name: &'b str) -> Image<'a, 'b> {
    Image { docker: docker, name: name }
  }

  pub fn inspect(self) -> Result<ImageDetails> {
    let raw = try!(self.docker.get(&format!("/images/{}/json", self.name)[..]));
    Ok(json::decode::<ImageDetails>(&raw).unwrap())
  }

  pub fn history(self) -> Result<Vec<History>> {
    let raw = try!(self.docker.get(&format!("/images/{}/history", self.name)[..]));
    Ok(json::decode::<Vec<History>>(&raw).unwrap())
  }

  // todo: rep Untagged, Deleted stream
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
}

/// Interface for docker images
pub struct Images<'a> {
  docker: &'a mut Docker
}

impl<'a> Images<'a> {
  pub fn new(docker: &'a mut Docker) -> Images<'a> {
    Images { docker: docker }
  }
  
  pub fn list(self) -> Result<Vec<ImageRep>> {
    let raw = try!(self.docker.get("/images/json"));
    Ok(json::decode::<Vec<ImageRep>>(&raw).unwrap())
  }

  pub fn get(&'a mut self, name: &'a str) -> Image {
    Image::new(self.docker, name)
  }

  pub fn search(self, term: &str) -> Result<Vec<SearchResult>> {
    let raw = try!(self.docker.get(&format!("/images/search?term={}", term)[..]));
    Ok(json::decode::<Vec<SearchResult>>(&raw).unwrap())
  }

  pub fn create(self, from: &str) -> Result<Box<Read>> {
    self.docker.stream_post(&format!("/images/create?fromImage={}", from)[..])
  }
}

/// Interface for accessing and manipulating a docker container
pub struct Container<'a, 'b> {
  docker: &'a mut Docker,
  id: &'b str
}

impl<'a, 'b> Container<'a, 'b> {
  pub fn new(docker: &'a mut Docker, id: &'b str) -> Container<'a, 'b> {
    Container { docker: docker, id: id }
  }

  pub fn inspect(self) -> Result<ContainerDetails> {
    let raw = try!(self.docker.get(&format!("/containers/{}/json", self.id)[..]));
    Ok(json::decode::<ContainerDetails>(&raw).unwrap())
  }

  pub fn top(self) -> Result<Top> {
    let raw = try!(self.docker.get(&format!("/containers/{}/top", self.id)[..]));
    Ok(json::decode::<Top>(&raw).unwrap())
  }

  pub fn logs(self) -> Result<Box<Read>> {
    let query = format!(
      "follow={}&stdout={}&stderr={}&timestamps={}&tail={}",
      true, true, true, true, "all");
    self.docker.stream_get(&format!("/containers/{}/logs?{}", self.id, query)[..])
  }

  pub fn changes(self) -> Result<Vec<Change>> {
    let raw = try!(self.docker.get(&format!("/containers/{}/changes", self.id)[..]));
    Ok(json::decode::<Vec<Change>>(&raw).unwrap())
  }

  pub fn export(self) -> Result<Box<Read>> {
    self.docker.stream_get(&format!("/containers/{}/export", self.id)[..])
  }

  pub fn stats(self) -> Result<Box<Iterator<Item=Stats>>> {
    let raw = try!(self.docker.stream_get(&format!("/containers/{}/stats", self.id)[..]));
    let it = jed::Iter::new(raw).into_iter().map(|j| {
      let s = json::encode(&j).unwrap();
      json::decode::<Stats>(&s).unwrap()
    });
    Ok(Box::new(it))
  }

  pub fn start(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/start", self.id)[..], None).map(|_| ())
  }

  pub fn stop(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/stop", self.id)[..], None).map(|_| ())
  }

  pub fn restart(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/restart", self.id)[..], None).map(|_| ())
  }

  pub fn kill(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/kill", self.id)[..], None).map(|_| ())
  }

  pub fn rename(self, name: &str) -> Result<()> {
    self.docker.post(&format!("/containers/{}/rename?name={}", self.id, name)[..], None).map(|_| ())
  }

  pub fn pause(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/pause", self.id)[..], None).map(|_| ())
  }

  pub fn unpause(self) -> Result<()> {
    self.docker.post(&format!("/containers/{}/unpause", self.id)[..], None).map(|_| ())
  }

  pub fn wait(self) -> Result<Exit> {
    let raw = try!(self.docker.post(&format!("/containers/{}/wait", self.id)[..], None));
    Ok(json::decode::<Exit>(&raw).unwrap())
  }

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
  pub fn new(docker: &'a mut Docker) -> Containers<'a> {
    Containers { docker: docker }
  }
  
  pub fn list(self) -> Result<Vec<ContainerRep>> {
    let raw = try!(self.docker.get("/containers/json"));
    Ok(json::decode::<Vec<ContainerRep>>(&raw).unwrap())
  }

  pub fn get(&'a mut self, name: &'a str) -> Container {
    Container::new(self.docker, name)
  }

  pub fn create(&'a mut self, image: &'a str) -> ContainerBuilder {
    ContainerBuilder::new(self.docker, image)
  }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
  pub fn new() -> Docker {
    let fallback: std::result::Result<String, VarError> = Ok("unix:///var/run/docker.sock".to_string());
    let host = env::var("DOCKER_HOST")
        .or(fallback)
        .map(|h| Url::parse(&h).ok()
             .expect("invalid url"))
         .ok()
         .expect("expected host");
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

  pub fn images<'a>(&'a mut self) -> Images {
    Images::new(self)
  }

  pub fn containers<'a>(&'a mut self) -> Containers {
    Containers::new(self)
  }

  pub fn version(&mut self) -> Result<Version> {
    let raw = try!(self.get("/version"));
    Ok(json::decode::<Version>(&raw).unwrap())
  }

  pub fn info(&mut self) -> Result<Info> {
    let raw = try!(self.get("/info"));
    Ok(json::decode::<Info>(&raw).unwrap())
  }

  pub fn ping(&mut self) -> Result<String> {
    self.get("/_ping")
  }

  pub fn events(&mut self) -> Result<Box<Iterator<Item=Event>>> {
    let raw = try!(self.stream_get("/events?since=1374067924"));
    let it = jed::Iter::new(raw).into_iter().map(|j| {
     let s = json::encode(&j).unwrap();
     json::decode::<Event>(&s).unwrap()
    });
    Ok(Box::new(it))
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
