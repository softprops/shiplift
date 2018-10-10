//! Transports for communicating with the docker daemon

extern crate hyper;
#[cfg(feature = "unix-socket")]
extern crate hyperlocal;

use self::super::{Error, Result};
use futures::{future, Future, IntoFuture};
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
use serde_json::{self, Value};
use std::cell::{RefCell, RefMut};
use std::fmt;
use std::io::Read;
use std::io::{BufReader, Cursor};
use std::sync::{Arc, Mutex, MutexGuard};
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
        runtime: Arc<Mutex<Runtime>>,
        host: String,
    },
    /// TCP/TLS
    EncryptedTcp {
        client: Client<HttpsConnector<HttpConnector>>,
        runtime: Arc<Mutex<Runtime>>,
        host: String,
    },
    /// A Unix domain socket
    #[cfg(feature = "unix-socket")]
    Unix {
        client: Client<UnixConnector>,
        runtime: Arc<Mutex<Runtime>>,
        path: String,
    },
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Transport::Tcp { ref host, .. } => write!(f, "Tcp({})", host),
            Transport::EncryptedTcp { ref host, .. } => write!(f, "EncryptedTcp({})", host),
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
    ) -> Result<Box<Read + Send>>
    where
        B: Into<Body>,
    {
        let req = self.build_request(method, endpoint, body)?;

        let fut = self
            .send_request(req)
            .and_then(|r| {
                let status = r.status();
                r.into_body()
                    .concat2()
                    .map(|chunk| chunk.into_iter().collect::<Vec<u8>>())
                    .map_err(Error::Hyper)
                    .map(move |b| (status, b))
            }).and_then(|(status, body)| match status {
                StatusCode::OK | StatusCode::CREATED | StatusCode::SWITCHING_PROTOCOLS => {
                    String::from_utf8(body)
                        .map(|s| Box::new(Cursor::new(s)) as Box<Read + Send>)
                        .map_err(|_| Error::InvalidUTF8)
                        .into_future()
                }
                StatusCode::NO_CONTENT => {
                    future::ok(Box::new(BufReader::new("".as_bytes())) as Box<Read + Send>)
                }
                _ => future::err(Error::Fault {
                    code: status,
                    // TODO get_error_message
                    message: status
                        .canonical_reason()
                        .unwrap_or("unknown error code")
                        .to_owned(),
                }),
            });
        self.runtime().block_on(fut)
    }

    fn send_request(
        &self,
        req: Request<hyper::Body>,
    ) -> impl Future<Item = hyper::Response<Body>, Error = Error> {
        let req = match self {
            Transport::Tcp { ref client, .. } => client.request(req),
            Transport::EncryptedTcp { ref client, .. } => client.request(req),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref client, .. } => client.request(req),
        };

        req.map_err(Error::Hyper)
    }

    fn runtime(&self) -> MutexGuard<Runtime> {
        match self {
            Transport::Tcp { ref runtime, .. } => runtime.lock().unwrap(),
            Transport::EncryptedTcp { ref runtime, .. } => runtime.lock().unwrap(),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref runtime, .. } => runtime.lock().unwrap(),
        }
    }

    // /// Extract the error message content from an HTTP response that
    // /// contains a Docker JSON error structure.
    // fn get_error_message(
    //     &self,
    //     res: Response<Body>,
    // ) -> Option<String> {
    //     let chunk = match self.runtime().block_on(res.into_body().concat2()) {
    //         Ok(c) => c,
    //         Err(..) => return None,
    //     };

    //     match String::from_utf8(chunk.into_iter().collect()) {
    //         Ok(output) => {
    //             let json_response = serde_json::from_str::<Value>(output.as_str()).ok();
    //             json_response
    //                 .as_ref()
    //                 .and_then(|x| x.as_object())
    //                 .and_then(|x| x.get("message"))
    //                 .and_then(|x| x.as_str())
    //                 .map(|x| x.to_owned())
    //         }
    //         Err(..) => None,
    //     }
    // }
}
