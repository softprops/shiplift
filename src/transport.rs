//! Transports for communicating with the docker daemon

use self::super::{Error, Result};
use futures::{
    future::{self, Either},
    Future, IntoFuture, Stream,
};
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Chunk, Method, Request, StatusCode,
};
use hyper_openssl::HttpsConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::UnixConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::Uri as DomainUri;
use mime::Mime;
use serde_json;
use std::fmt;
use tokio_io::{AsyncRead, AsyncWrite};

pub fn tar() -> Mime {
    "application/tar".parse().unwrap()
}

/// Transports are types which define the means of communication
/// with the docker daemon
#[derive(Clone)]
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
    /// Make a request and return the whole response in a `String`
    pub fn request<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Future<Item = String, Error = Error>
    where
        B: Into<Body>,
    {
        let endpoint = endpoint.to_string();
        self.stream_chunks(method, &endpoint, body)
            .concat2()
            .and_then(|v| {
                String::from_utf8(v.to_vec())
                    .map_err(Error::Encoding)
                    .into_future()
            })
            .inspect(move |body| debug!("{} raw response: {}", endpoint, body))
    }

    /// Make a request and return a `Stream` of `Chunks` as they are returned.
    pub fn stream_chunks<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Stream<Item = Chunk, Error = Error>
    where
        B: Into<Body>,
    {
        let req = self
            .build_request(method, endpoint, body, |_| ())
            .expect("Failed to build request!");

        self.send_request(req)
            .and_then(|res| {
                let status = res.status();
                match status {
                    // Success case: pass on the response
                    StatusCode::OK
                    | StatusCode::CREATED
                    | StatusCode::SWITCHING_PROTOCOLS
                    | StatusCode::NO_CONTENT => Either::A(future::ok(res)),
                    // Error case: parse the body to try to extract the error message
                    _ => Either::B(
                        res.into_body()
                            .concat2()
                            .map_err(Error::Hyper)
                            .and_then(|v| {
                                String::from_utf8(v.into_iter().collect::<Vec<u8>>())
                                    .map_err(Error::Encoding)
                            })
                            .and_then(move |body| {
                                future::err(Error::Fault {
                                    code: status,
                                    message: Self::get_error_message(&body).unwrap_or_else(|| {
                                        status
                                            .canonical_reason()
                                            .unwrap_or_else(|| "unknown error code")
                                            .to_owned()
                                    }),
                                })
                            }),
                    ),
                }
            })
            .map(|r| {
                // Convert the response body into a stream of chunks
                r.into_body().map_err(Error::Hyper)
            })
            .flatten_stream()
    }

    /// Builds an HTTP request.
    fn build_request<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
        f: impl FnOnce(&mut ::http::request::Builder),
    ) -> Result<Request<Body>>
    where
        B: Into<Body> {
        let mut builder = Request::builder();
        f(&mut builder);

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

    /// Send the given request to the docker daemon and return a Future of the response.
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

    /// Makes an HTTP request, upgrading the connection to a TCP
    /// stream on success.
    ///
    /// This method can be used for operations such as viewing
    /// docker container logs interactively.
    pub fn stream_upgrade<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Future<Item = impl AsyncRead + AsyncWrite, Error = Error>
    where
        B: Into<Body>
    {
        match self {
            Transport::Tcp { .. } | Transport::EncryptedTcp { .. } => (),
            _ => panic!("connection streaming is only supported over TCP"),
        };

        let req = self.build_request(method, endpoint, body, |builder| {
            builder.header(header::CONNECTION, "Upgrade")
                .header(header::UPGRADE, "tcp");
        }).expect("Failed to build request!");

        self.send_request(req).and_then(|res| {
            if res.status() != StatusCode::SWITCHING_PROTOCOLS {
                panic!("Our server didn't upgrade: {}", res.status());
            }

            res.into_body()
               .on_upgrade()
               .from_err()
        })
    }

    pub fn stream_upgrade_multiplexed<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Future<Item = ::tty::Multiplexed, Error = Error>
    where
        B: Into<Body> + 'static
    {
        self.stream_upgrade(method, endpoint, body)
            .map(|u| ::tty::Multiplexed::new(u))
    }

    /// Extract the error message content from an HTTP response that
    /// contains a Docker JSON error structure.
    fn get_error_message(body: &str) -> Option<String> {
        serde_json::from_str::<ErrorResponse>(body)
            .map(|e| e.message)
            .ok()
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    message: String,
}
