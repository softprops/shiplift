//! Transports for communicating with the docker daemon

extern crate hyper;
extern crate mime;

use hyper::Client;
use hyper::client;
use self::hyper::buffer::BufReader;
use self::hyper::header::ContentType;
use self::hyper::status::StatusCode;
use hyper::method::Method;
use self::mime::{Attr, Mime, Value};
use self::mime::TopLevel::Application;
use self::mime::SubLevel::Json;
use std::fmt;
use std::ops::DerefMut;
use std::io::{Error, ErrorKind, Read, Result, Write};
use hyperlocal::DomainUrl;

fn lift_status_err(status: u16) -> Result<Box<Read>> {
    match status {
        400 => Err(Error::new(ErrorKind::InvalidInput, "bad parameter")),
        404 => Err(Error::new(ErrorKind::InvalidInput, "not found")),
        406 => Err(Error::new(ErrorKind::InvalidInput, "not acceptable")),
        409 => Err(Error::new(ErrorKind::InvalidInput, "conflict found")),
        500 => Err(Error::new(ErrorKind::InvalidInput, "interal server error")),
        _ => unreachable!(),
    }
}

pub enum Transport {
    Tcp {
        client: Client,
        host: String,
    },
    Unix {
        client: Client,
        path: String,
    },
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Transport::Tcp { ref host, .. } => write!(f, "Tcp({})", host),
            Transport::Unix { ref path, .. } => write!(f, "Unix({})", path),
        }
    }
}

impl Transport {
    pub fn request(&mut self,
                   method: Method,
                   endpoint: &str,
                   body: Option<Body>)
                   -> Result<String> {
        let mut res = match self.stream(method, endpoint, body) {
            Ok(r) => r,
            Err(e) => panic!("failed request {:?}", e),
        };
        let mut body = String::new();
        res.read_to_string(&mut body).map(|_| body)
    }

    pub fn stream(&mut self,
                  method: Method,
                  endpoint: &str,
                  body: Option<Body>)
                  -> Result<Box<Read>> {
        println!("requesting {:?} {:?}", self, endpoint);
        let req = match *self {
            Transport::Tcp { ref client, ref host } => {
                client.request(method, &format!("{}{}", host, endpoint)[..])
            }
            Transport::Unix {  ref client, ref path } => {
                client.request(method, DomainUrl::new(&path, endpoint))
            }
        };

        let embodied = match body {
            Some(Body { read: r, size: l }) => {
                let reader: &mut Read = *r.deref_mut();
                let content_type: Mime = Mime(Application,
                                              Json,
                                              vec![(Attr::Charset, Value::Utf8)]);
                req.header(ContentType(content_type)).body(client::Body::SizedBody(reader, l))
            }
            _ => req,
        };
        let res = match embodied.send() {
            Ok(r) => r,
            Err(e) => panic!("failed request {:?}", e),
        };
        match res.status {
            StatusCode::Ok | StatusCode::Created | StatusCode::SwitchingProtocols => {
                Ok(Box::new(res))
            }
            StatusCode::NoContent => Ok(Box::new(BufReader::new("".as_bytes()))),
            status => lift_status_err(status.to_u16()),
        }
    }
}

#[doc(hidden)]
pub struct Body<'a> {
    read: &'a mut Box<&'a mut Read>,
    size: u64,
}

impl<'a> Body<'a> {
    /// Create a new body instance
    pub fn new(read: &'a mut Box<&'a mut Read>, size: u64) -> Body<'a> {
        Body {
            read: read,
            size: size,
        }
    }
}
