//! Transports for communicating with the docker daemon

extern crate hyper;
extern crate mime;
extern crate unix_socket;

use hyper::Client;
use hyper::client;
use self::hyper::buffer::BufReader;
use self::hyper::http::RawStatus;
use self::hyper::http::h1::parse_response;
use self::hyper::http::h1::HttpReader::EofReader;
use self::hyper::header::ContentType;
use self::hyper::status::StatusCode;
use hyper::method::Method;
use self::mime::{ Attr, Mime, Value };
use self::mime::TopLevel::Application;
use self::mime::SubLevel::Json;
use std::ops::DerefMut;
use std::io::{ Error, ErrorKind, Read, Result, Write };
use unix_socket::UnixStream;

fn lift_status_err(status: u16) -> Result<Box<Read>> {
  match status {
    400 => Err(Error::new(ErrorKind::InvalidInput, "bad parameter")),
    404 => Err(Error::new(ErrorKind::InvalidInput, "not found")),
    406 => Err(Error::new(ErrorKind::InvalidInput, "not acceptable")),
    409 => Err(Error::new(ErrorKind::InvalidInput, "conflict found")),
    500 => Err(Error::new(ErrorKind::InvalidInput, "interal server error")),
     _  => unreachable!()
  }
}

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

/// Primary interface for communicating with docker daemon
pub trait Transport {
  fn request(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<String> {
    let mut res = match self.stream(method, endpoint, body) {
      Ok(r) => r,
      Err(e) => panic!("failed request {:?}", e)
    };
    let mut body = String::new();
    res.read_to_string(&mut body).map(|_| body)
  }

  fn stream(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<Box<Read>>;
}

impl Transport for UnixStream {
  fn stream(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<Box<Read>> {
    let method_str = match method {
      Method::Put    => "PUT",
      Method::Post   => "POST",
      Method::Delete => "DELETE",
               _     => "GET"
    };
    let req = format!("{} {} HTTP/1.0\r\n\r\n", method_str, endpoint);
    try!(self.write_all(req.as_bytes()));
    // read the body -- https://github.com/hyperium/hyper/blob/06d072bca1b4af3507af370cbd0ca2ac8f64fc00/src/client/response.rs#L36-L74
    let cloned = try!(self.try_clone());
    let mut stream = BufReader::new(cloned);
    let res = parse_response(&mut stream).unwrap();
    match res.subject {
      RawStatus(200, _) | RawStatus(201, _) | RawStatus(101, _) =>
        Ok(Box::new(EofReader(stream))),
      RawStatus(204, _) =>
        Ok(Box::new(BufReader::new("".as_bytes()))),
      RawStatus(status, _) =>
        lift_status_err(status)
    }
  }
}

impl Transport for (Client, String) {
  fn stream(&mut self, method: Method, endpoint: &str, body: Option<Body>) -> Result<Box<Read>> {
    let uri = format!("{}{}", self.1, endpoint);
    let req = match method {
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
    let res = match embodied.send() {
      Ok(r) => r,
      Err(e) => panic!("failed request {:?}", e)
    };
    match res.status {
      StatusCode::Ok | StatusCode::Created | StatusCode::SwitchingProtocols =>
        Ok(Box::new(res)),
      StatusCode::NoContent =>
        Ok(Box::new(BufReader::new("".as_bytes()))),
      status =>
        lift_status_err(status.to_u16())
    }
  }
}
