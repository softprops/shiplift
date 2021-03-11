//! Main entrypoint for interacting with the Docker API.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/>

use std::{collections::HashMap, env, io, path::Path};

use futures_util::{stream::Stream, TryStreamExt};
use hyper::{client::HttpConnector, Body, Client, Method};
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::form_urlencoded;

use crate::{
    container::Containers,
    errors::{Error, Result},
    image::Images,
    network::Networks,
    service::Services,
    transport::{Headers, Payload, Transport},
    volume::Volumes,
    Uri,
};

#[cfg(feature = "chrono")]
use crate::datetime::{datetime_from_nano_timestamp, datetime_from_unix_timestamp};
#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[cfg(feature = "tls")]
use hyper_openssl::HttpsConnector;
#[cfg(feature = "tls")]
use openssl::ssl::{SslConnector, SslFiletype, SslMethod};

#[cfg(feature = "unix-socket")]
use hyperlocal::UnixConnector;

/// Entrypoint interface for communicating with docker daemon
#[derive(Debug, Clone)]
pub struct Docker {
    transport: Transport,
}

fn get_http_connector() -> HttpConnector {
    let mut http = HttpConnector::new();
    http.enforce_http(false);

    http
}

#[cfg(feature = "tls")]
// To be deleted after 0.8.0
fn get_docker_for_tcp(tcp_host_str: String) -> Docker {
    let http = get_http_connector();
    if let Ok(ref certs) = env::var("DOCKER_CERT_PATH") {
        // fixme: don't unwrap before you know what's in the box
        // https://github.com/hyperium/hyper/blob/master/src/net.rs#L427-L428
        let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
        connector.set_cipher_list("DEFAULT").unwrap();
        let cert = &format!("{}/cert.pem", certs);
        let key = &format!("{}/key.pem", certs);
        connector
            .set_certificate_file(&Path::new(cert), SslFiletype::PEM)
            .unwrap();
        connector
            .set_private_key_file(&Path::new(key), SslFiletype::PEM)
            .unwrap();
        if env::var("DOCKER_TLS_VERIFY").is_ok() {
            let ca = &format!("{}/ca.pem", certs);
            connector.set_ca_file(&Path::new(ca)).unwrap();
        }

        // If we are attempting to connec to the docker daemon via tcp
        // we need to convert the scheme to `https` to let hyper connect.
        // Otherwise, hyper will reject the connection since it does not
        // recongnize `tcp` as a valid `http` scheme.
        let tcp_host_str = if tcp_host_str.contains("tcp://") {
            tcp_host_str.replace("tcp://", "https://")
        } else {
            tcp_host_str
        };

        Docker {
            transport: Transport::EncryptedTcp {
                client: Client::builder()
                    .build(HttpsConnector::with_connector(http, connector).unwrap()),
                host: tcp_host_str,
            },
        }
    } else {
        Docker {
            transport: Transport::Tcp {
                client: Client::builder().build(http),
                host: tcp_host_str,
            },
        }
    }
}

#[cfg(not(feature = "tls"))]
// To be deleted after 0.8.0
fn get_docker_for_tcp(tcp_host_str: String) -> Docker {
    let http = get_http_connector();
    Docker {
        transport: Transport::Tcp {
            client: Client::builder().build(http),
            host: tcp_host_str,
        },
    }
}

#[cfg(feature = "tls")]
fn get_docker_for_tcp_tls(
    host: String,
    cert_path: &Path,
    verify: bool,
) -> Result<Docker> {
    let http = get_http_connector();
    let mut connector = SslConnector::builder(SslMethod::tls())?;
    connector.set_cipher_list("DEFAULT")?;
    let cert = cert_path.join("cert.pem");
    let key = cert_path.join("key.pem");
    connector.set_certificate_file(cert.as_path(), SslFiletype::PEM)?;
    connector.set_private_key_file(key.as_path(), SslFiletype::PEM)?;
    if verify {
        let ca = cert_path.join("ca.pem");
        connector.set_ca_file(ca.as_path())?;
    }

    Ok(Docker {
        transport: Transport::EncryptedTcp {
            client: Client::builder().build(HttpsConnector::with_connector(http, connector)?),
            host: format!("https://{}", host),
        },
    })
}

impl Docker {
    /// Creates a new Docker instance by automatically choosing appropriate connection type based
    /// on provided `uri`.
    ///
    /// Supported schemes are:
    ///  - `unix://` with feature `unix-socket` enabled, otherwise returns an Error
    ///  - `tcp://`
    ///  - `http://`
    ///
    ///  To create a Docker instance utilizing TLS use explicit [Docker::tls](Docker::tls)
    ///  constructor.
    pub fn new<S>(uri: S) -> Result<Docker>
    where
        S: AsRef<str>,
    {
        let uri = uri.as_ref();
        let mut it = uri.split("://");

        match it.next() {
            #[cfg(feature = "unix-socket")]
            Some("unix") => {
                if let Some(path) = it.next() {
                    Ok(Docker::unix(path))
                } else {
                    Err(Error::MissingAuthority)
                }
            }
            #[cfg(not(feature = "unix-socket"))]
            Some("unix") => Err(Error::UnsupportedScheme("unix".to_string())),
            Some("tcp") | Some("http") => {
                if let Some(host) = it.next() {
                    Ok(Docker::tcp(host))
                } else {
                    Err(Error::MissingAuthority)
                }
            }
            Some(scheme) => Err(Error::UnsupportedScheme(scheme.to_string())),
            None => unreachable!(), // This is never possible because calling split on an empty string
                                    // always returns at least one element
        }
    }

    /// Creates a new docker instance for a docker host listening on a given Unix socket.
    ///
    /// `socket_path` is the part of URI that comes after the `unix://`. For example a URI `unix:///run/docker.sock` has a
    /// `socket_path` == "/run/docker.sock".
    #[cfg(feature = "unix-socket")]
    pub fn unix<S>(socket_path: S) -> Docker
    where
        S: Into<String>,
    {
        Docker {
            transport: Transport::Unix {
                client: Client::builder()
                    .pool_max_idle_per_host(0)
                    .build(UnixConnector),
                path: socket_path.into(),
            },
        }
    }

    #[cfg(feature = "tls")]
    /// Creates a new docker instance for a docker host listening on a given TCP socket `host`.
    /// `host` is the part of URI that comes after `tcp://` or `http://` or `https://` schemes,
    /// also known as authority part.
    ///
    /// `cert_path` specifies the base path in the filesystem containing a certificate (`cert.pem`)
    /// and a key (`key.pem`) that will be used by the client. If verify is `true` a CA file will be
    /// added (`ca.pem`) to the connector.
    pub fn tls<S, P>(
        host: S,
        cert_path: P,
        verify: bool,
    ) -> Result<Docker>
    where
        S: Into<String>,
        P: AsRef<Path>,
    {
        get_docker_for_tcp_tls(host.into(), cert_path.as_ref(), verify)
    }

    /// Creates a new docker instance for a docker host listening on a given TCP socket `host`.
    /// `host` is the part of URI that comes after `tcp://` or `http://` schemes, also known as
    /// authority part.
    ///
    /// TLS is supported with feature `tls` enabled through [Docker::tls](Docker::tls) constructor.
    pub fn tcp<S>(host: S) -> Docker
    where
        S: Into<String>,
    {
        let http = get_http_connector();
        Docker {
            transport: Transport::Tcp {
                client: Client::builder().build(http),
                host: format!("tcp://{}", host.into()),
            },
        }
    }

    #[deprecated(since = "0.9.0", note = "Please use Docker::new instead")]
    /// constructs a new Docker instance for docker host listening at the given host url
    pub fn host(host: Uri) -> Docker {
        let tcp_host_str = format!(
            "{}://{}:{}",
            host.scheme_str().unwrap(),
            host.host().unwrap().to_owned(),
            host.port_u16().unwrap_or(80)
        );

        match host.scheme_str() {
            #[cfg(feature = "unix-socket")]
            Some("unix") => Docker {
                transport: Transport::Unix {
                    client: Client::builder().build(UnixConnector),
                    path: host.path().to_owned(),
                },
            },

            #[cfg(not(feature = "unix-socket"))]
            Some("unix") => panic!("Unix socket support is disabled"),

            _ => get_docker_for_tcp(tcp_host_str),
        }
    }

    /// Exports an interface for interacting with docker images
    pub fn images(&'_ self) -> Images<'_> {
        Images::new(self)
    }

    /// Exports an interface for interacting with docker containers
    pub fn containers(&'_ self) -> Containers<'_> {
        Containers::new(self)
    }

    /// Exports an interface for interacting with docker services
    pub fn services(&'_ self) -> Services<'_> {
        Services::new(self)
    }

    pub fn networks(&'_ self) -> Networks<'_> {
        Networks::new(self)
    }

    pub fn volumes(&'_ self) -> Volumes<'_> {
        Volumes::new(self)
    }

    /// Returns version information associated with the docker daemon
    pub async fn version(&self) -> Result<Version> {
        self.get_json("/version").await
    }

    /// Returns information associated with the docker daemon
    pub async fn info(&self) -> Result<Info> {
        self.get_json("/info").await
    }

    /// Returns a simple ping response indicating the docker daemon is accessible
    pub async fn ping(&self) -> Result<String> {
        self.get("/_ping").await
    }

    /// Returns a stream of docker events
    pub fn events<'docker>(
        &'docker self,
        opts: &EventsOptions,
    ) -> impl Stream<Item = Result<Event>> + Unpin + 'docker {
        let mut path = vec!["/events".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let reader = Box::pin(
            self.stream_get(path.join("?"))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
        )
        .into_async_read();

        let codec = futures_codec::LinesCodec {};

        Box::pin(
            futures_codec::FramedRead::new(reader, codec)
                .map_err(Error::IO)
                .and_then(|s: String| async move {
                    serde_json::from_str(&s).map_err(Error::SerdeJsonError)
                }),
        )
    }

    //
    // Utility functions to make requests
    //

    pub(crate) async fn get(
        &self,
        endpoint: &str,
    ) -> Result<String> {
        self.transport
            .request(Method::GET, endpoint, Payload::None, Headers::None)
            .await
    }

    pub(crate) async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T> {
        let raw_string = self
            .transport
            .request(Method::GET, endpoint, Payload::None, Headers::None)
            .await?;

        Ok(serde_json::from_str::<T>(&raw_string)?)
    }

    pub(crate) async fn post(
        &self,
        endpoint: &str,
        body: Option<(Body, Mime)>,
    ) -> Result<String> {
        self.transport
            .request(Method::POST, endpoint, body, Headers::None)
            .await
    }

    pub(crate) async fn put(
        &self,
        endpoint: &str,
        body: Option<(Body, Mime)>,
    ) -> Result<String> {
        self.transport
            .request(Method::PUT, endpoint, body, Headers::None)
            .await
    }

    pub(crate) async fn post_json<T, B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: Into<Body>,
    {
        let string = self
            .transport
            .request(Method::POST, endpoint, body, Headers::None)
            .await?;

        Ok(serde_json::from_str::<T>(&string)?)
    }

    pub(crate) async fn post_json_headers<'a, T, B, H>(
        &self,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)> + 'a,
    {
        let string = self
            .transport
            .request(Method::POST, endpoint, body, headers)
            .await?;

        Ok(serde_json::from_str::<T>(&string)?)
    }

    pub(crate) async fn delete(
        &self,
        endpoint: &str,
    ) -> Result<String> {
        self.transport
            .request(Method::DELETE, endpoint, Payload::None, Headers::None)
            .await
    }

    pub(crate) async fn delete_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T> {
        let string = self
            .transport
            .request(Method::DELETE, endpoint, Payload::None, Headers::None)
            .await?;

        Ok(serde_json::from_str::<T>(&string)?)
    }

    /// Send a streaming post request.
    ///
    /// Use stream_post_into_values if the endpoint returns JSON values
    pub(crate) fn stream_post<'a, H>(
        &'a self,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(Body, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<hyper::body::Bytes>> + 'a
    where
        H: IntoIterator<Item = (&'static str, String)> + 'a,
    {
        self.transport
            .stream_chunks(Method::POST, endpoint, body, headers)
    }

    /// Send a streaming post request that returns a stream of JSON values
    ///
    /// Assumes that each received chunk contains one or more JSON values
    pub(crate) fn stream_post_into_values<'a, H>(
        &'a self,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(Body, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<Value>> + 'a
    where
        H: IntoIterator<Item = (&'static str, String)> + 'a,
    {
        self.stream_post(endpoint, body, headers)
            .and_then(|chunk| async move {
                let stream = futures_util::stream::iter(
                    serde_json::Deserializer::from_slice(&chunk)
                        .into_iter()
                        .collect::<Vec<_>>(),
                )
                .map_err(Error::from);

                Ok(stream)
            })
            .try_flatten()
    }

    pub(crate) fn stream_get<'a>(
        &'a self,
        endpoint: impl AsRef<str> + Unpin + 'a,
    ) -> impl Stream<Item = Result<hyper::body::Bytes>> + 'a {
        let headers = Some(Vec::default());
        self.transport
            .stream_chunks(Method::GET, endpoint, Option::<(Body, Mime)>::None, headers)
    }

    pub(crate) async fn stream_post_upgrade<'a>(
        &'a self,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(Body, Mime)>,
    ) -> Result<impl futures_util::io::AsyncRead + futures_util::io::AsyncWrite + 'a> {
        self.transport
            .stream_upgrade(Method::POST, endpoint, body)
            .await
    }
}

/// Options for filtering streams of Docker events
#[derive(Default, Debug)]
pub struct EventsOptions {
    params: HashMap<&'static str, String>,
}

impl EventsOptions {
    pub fn builder() -> EventsOptionsBuilder {
        EventsOptionsBuilder::default()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(&self.params)
                    .finish(),
            )
        }
    }
}

#[derive(Copy, Clone)]
pub enum EventFilterType {
    Container,
    Image,
    Volume,
    Network,
    Daemon,
}

fn event_filter_type_to_string(filter: EventFilterType) -> &'static str {
    match filter {
        EventFilterType::Container => "container",
        EventFilterType::Image => "image",
        EventFilterType::Volume => "volume",
        EventFilterType::Network => "network",
        EventFilterType::Daemon => "daemon",
    }
}

/// Filter options for image listings
pub enum EventFilter {
    Container(String),
    Event(String),
    Image(String),
    Label(String),
    Type(EventFilterType),
    Volume(String),
    Network(String),
    Daemon(String),
}

/// Builder interface for `EventOptions`
#[derive(Default)]
pub struct EventsOptionsBuilder {
    params: HashMap<&'static str, String>,
    events: Vec<String>,
    containers: Vec<String>,
    images: Vec<String>,
    labels: Vec<String>,
    volumes: Vec<String>,
    networks: Vec<String>,
    daemons: Vec<String>,
    types: Vec<String>,
}

impl EventsOptionsBuilder {
    /// Filter events since a given timestamp
    pub fn since(
        &mut self,
        ts: &u64,
    ) -> &mut Self {
        self.params.insert("since", ts.to_string());
        self
    }

    /// Filter events until a given timestamp
    pub fn until(
        &mut self,
        ts: &u64,
    ) -> &mut Self {
        self.params.insert("until", ts.to_string());
        self
    }

    pub fn filter(
        &mut self,
        filters: Vec<EventFilter>,
    ) -> &mut Self {
        let mut params = HashMap::new();
        for f in filters {
            match f {
                EventFilter::Container(n) => {
                    self.containers.push(n);
                    params.insert("container", self.containers.clone())
                }
                EventFilter::Event(n) => {
                    self.events.push(n);
                    params.insert("event", self.events.clone())
                }
                EventFilter::Image(n) => {
                    self.images.push(n);
                    params.insert("image", self.images.clone())
                }
                EventFilter::Label(n) => {
                    self.labels.push(n);
                    params.insert("label", self.labels.clone())
                }
                EventFilter::Volume(n) => {
                    self.volumes.push(n);
                    params.insert("volume", self.volumes.clone())
                }
                EventFilter::Network(n) => {
                    self.networks.push(n);
                    params.insert("network", self.networks.clone())
                }
                EventFilter::Daemon(n) => {
                    self.daemons.push(n);
                    params.insert("daemon", self.daemons.clone())
                }
                EventFilter::Type(n) => {
                    let event_type = event_filter_type_to_string(n).to_string();
                    self.types.push(event_type);
                    params.insert("type", self.types.clone())
                }
            };
        }
        self.params
            .insert("filters", serde_json::to_string(&params).unwrap());
        self
    }

    pub fn build(&self) -> EventsOptions {
        EventsOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Version {
    pub version: String,
    pub api_version: String,
    pub git_commit: String,
    pub go_version: String,
    pub os: String,
    pub arch: String,
    pub kernel_version: String,
    #[cfg(feature = "chrono")]
    pub build_time: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub build_time: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Info {
    pub containers: u64,
    pub images: u64,
    pub driver: String,
    pub docker_root_dir: String,
    pub driver_status: Vec<Vec<String>>,
    #[serde(rename = "ID")]
    pub id: String,
    pub kernel_version: String,
    // pub Labels: Option<???>,
    pub mem_total: u64,
    pub memory_limit: bool,
    #[serde(rename = "NCPU")]
    pub n_cpu: u64,
    pub n_events_listener: u64,
    pub n_goroutines: u64,
    pub name: String,
    pub operating_system: String,
    // pub RegistryConfig:???
    pub swap_limit: bool,
    pub system_time: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "Type")]
    pub typ: String,
    #[serde(rename = "Action")]
    pub action: String,
    #[serde(rename = "Actor")]
    pub actor: Actor,
    pub status: Option<String>,
    pub id: Option<String>,
    pub from: Option<String>,
    #[cfg(feature = "chrono")]
    #[serde(deserialize_with = "datetime_from_unix_timestamp")]
    pub time: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub time: u64,
    #[cfg(feature = "chrono")]
    #[serde(deserialize_with = "datetime_from_nano_timestamp", rename = "timeNano")]
    pub time_nano: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    #[serde(rename = "timeNano")]
    pub time_nano: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Attributes")]
    pub attributes: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::{Docker, Error};
    #[test]
    fn creates_correct_docker() {
        let d = Docker::new("tcp://127.0.0.1:80");
        d.unwrap();
        let d = Docker::new("http://127.0.0.1:80");
        d.unwrap();

        #[cfg(feature = "unix-socket")]
        let d = Docker::new("unix://127.0.0.1:80");
        d.unwrap();

        #[cfg(not(feature = "unix-socket"))]
        {
            let d = Docker::new("unix://127.0.0.1:80");
            assert!(d.is_err());
            match d.unwrap_err() {
                Error::UnsupportedScheme(scheme) if &scheme == "unix" => {}
                e => panic!(r#"Expected Error::UnsupportedScheme("unix"), got {}"#, e),
            }
        }

        let d = Docker::new("rand://127.0.0.1:80");
        match d.unwrap_err() {
            Error::UnsupportedScheme(scheme) if &scheme == "rand" => {}
            e => panic!(r#"Expected Error::UnsupportedScheme("rand"), got {}"#, e),
        }

        let d = Docker::new("invalid_uri");
        match d.unwrap_err() {
            Error::UnsupportedScheme(scheme) if &scheme == "invalid_uri" => {}
            e => panic!(
                r#"Expected Error::UnsupportedScheme("invalid_uri"), got {}"#,
                e
            ),
        }
        let d = Docker::new("");
        match d.unwrap_err() {
            Error::UnsupportedScheme(scheme) if scheme.is_empty() => {}
            e => panic!(r#"Expected Error::UnsupportedScheme(""), got {}"#, e),
        }
    }
}
