extern crate hyper;
extern crate openssl;
extern crate unix_socket;
extern crate url;

use hyper::{ Client, Url };
use hyper::method::Method;
use hyper::net::NetworkConnector;
use openssl::x509::X509FileType;
use std::io::{ Read, Write };
use std::io;
use std::{ env, result };
use std::path::Path;
use std::io::Error;
use unix_socket::UnixStream;
use url::{ Host, RelativeSchemeData, SchemeData };

pub type Result<T> = result::Result<T, Error>;

trait Transport {
  fn request(&mut self, method: Method, endpoint: &str) -> Result<String>;
  fn stream(&mut self, method: Method, endpoint: &str) -> Result<Box<Read>>;
}

pub struct Docker {
  transport: Box<Transport>
}

impl Transport for UnixStream {
  fn request(&mut self, method: Method, endpoint: &str) -> Result<String> {
      let method_str = match method {
       Method::Put    => "PUT",
       Method::Post   => "POST",
       Method::Delete => "DELETE",
                _     => "GET"
     };
     let req = format!("{} {} HTTP/1.0\r\n\r\n", method_str, endpoint);
     try!(self.write_all(req.as_bytes()));
     let mut result = String::new();
     self.read_to_string(&mut result).map(|_| result)
  }

  fn stream(&mut self, method: Method, endpoint: &str) -> Result<Box<Read>> {
    Err(io::Error::new(io::ErrorKind::InvalidInput, "Not yet implemented", None))
  }
}

impl<C: NetworkConnector> Transport for (Client<C>, String) {
  fn request(&mut self, method: Method, endpoint: &str) -> Result<String> {
    let uri = format!("{}{}", self.1, endpoint);
    let req = match method {
       Method::Put    => self.0.put(&uri[..]),
       Method::Post   => self.0.post(&uri[..]),
       Method::Delete => self.0.delete(&uri[..]),
                    _ => self.0.get(&uri[..])
    };
    let mut res = match req.send() {
      Ok(r) => r,
      Err(e) => panic!("failed request {:?}", e)
    };
    let mut body = String::new();
    res.read_to_string(&mut body).map(|_| body)
  }

  fn stream(&mut self, method: Method, endpoint: &str) -> Result<Box<Read>> {
    let uri = format!("{}{}", self.1, endpoint);
    let req = match method {
       Method::Put    => self.0.put(&uri[..]),
       Method::Post   => self.0.post(&uri[..]),
       Method::Delete => self.0.delete(&uri[..]),
                    _ => self.0.get(&uri[..])
    };
    let mut res = match req.send() {
      Ok(r) => r,
      Err(e) => panic!("failed request {:?}", e)
    };
    Ok(Box::new(res))
  }
}

pub struct Image<'a, 'b> {
  docker: &'a mut Docker,
  name: &'b str
}

impl<'a, 'b> Image<'a, 'b> {
  pub fn new(docker: &'a mut Docker, name: &'b str) -> Image<'a, 'b> {
    Image { docker: docker, name: name }
  }

  pub fn inspect(self) -> Result<String> {
    self.docker.get(&format!("/images/{}/json", self.name)[..])
  }

  pub fn history(self) -> Result<String> {
    self.docker.get(&format!("/images/{}/history", self.name)[..])
  }

  pub fn delete(self) -> Result<String> {
    self.docker.delete(&format!("/images/{}", self.name)[..])
  }
}

pub struct Images<'a> {
  docker: &'a mut Docker
}

impl<'a> Images<'a> {
  pub fn new(docker: &'a mut Docker) -> Images<'a> {
    Images { docker: docker }
  }
  
  pub fn list(self) -> Result<String> {
    self.docker.get("/images/json")
  }

  pub fn get(&'a mut self, name: &'a str) -> Image {
    Image::new(self.docker, name)
  }

  pub fn search(self, term: &str) -> Result<String> {
    self.docker.get(&format!("/images/search?term={}", term)[..])
  }
}

pub struct Container<'a, 'b> {
  docker: &'a mut Docker,
  id: &'b str
}

impl<'a, 'b> Container<'a, 'b> {
  pub fn new(docker: &'a mut Docker, id: &'b str) -> Container<'a, 'b> {
    Container { docker: docker, id: id }
  }

  pub fn inspect(self) -> Result<String> {
    self.docker.get(&format!("/containers/{}/json", self.id)[..])
  }

  pub fn top(self) -> Result<String> {
    self.docker.get(&format!("/containers/{}/top", self.id)[..])
  }

  pub fn logs(self) -> Result<String> {
    self.docker.get(&format!("/containers/{}/logs", self.id)[..])
  }

  pub fn changes(self) -> Result<String> {
    self.docker.get(&format!("/containers/{}/changes", self.id)[..])
  }

  pub fn stats(self) -> Result<String> {
    self.docker.get(&format!("/containers/{}/stats", self.id)[..])
  }

  pub fn start(self) -> Result<String> {
    self.docker.post(&format!("/containers/{}/start", self.id)[..])
  }

  pub fn stop(self) -> Result<String> {
    self.docker.post(&format!("/containers/{}/stop", self.id)[..])
  }

  pub fn restart(self) -> Result<String> {
    self.docker.post(&format!("/containers/{}/restart", self.id)[..])
  }

  pub fn kill(self) -> Result<String> {
    self.docker.post(&format!("/containers/{}/kill", self.id)[..])
  }

  pub fn rename(self, name: &str) -> Result<String> {
    self.docker.post(&format!("/containers/{}/rename?name={}", self.id, name)[..])
  }

  pub fn pause(self) -> Result<String> {
    self.docker.post(&format!("/containers/{}/pause", self.id)[..])
  }

  pub fn unpause(self) -> Result<String> {
    self.docker.post(&format!("/containers/{}/unpause", self.id)[..])
  }

  pub fn wait(self) -> Result<String> {
    self.docker.post(&format!("/containers/{}/wait", self.id)[..])
  }

  pub fn delete(self) -> Result<String> {
    self.docker.delete(&format!("/containers/{}", self.id)[..])
  }
  // todo attach, attach/ws,
}

pub struct Containers<'a> {
  docker: &'a mut Docker
}

impl<'a> Containers<'a> {
  pub fn new(docker: &'a mut Docker) -> Containers<'a> {
    Containers { docker: docker }
  }
  
  pub fn list(self) -> Result<String> {
    self.docker.get("/containers/json")
  }

  pub fn get(&'a mut self, name: &'a str) -> Container {
    Container::new(self.docker, name)
  }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
  pub fn new() -> Docker {
    let host = env::var("DOCKER_HOST")
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

  pub fn version(&mut self) -> Result<String> {
    self.get("/version")
  }

  pub fn info(&mut self) -> Result<String> {
    self.get("/info")
  }

  pub fn ping(&mut self) -> Result<String> {
    self.get("/_ping")
  }

  // todo stream
  pub fn events(&mut self) -> Result<String> {
    self.get("/events")
  }
 
  fn get(&mut self, endpoint: &str) -> Result<String> {
    (*self.transport).request(Method::Get, endpoint)
  }

  fn post(&mut self, endpoint: &str) -> Result<String> {
    (*self.transport).request(Method::Post, endpoint)
  }

  fn delete(&mut self, endpoint: &str) -> Result<String> {
     (*self.transport).request(Method::Delete, endpoint)
   }
}
