//! Transports for communicating with the docker daemon

use crate::{Error, Result};
use futures_util::{stream::Stream, TryFutureExt, TryStreamExt};
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Chunk, Method, Request, StatusCode,
};
#[cfg(feature = "tls")]
use hyper_openssl::HttpsConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::UnixConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::Uri as DomainUri;
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fmt, iter};

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
    #[cfg(feature = "tls")]
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
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref host, .. } => write!(f, "EncryptedTcp({})", host),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => write!(f, "Unix({})", path),
        }
    }
}

impl Transport {
    /// Make a request and return the whole response in a `String`
    pub async fn request<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<String> 
    where B: Into<Body>,
    {
        let chunk = self
            .stream_chunks(method, endpoint, body, None::<iter::Empty<_>>)
            .try_concat()
            .await?;

        let string = String::from_utf8(chunk.to_vec())?;

        Ok(string)
    }

    async fn get_body<B, H>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> Result<Body>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let req = self
            .build_request(method, endpoint, body, headers, |_| ())
            .expect("Failed to build request!");

        let response = self.send_request(req).await?;

        let status = response.status();

        match status {
            // Success case: pass on the response
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::SWITCHING_PROTOCOLS
            | StatusCode::NO_CONTENT => Ok(response.into_body()),
            _ => {
                let chunk = concat_chunks(response.into_body()).await?;

                let message_body = String::from_utf8(chunk.into_bytes().into_iter().collect())?;

                Err(Error::Fault {
                    code: status,
                    message: Self::get_error_message(&message_body).unwrap_or_else(|| {
                        status
                            .canonical_reason()
                            .unwrap_or_else(|| "unknown error code")
                            .to_owned()
                    }),
                })
            }
        }
    }

    async fn x<B, H>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> Result<impl Stream<Item = Result<Chunk>>>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let body = self.get_body(method, endpoint, body, headers).await?;

        Ok(stream_body(body))
    }

    pub fn stream_chunks<'a, H, B>(
        &'a self,
        method: Method,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<Chunk>> + 'a
    where
        H: IntoIterator<Item = (&'static str, String)> + 'a,
        B: Into<Body> + 'a,
    {
        self.x(method, endpoint, body, headers).try_flatten_stream()
    }

    /*     /// Make a request and return a `Stream` of `Chunks` as they are returned.
    pub fn stream_chunks<B, H>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<Chunk>>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let req = self
            .build_request(method, endpoint, body, headers, |_| ())
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
    } */

    /// Builds an HTTP request.
    fn build_request<B, H>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
        f: impl FnOnce(&mut ::http::request::Builder),
    ) -> Result<Request<Body>>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let mut builder = Request::builder();
        f(&mut builder);

        let req = match *self {
            Transport::Tcp { ref host, .. } => {
                builder
                    .method(method)
                    .uri(&format!("{}{}", host, endpoint.as_ref()))
            }
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref host, .. } => {
                builder
                    .method(method)
                    .uri(&format!("{}{}", host, endpoint.as_ref()))
            }
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => {
                let uri: hyper::Uri = DomainUri::new(&path, endpoint).into();
                builder.method(method).uri(&uri.to_string())
            }
        };
        let req = req.header(header::HOST, "");

        if let Some(h) = headers {
            for (k, v) in h.into_iter() {
                req.header(k, v);
            }
        }

        match body {
            Some((b, c)) => Ok(req
                .header(header::CONTENT_TYPE, &c.to_string()[..])
                .body(b.into())?),
            _ => Ok(req.body(Body::empty())?),
        }
    }

    /// Send the given request to the docker daemon and return a Future of the response.
    async fn send_request(
        &self,
        req: Request<hyper::Body>,
    ) -> Result<hyper::Response<Body>> {
        match self {
            Transport::Tcp { ref client, .. } => Ok(client.request(req).await?),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref client, .. } => Ok(client.request(req).await?),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref client, .. } => Ok(client.request(req).await?),
        }
    }

    /*     /// Makes an HTTP request, upgrading the connection to a TCP
       /// stream on success.
       ///
       /// This method can be used for operations such as viewing
       /// docker container logs interactively.
       pub async fn stream_upgrade<B>(
           &self,
           method: Method,
           endpoint: &str,
           body: Option<(B, Mime)>,
       ) -> Result<impl AsyncRead + AsyncWrite>
       where
           B: Into<Body>,
       {
           unimplemented!()
       }
    */
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

fn stream_body(body: Body) -> impl Stream<Item = Result<Chunk>> {
    futures_util::stream::unfold(body, stream_body_unfold)
}

async fn stream_body_unfold(mut body: Body) -> Option<(Result<Chunk>, Body)> {
    let chunk_result = body.next().await?.map_err(Error::from);

    Some((chunk_result, body))
}

async fn concat_chunks(body: Body) -> Result<Chunk> {
    stream_body(body).try_concat().await
}
