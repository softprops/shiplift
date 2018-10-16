//! Transports for communicating with the docker daemon

extern crate hyper;
#[cfg(feature = "unix-socket")]
extern crate hyperlocal;

use self::super::{Error, Result};
use futures::{future, Future, IntoFuture, Stream};
use hyper::client::{Client, HttpConnector};
use hyper::header;
use hyper::Body;
use hyper::{Method, Request, StatusCode};
use hyper_openssl::HttpsConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::UnixConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::Uri as DomainUri;
use mime::Mime;
use std::fmt;

pub fn tar() -> Mime {
    "application/tar".parse().unwrap()
}

/// Transports are types which define the means of communication
/// with the docker daemon
pub enum Transport {
    /// A network tcp interface
    Tcp {
        client: Client<HttpConnector>,
        host: String,
    },
    /// TCP/TLS
    EncryptedTcp {
        client: Client<HttpsConnector<HttpConnector>>,
        host: String,
    },
    /// A Unix domain socket
    #[cfg(feature = "unix-socket")]
    Unix {
        client: Client<UnixConnector>,
        path: String,
    },
}

impl fmt::Debug for Transport {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
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
    ) -> impl Future<Item = String, Error = Error>
    where
        B: Into<Body>,
    {
        self.stream(method, endpoint, body)
            .concat2()
            .and_then(|v| String::from_utf8(v).map_err(Error::Encoding).into_future())
        // debug!("{} raw response: {}", endpoint, body);
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
    ) -> impl Stream<Item = Vec<u8>, Error = Error>
    where
        B: Into<Body>,
    {
        let req = self
            .build_request(method, endpoint, body)
            .expect("Failed to build request!");

        self.send_request(req)
            .and_then(|res| {
                let status = res.status();
                match status {
                    StatusCode::OK
                    | StatusCode::CREATED
                    | StatusCode::SWITCHING_PROTOCOLS
                    | StatusCode::NO_CONTENT => future::ok(res),
                    _ => future::err(Error::Fault {
                        code: status,
                        // TODO get_error_message
                        message: status
                            .canonical_reason()
                            .unwrap_or("unknown error code")
                            .to_owned(),
                    }),
                }
            })
            .map(|r| {
                r.into_body()
                    .map(|chunk| chunk.into_iter().collect::<Vec<u8>>())
                    .map_err(Error::Hyper)
            })
            .flatten_stream()
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

    // Extract the error message content from an HTTP response that
    // contains a Docker JSON error structure.
    // fn get_error_message(&self, body: &str) -> Option<String> {
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
