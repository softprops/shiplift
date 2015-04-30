extern crate hyper;
extern crate mime;
extern crate unix_socket;

use std::ops::DerefMut;
use hyper::Client;
use hyper::client;
use self::hyper::header::{ ContentType, UserAgent, qitem };
use hyper::method::Method;
use self::mime::{ Attr, Mime, Value };
use self::mime::TopLevel::Application;
use self::mime::SubLevel::Json;
use std::io::{ self, Read, Result, Write };
use unix_socket::UnixStream;

#[doc(hidden)]
pub struct Body<'a> {
 read: &'a mut Box<&'a mut Read>,
 size: u64
}

impl<'a> Body<'a> {
  /// Create a new body instance
  pub fn new(read: &'a mut Box<&'a mut Read>, size: u64) -> Body<'a> {
    Body { read: read, size: size }
  }
}

pub trait Transport {
  fn request(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<String>;
  fn stream(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<Box<Read>>;
}

impl Transport for UnixStream {
  fn request(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<String> {
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

  fn stream(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<Box<Read>> {
    Err(io::Error::new(io::ErrorKind::InvalidInput, "Not yet implemented"))
  }
}

impl Transport for (Client, String) {
  fn request(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<String> {
    let mut res = match self.stream(method, endpoint, body) {
      Ok(r) => r,
      Err(e) => panic!("failed request {:?}", e)
    };
    let mut body = String::new();
    
    res.read_to_string(&mut body).map(|_| body)
  }

  fn stream(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<Box<Read>> {
    let uri = format!("{}{}", self.1, endpoint);
    let mut req = match method {
      Method::Put    => self.0.put(&uri[..]),
      Method::Post   => self.0.post(&uri[..]),
      Method::Delete => self.0.delete(&uri[..]),
                   _ => self.0.get(&uri[..])
    };
    let embodied = match body {
      Some(Body { read: r, size: l }) => {
        let reader: &mut Read = *r.deref_mut();
        let content_type: Mime = Mime(Application, Json, vec![(Attr::Charset, Value::Utf8)]);
        req.header(ContentType(content_type)).body(client::Body::SizedBody(reader, l))
      },
      _ => req
    };
    let mut res = match embodied.send() {
      Ok(r) => r,
      Err(e) => panic!("failed request {:?}", e)
    };
    Ok(Box::new(res))
  }
}
