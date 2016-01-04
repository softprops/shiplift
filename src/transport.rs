//! Transports for communicating with the docker daemon

extern crate hyper;
extern crate mime;

use hyper::Client;
use hyper::client;
use self::super::{Error, Result};
use self::hyper::buffer::BufReader;
use self::hyper::header::ContentType;
use self::hyper::status::StatusCode;
use hyper::method::Method;
use std::fmt;
use std::ops::DerefMut;
use std::io::{Read, Write};
use hyperlocal::DomainUrl;

/// Transports are types which define the means of communication
/// with the docker daemon
pub enum Transport {
    /// A network tcp interface
    Tcp {
        client: Client,
        host: String,
    },
    /// A Unix domain socket
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
    pub fn request(&self, method: Method, endpoint: &str, body: Option<Body>) -> Result<String> {
        let mut res = match self.stream(method, endpoint, body) {
            Ok(r) => r,
            Err(e) => panic!("failed request {:?}", e),
        };
        let mut body = String::new();
        try!(res.read_to_string(&mut body));
        Ok(body)
    }

    pub fn stream(&self, method: Method, endpoint: &str, body: Option<Body>) -> Result<Box<Read>> {
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
                req.header(ContentType::json()).body(client::Body::SizedBody(reader, l))
            }
            _ => req,
        };
        let res = try!(embodied.send());
        match res.status {
            StatusCode::Ok | StatusCode::Created | StatusCode::SwitchingProtocols => {
                Ok(Box::new(res))
            }
            StatusCode::NoContent => Ok(Box::new(BufReader::new("".as_bytes()))),
            // todo: constantize these
            StatusCode::BadRequest => {
                Err(Error::Fault {
                    code: res.status,
                    message: "bad parameter".to_owned(),
                })
            }
            StatusCode::NotFound => {
                Err(Error::Fault {
                    code: res.status,
                    message: "not found".to_owned(),
                })
            }
            StatusCode::NotAcceptable => {
                Err(Error::Fault {
                    code: res.status,
                    message: "not acceptable".to_owned(),
                })
            }
            StatusCode::Conflict => {
                Err(Error::Fault {
                    code: res.status,
                    message: "conflict found".to_owned(),
                })
            }
            StatusCode::InternalServerError => {
                Err(Error::Fault {
                    code: res.status,
                    message: "internal server error".to_owned(),
                })
            }
            _ => unreachable!(),
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
