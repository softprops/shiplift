extern crate hyper;
extern crate unix_socket;

use hyper::Client;
use hyper::method::Method;
use std::io::{ self, Read, Result, Write };
use unix_socket::UnixStream;

pub trait Transport {
  fn request(&mut self, method: Method, endpoint: &str) -> Result<String>;
  fn stream(&mut self, method: Method, endpoint: &str) -> Result<Box<Read>>;
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
    Err(io::Error::new(io::ErrorKind::InvalidInput, "Not yet implemented"))
  }
}

impl Transport for (Client, String) {
  fn request(&mut self, method: Method, endpoint: &str) -> Result<String> {
    let mut res = match self.stream(method, endpoint) {
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
