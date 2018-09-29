//! Transports for communicating with the docker daemon

extern crate hyper;
#[cfg(feature = "unix-socket")]
extern crate hyperlocal;

use self::super::{Error, Result};
use hyper::client::{Client, HttpConnector};
use hyper::header;
use hyper::rt::Stream;
use hyper::Body;
use hyper::{Method, Request, Response, StatusCode};
use hyper_openssl::HttpsConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::UnixConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::Uri as DomainUri;
use mime::Mime;
use rustc_serialize::json;
use std::cell::{RefCell, RefMut};
use std::fmt;
use std::io::Read;
use std::io::{BufReader, Cursor};
use tokio::runtime::Runtime;

pub fn tar() -> Mime {
    "application/tar".parse().unwrap()
}

/// Transports are types which define the means of communication
/// with the docker daemon
pub enum Transport {
    /// A network tcp interface
    Tcp {
        client: Client<HttpConnector>,
        runtime: RefCell<Runtime>,
        host: String,
    },
    /// TCP/TLS
    EncryptedTcp {
        client: Client<HttpsConnector<HttpConnector>>,
        runtime: RefCell<Runtime>,
        host: String,
    },
    /// A Unix domain socket
    #[cfg(feature = "unix-socket")]
    Unix {
        client: Client<UnixConnector>,
        runtime: RefCell<Runtime>,
        path: String,
    },
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Transport::Tcp { ref host, .. } => write!(f, "Tcp({})", host),
            Transport::EncryptedTcp { ref host, .. } => {
                write!(f, "EncryptedTcp({})", host)
            }
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => write!(f, "Unix({})", path),
        }
    }
}

impl Transport {
    pub fn request<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> Result<String>
    where
        B: Into<Body>,
    {
        let mut res = self.stream(method, endpoint, body)?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        debug!("{} raw response: {}", endpoint, body);
        Ok(body)
    }

    /// Builds an HTTP request.
    fn build_request<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> Result<Request<Body>>
    where
        B: Into<Body>,
    {
        let mut builder = Request::builder();
        let req = match *self {
            Transport::Tcp { ref host, .. } => {
                builder.method(method).uri(&format!("{}{}", host, endpoint))
            }
            Transport::EncryptedTcp { ref host, .. } => {
                builder.method(method).uri(&format!("{}{}", host, endpoint))
            }
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => {
                let uri: hyper::Uri = DomainUri::new(&path, endpoint).into();
                builder.method(method).uri(&uri.to_string())
            }
        };
        let req = req.header(header::HOST, "");

        match body {
            Some((b, c)) => Ok(req
                .header(header::CONTENT_TYPE, &c.to_string()[..])
                .body(b.into())?),
            _ => Ok(req.body(Body::empty())?),
        }
    }

    pub fn stream<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> Result<Box<Read>>
    where
        B: Into<Body>,
    {
        let req = self.build_request(method, endpoint, body)?;
        let res = self.send_request(req)?;

        match res.status() {
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::SWITCHING_PROTOCOLS => {
                let chunk =
                    self.runtime().block_on(res.into_body().concat2())?;
                Ok(Box::new(Cursor::new(
                    chunk.into_iter().collect::<Vec<u8>>(),
                )))
            }
            StatusCode::NO_CONTENT => {
                Ok(Box::new(BufReader::new("".as_bytes())))
            }
            // todo: constantize these
            StatusCode::BAD_REQUEST => Err(Error::Fault {
                code: res.status(),
                message: self
                    .get_error_message(res)
                    .unwrap_or("bad parameter".to_owned()),
            }),
            StatusCode::NOT_FOUND => Err(Error::Fault {
                code: res.status(),
                message: self
                    .get_error_message(res)
                    .unwrap_or("not found".to_owned()),
            }),
            StatusCode::NOT_MODIFIED => Err(Error::Fault {
                code: res.status(),
                message: self
                    .get_error_message(res)
                    .unwrap_or("not modified".to_owned()),
            }),
            StatusCode::NOT_ACCEPTABLE => Err(Error::Fault {
                code: res.status(),
                message: self
                    .get_error_message(res)
                    .unwrap_or("not acceptable".to_owned()),
            }),
            StatusCode::CONFLICT => Err(Error::Fault {
                code: res.status(),
                message: self
                    .get_error_message(res)
                    .unwrap_or("conflict found".to_owned()),
            }),
            StatusCode::INTERNAL_SERVER_ERROR => Err(Error::Fault {
                code: res.status(),
                message: self
                    .get_error_message(res)
                    .unwrap_or("internal server error".to_owned()),
            }),
            _ => unreachable!(),
        }
    }

    fn send_request(
        &self,
        req: Request<hyper::Body>,
    ) -> Result<hyper::Response<Body>> {
        let req = match self {
            Transport::Tcp { ref client, .. } => client.request(req),
            Transport::EncryptedTcp { ref client, .. } => client.request(req),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref client, .. } => client.request(req),
        };

        self.runtime().block_on(req).map_err(Error::Hyper)
    }

    fn runtime(&self) -> RefMut<Runtime> {
        match self {
            Transport::Tcp { ref runtime, .. } => runtime.borrow_mut(),
            Transport::EncryptedTcp { ref runtime, .. } => runtime.borrow_mut(),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref runtime, .. } => runtime.borrow_mut(),
        }
    }

    /// Extract the error message content from an HTTP response that
    /// contains a Docker JSON error structure.
    fn get_error_message(&self, res: Response<Body>) -> Option<String> {
        let chunk = match self.runtime().block_on(res.into_body().concat2()) {
            Ok(c) => c,
            Err(..) => return None,
        };

        match String::from_utf8(chunk.into_iter().collect()) {
            Ok(output) => {
                let json_response = json::Json::from_str(output.as_str()).ok();
                let message = json_response
                    .as_ref()
                    .and_then(|x| x.as_object())
                    .and_then(|x| x.get("message"))
                    .and_then(|x| x.as_string())
                    .map(|x| x.to_owned());

                message
            }
            Err(..) => None,
        }
    }
}
