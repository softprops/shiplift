//! Shiplift is a multi-transport utility for maneuvering [docker](https://www.docker.com/) containers
//!
//! # examples
//!
//! ```no_run
//! use tokio::prelude::Future;
//!
//! let docker = shiplift::Docker::new();
//! let fut = docker.images().list(&Default::default()).map(|images| {
//!   println!("docker images in stock");
//!   for i in images {
//!     println!("{:?}", i.repo_tags);
//!   }
//! }).map_err(|e| eprintln!("Something bad happened! {}", e));
//!
//! tokio::run(fut);
//! ```

pub mod builder;
pub mod errors;
pub mod read;
pub mod rep;
pub mod transport;
pub mod tty;

mod tarball;

pub use crate::{
    builder::{
        BuildOptions, ContainerConnectionOptions, ContainerFilter, ContainerListOptions,
        ContainerOptions, EventsOptions, ExecContainerOptions, ImageFilter, ImageListOptions,
        LogsOptions, NetworkCreateOptions, NetworkListOptions, PullOptions, RegistryAuth,
        RmContainerOptions, TagOptions, VolumeCreateOptions,
    },
    errors::Error,
};
use crate::{
    read::StreamReader,
    rep::{
        Change, Container as ContainerRep, ContainerCreateInfo, ContainerDetails, Event, Exit,
        History, Image as ImageRep, ImageDetails, Info, NetworkCreateInfo,
        NetworkDetails as NetworkInfo, SearchResult, Stats, Status, Top, Version,
        Volume as VolumeRep, VolumeCreateInfo, Volumes as VolumesRep,
    },
    transport::{tar, Transport},
    tty::TtyDecoder,
};
use futures::{future::Either, Future, IntoFuture, Stream};
pub use hyper::Uri;
use hyper::{client::HttpConnector, Body, Client, Method};
#[cfg(feature = "tls")]
use hyper_openssl::HttpsConnector;
#[cfg(feature = "unix-socket")]
use hyperlocal::UnixConnector;
use mime::Mime;
#[cfg(feature = "tls")]
use openssl::ssl::{SslConnector, SslFiletype, SslMethod};
use serde_json::Value;
use std::{borrow::Cow, env, io::Read, iter, path::Path, time::Duration};
use tokio_codec::{FramedRead, LinesCodec};
use url::form_urlencoded;

/// Represents the result of all docker operations
pub type Result<T> = std::result::Result<T, Error>;

/// Entrypoint interface for communicating with docker daemon
#[derive(Clone)]
pub struct Docker {
    transport: Transport,
}

/// Interface for accessing and manipulating a named docker image
pub struct Image<'a, 'b> {
    docker: &'a Docker,
    name: Cow<'b, str>,
}

impl<'a, 'b> Image<'a, 'b> {
    /// Exports an interface for operations that may be performed against a named image
    pub fn new<S>(
        docker: &'a Docker,
        name: S,
    ) -> Image<'a, 'b>
    where
        S: Into<Cow<'b, str>>,
    {
        Image {
            docker,
            name: name.into(),
        }
    }

    /// Inspects a named image's details
    pub fn inspect(&self) -> impl Future<Item = ImageDetails, Error = Error> {
        self.docker
            .get_json(&format!("/images/{}/json", self.name)[..])
    }

    /// Lists the history of the images set of changes
    pub fn history(&self) -> impl Future<Item = Vec<History>, Error = Error> {
        self.docker
            .get_json(&format!("/images/{}/history", self.name)[..])
    }

    /// Deletes an image
    pub fn delete(&self) -> impl Future<Item = Vec<Status>, Error = Error> {
        self.docker
            .delete_json::<Vec<Status>>(&format!("/images/{}", self.name)[..])
    }

    /// Export this image to a tarball
    pub fn export(&self) -> impl Stream<Item = Vec<u8>, Error = Error> {
        self.docker
            .stream_get(&format!("/images/{}/get", self.name)[..])
            .map(|c| c.to_vec())
    }

    /// Adds a tag to an image
    pub fn tag(
        &self,
        opts: &TagOptions,
    ) -> impl Future<Item = (), Error = Error> {
        let mut path = vec![format!("/images/{}/tag", self.name)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker.post::<Body>(&path.join("?"), None).map(|_| ())
    }
}

/// Interface for docker images
pub struct Images<'a> {
    docker: &'a Docker,
}

impl<'a> Images<'a> {
    /// Exports an interface for interacting with docker images
    pub fn new(docker: &'a Docker) -> Images<'a> {
        Images { docker }
    }

    /// Builds a new image build by reading a Dockerfile in a target directory
    pub fn build(
        &self,
        opts: &BuildOptions,
    ) -> impl Stream<Item = Value, Error = Error> {
        let mut path = vec!["/build".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        let mut bytes = vec![];

        match tarball::dir(&mut bytes, &opts.path[..]) {
            Ok(_) => Box::new(
                self.docker
                    .stream_post(
                        &path.join("?"),
                        Some((Body::from(bytes), tar())),
                        None::<iter::Empty<_>>,
                    )
                    .map(|r| {
                        futures::stream::iter_result(
                            serde_json::Deserializer::from_slice(&r[..])
                                .into_iter::<Value>()
                                .collect::<Vec<_>>(),
                        )
                        .map_err(Error::from)
                    })
                    .flatten(),
            ) as Box<dyn Stream<Item = Value, Error = Error> + Send>,
            Err(e) => Box::new(futures::future::err(Error::IO(e)).into_stream())
                as Box<dyn Stream<Item = Value, Error = Error> + Send>,
        }
    }

    /// Lists the docker images on the current docker host
    pub fn list(
        &self,
        opts: &ImageListOptions,
    ) -> impl Future<Item = Vec<ImageRep>, Error = Error> {
        let mut path = vec!["/images/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        self.docker.get_json::<Vec<ImageRep>>(&path.join("?"))
    }

    /// Returns a reference to a set of operations available for a named image
    pub fn get<'b>(
        &self,
        name: &'b str,
    ) -> Image<'a, 'b> {
        Image::new(self.docker, name)
    }

    /// Search for docker images by term
    pub fn search(
        &self,
        term: &str,
    ) -> impl Future<Item = Vec<SearchResult>, Error = Error> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("term", term)
            .finish();
        self.docker
            .get_json::<Vec<SearchResult>>(&format!("/images/search?{}", query)[..])
    }

    /// Pull and create a new docker images from an existing image
    pub fn pull(
        &self,
        opts: &PullOptions,
    ) -> impl Stream<Item = Value, Error = Error> {
        let mut path = vec!["/images/create".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let headers = opts
            .auth_header()
            .map(|a| iter::once(("X-Registry-Auth", a)));
        self.docker
            .stream_post::<Body, _>(&path.join("?"), None, headers)
            // todo: give this a proper enum type
            .map(|r| {
                futures::stream::iter_result(
                    serde_json::Deserializer::from_slice(&r[..])
                        .into_iter::<Value>()
                        .collect::<Vec<_>>(),
                )
                .map_err(Error::from)
            })
            .flatten()
    }

    /// exports a collection of named images,
    /// either by name, name:tag, or image id, into a tarball
    pub fn export(
        &self,
        names: Vec<&str>,
    ) -> impl Stream<Item = Vec<u8>, Error = Error> {
        let params = names.iter().map(|n| ("names", *n));
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();
        self.docker
            .stream_get(&format!("/images/get?{}", query)[..])
            .map(|c| c.to_vec())
    }

    /// imports an image or set of images from a given tarball source
    /// source can be uncompressed on compressed via gzip, bzip2 or xz
    pub fn import(
        self,
        mut tarball: Box<dyn Read>,
    ) -> impl Stream<Item = Value, Error = Error> {
        let mut bytes = Vec::new();

        match tarball.read_to_end(&mut bytes) {
            Ok(_) => Box::new(
                self.docker
                    .stream_post(
                        "/images/load",
                        Some((Body::from(bytes), tar())),
                        None::<iter::Empty<_>>,
                    )
                    .and_then(|bytes| {
                        serde_json::from_slice::<'_, Value>(&bytes[..])
                            .map_err(Error::from)
                            .into_future()
                    }),
            ) as Box<dyn Stream<Item = Value, Error = Error> + Send>,
            Err(e) => Box::new(futures::future::err(Error::IO(e)).into_stream())
                as Box<dyn Stream<Item = Value, Error = Error> + Send>,
        }
    }
}

/// Interface for accessing and manipulating a docker container
pub struct Container<'a, 'b> {
    docker: &'a Docker,
    id: Cow<'b, str>,
}

impl<'a, 'b> Container<'a, 'b> {
    /// Exports an interface exposing operations against a container instance
    pub fn new<S>(
        docker: &'a Docker,
        id: S,
    ) -> Container<'a, 'b>
    where
        S: Into<Cow<'b, str>>,
    {
        Container {
            docker,
            id: id.into(),
        }
    }

    /// a getter for the container id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Inspects the current docker container instance's details
    pub fn inspect(&self) -> impl Future<Item = ContainerDetails, Error = Error> {
        self.docker
            .get_json::<ContainerDetails>(&format!("/containers/{}/json", self.id)[..])
    }

    /// Returns a `top` view of information about the container process
    pub fn top(
        &self,
        psargs: Option<&str>,
    ) -> impl Future<Item = Top, Error = Error> {
        let mut path = vec![format!("/containers/{}/top", self.id)];
        if let Some(ref args) = psargs {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("ps_args", args)
                .finish();
            path.push(encoded)
        }
        self.docker.get_json(&path.join("?"))
    }

    /// Returns a stream of logs emitted but the container instance
    pub fn logs(
        &self,
        opts: &LogsOptions,
    ) -> impl Stream<Item = tty::Chunk, Error = Error> {
        let mut path = vec![format!("/containers/{}/logs", self.id)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        let decoder = TtyDecoder::new();
        let chunk_stream = StreamReader::new(self.docker.stream_get(&path.join("?")));

        FramedRead::new(chunk_stream, decoder)
    }

    /// Attaches to a running container, returning a stream that can
    /// be used to interact with the standard IO streams.
    pub fn attach(&self) -> impl Future<Item = tty::Multiplexed, Error = Error> {
        self.docker.stream_post_upgrade_multiplexed::<Body>(
            &format!(
                "/containers/{}/attach?stream=1&stdout=1&stderr=1&stdin=1",
                self.id
            ),
            None,
        )
    }

    /// Attaches to a running container, returning a stream that can
    /// be used to interact with the standard IO streams.
    pub fn attach_blocking(&self) -> Result<tty::MultiplexedBlocking> {
        self.attach().map(|s| s.wait()).wait()
    }

    /// Returns a set of changes made to the container instance
    pub fn changes(&self) -> impl Future<Item = Vec<Change>, Error = Error> {
        self.docker
            .get_json::<Vec<Change>>(&format!("/containers/{}/changes", self.id)[..])
    }

    /// Exports the current docker container into a tarball
    pub fn export(&self) -> impl Stream<Item = Vec<u8>, Error = Error> {
        self.docker
            .stream_get(&format!("/containers/{}/export", self.id)[..])
            .map(|c| c.to_vec())
    }

    /// Returns a stream of stats specific to this container instance
    pub fn stats(&self) -> impl Stream<Item = Stats, Error = Error> {
        let decoder = LinesCodec::new();
        let stream_of_chunks = StreamReader::new(
            self.docker
                .stream_get(&format!("/containers/{}/stats", self.id)[..]),
        );

        FramedRead::new(stream_of_chunks, decoder)
            .map_err(Error::IO)
            .and_then(|s| {
                serde_json::from_str::<Stats>(&s)
                    .map_err(Error::SerdeJsonError)
                    .into_future()
            })
    }

    /// Start the container instance
    pub fn start(&self) -> impl Future<Item = (), Error = Error> {
        self.docker
            .post::<Body>(&format!("/containers/{}/start", self.id)[..], None)
            .map(|_| ())
    }

    /// Stop the container instance
    pub fn stop(
        &self,
        wait: Option<Duration>,
    ) -> impl Future<Item = (), Error = Error> {
        let mut path = vec![format!("/containers/{}/stop", self.id)];
        if let Some(w) = wait {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("t", &w.as_secs().to_string())
                .finish();

            path.push(encoded)
        }
        self.docker.post::<Body>(&path.join("?"), None).map(|_| ())
    }

    /// Restart the container instance
    pub fn restart(
        &self,
        wait: Option<Duration>,
    ) -> impl Future<Item = (), Error = Error> {
        let mut path = vec![format!("/containers/{}/restart", self.id)];
        if let Some(w) = wait {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("t", &w.as_secs().to_string())
                .finish();
            path.push(encoded)
        }
        self.docker.post::<Body>(&path.join("?"), None).map(|_| ())
    }

    /// Kill the container instance
    pub fn kill(
        &self,
        signal: Option<&str>,
    ) -> impl Future<Item = (), Error = Error> {
        let mut path = vec![format!("/containers/{}/kill", self.id)];
        if let Some(sig) = signal {
            let encoded = form_urlencoded::Serializer::new(String::new())
                .append_pair("signal", &sig.to_owned())
                .finish();
            path.push(encoded)
        }
        self.docker.post::<Body>(&path.join("?"), None).map(|_| ())
    }

    /// Rename the container instance
    pub fn rename(
        &self,
        name: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("name", name)
            .finish();
        self.docker
            .post::<Body>(
                &format!("/containers/{}/rename?{}", self.id, query)[..],
                None,
            )
            .map(|_| ())
    }

    /// Pause the container instance
    pub fn pause(&self) -> impl Future<Item = (), Error = Error> {
        self.docker
            .post::<Body>(&format!("/containers/{}/pause", self.id)[..], None)
            .map(|_| ())
    }

    /// Unpause the container instance
    pub fn unpause(&self) -> impl Future<Item = (), Error = Error> {
        self.docker
            .post::<Body>(&format!("/containers/{}/unpause", self.id)[..], None)
            .map(|_| ())
    }

    /// Wait until the container stops
    pub fn wait(&self) -> impl Future<Item = Exit, Error = Error> {
        self.docker
            .post_json::<Body, Exit>(&format!("/containers/{}/wait", self.id)[..], None)
    }

    /// Delete the container instance
    ///
    /// Use remove instead to use the force/v options.
    pub fn delete(&self) -> impl Future<Item = (), Error = Error> {
        self.docker
            .delete(&format!("/containers/{}", self.id)[..])
            .map(|_| ())
    }

    /// Delete the container instance (todo: force/v)
    pub fn remove(
        &self,
        opts: RmContainerOptions,
    ) -> impl Future<Item = (), Error = Error> {
        let mut path = vec![format!("/containers/{}", self.id)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker.delete(&path.join("?")).map(|_| ())
    }

    // TODO(abusch) fix this
    /// Exec the specified command in the container
    pub fn exec(
        &self,
        opts: &ExecContainerOptions,
    ) -> impl Stream<Item = tty::Chunk, Error = Error> {
        let data = opts.serialize().unwrap(); // TODO fixme
        let bytes = data.into_bytes();
        let docker2 = self.docker.clone();
        self.docker
            .post(
                &format!("/containers/{}/exec", self.id)[..],
                Some((bytes, mime::APPLICATION_JSON)),
            )
            .map(move |res| {
                let data = "{}";
                let bytes = data.as_bytes();
                let id = serde_json::from_str::<Value>(res.as_str())
                    .ok()
                    .and_then(|v| {
                        v.as_object()
                            .and_then(|v| v.get("Id"))
                            .and_then(|v| v.as_str().map(|v| v.to_string()))
                    })
                    .unwrap(); // TODO fixme

                let decoder = TtyDecoder::new();
                let chunk_stream = StreamReader::new(docker2.stream_post(
                    &format!("/exec/{}/start", id)[..],
                    Some((bytes, mime::APPLICATION_JSON)),
                    None::<iter::Empty<_>>,
                ));
                FramedRead::new(chunk_stream, decoder)
            })
            .flatten_stream()
    }

    /// Copy a file/folder from the container.  The resulting stream is a tarball of the extracted
    /// files.
    ///
    /// If `path` is not an absolute path, it is relative to the container’s root directory. The
    /// resource specified by `path` must exist. To assert that the resource is expected to be a
    /// directory, `path` should end in `/` or `/`. (assuming a path separator of `/`). If `path`
    /// ends in `/.`  then this indicates that only the contents of the path directory should be
    /// copied.  A symlink is always resolved to its target.
    pub fn copy_from(
        &self,
        path: &Path,
    ) -> impl Stream<Item = Vec<u8>, Error = Error> {
        let path_arg = form_urlencoded::Serializer::new(String::new())
            .append_pair("path", &path.to_string_lossy())
            .finish();
        self.docker
            .stream_get(&format!("/containers/{}/archive?{}", self.id, path_arg))
            .map(|c| c.to_vec())
    }

    /// Copy a byte slice as file into (see `bytes`) the container.
    ///
    /// The file will be copied at the given location (see `path`) and will be owned by root
    /// with access mask 644.
    pub fn copy_file_into<P: AsRef<Path>>(
        &self,
        path: P,
        bytes: &[u8],
    ) -> impl Future<Item = (), Error = Error> {
        let path = path.as_ref();

        let mut ar = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o0644);
        ar.append_data(
            &mut header,
            path.to_path_buf()
                .iter()
                .skip(1)
                .collect::<std::path::PathBuf>(),
            bytes,
        )
        .unwrap();
        let data = ar.into_inner().unwrap();

        let body = Some((data, "application/x-tar".parse::<Mime>().unwrap()));

        let path_arg = form_urlencoded::Serializer::new(String::new())
            .append_pair("path", "/")
            .finish();

        self.docker
            .put(
                &format!("/containers/{}/archive?{}", self.id, path_arg),
                body,
            )
            .map(|_| ())
    }
}

/// Interface for docker containers
pub struct Containers<'a> {
    docker: &'a Docker,
}

impl<'a> Containers<'a> {
    /// Exports an interface for interacting with docker containers
    pub fn new(docker: &'a Docker) -> Containers<'a> {
        Containers { docker }
    }

    /// Lists the container instances on the docker host
    pub fn list(
        &self,
        opts: &ContainerListOptions,
    ) -> impl Future<Item = Vec<ContainerRep>, Error = Error> {
        let mut path = vec!["/containers/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker.get_json::<Vec<ContainerRep>>(&path.join("?"))
    }

    /// Returns a reference to a set of operations available to a specific container instance
    pub fn get<'b>(
        &self,
        name: &'b str,
    ) -> Container<'a, 'b> {
        Container::new(self.docker, name)
    }

    /// Returns a builder interface for creating a new container instance
    pub fn create(
        &self,
        opts: &ContainerOptions,
    ) -> impl Future<Item = ContainerCreateInfo, Error = Error> {
        let data = match opts.serialize() {
            Ok(data) => data,
            Err(e) => return Either::A(futures::future::err(e)),
        };

        let bytes = data.into_bytes();
        let mut path = vec!["/containers/create".to_owned()];

        if let Some(ref name) = opts.name {
            path.push(
                form_urlencoded::Serializer::new(String::new())
                    .append_pair("name", name)
                    .finish(),
            );
        }

        Either::B(
            self.docker
                .post_json(&path.join("?"), Some((bytes, mime::APPLICATION_JSON))),
        )
    }
}

/// Interface for docker network
pub struct Networks<'a> {
    docker: &'a Docker,
}

impl<'a> Networks<'a> {
    /// Exports an interface for interacting with docker Networks
    pub fn new(docker: &'a Docker) -> Networks<'a> {
        Networks { docker }
    }

    /// List the docker networks on the current docker host
    pub fn list(
        &self,
        opts: &NetworkListOptions,
    ) -> impl Future<Item = Vec<NetworkInfo>, Error = Error> {
        let mut path = vec!["/networks".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        self.docker.get_json(&path.join("?"))
    }

    /// Returns a reference to a set of operations available to a specific network instance
    pub fn get<'b>(
        &self,
        id: &'b str,
    ) -> Network<'a, 'b> {
        Network::new(self.docker, id)
    }

    /// Create a new Network instance
    pub fn create(
        &self,
        opts: &NetworkCreateOptions,
    ) -> impl Future<Item = NetworkCreateInfo, Error = Error> {
        let data = match opts.serialize() {
            Ok(data) => data,
            Err(e) => return Either::A(futures::future::err(e)),
        };
        let bytes = data.into_bytes();
        let path = vec!["/networks/create".to_owned()];

        Either::B(
            self.docker
                .post_json(&path.join("?"), Some((bytes, mime::APPLICATION_JSON))),
        )
    }
}

/// Interface for accessing and manipulating a docker network
pub struct Network<'a, 'b> {
    docker: &'a Docker,
    id: Cow<'b, str>,
}

impl<'a, 'b> Network<'a, 'b> {
    /// Exports an interface exposing operations against a network instance
    pub fn new<S>(
        docker: &'a Docker,
        id: S,
    ) -> Network<'a, 'b>
    where
        S: Into<Cow<'b, str>>,
    {
        Network {
            docker,
            id: id.into(),
        }
    }

    /// a getter for the Network id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Inspects the current docker network instance's details
    pub fn inspect(&self) -> impl Future<Item = NetworkInfo, Error = Error> {
        self.docker.get_json(&format!("/networks/{}", self.id)[..])
    }

    /// Delete the network instance
    pub fn delete(&self) -> impl Future<Item = (), Error = Error> {
        self.docker
            .delete(&format!("/networks/{}", self.id)[..])
            .map(|_| ())
    }

    /// Connect container to network
    pub fn connect(
        &self,
        opts: &ContainerConnectionOptions,
    ) -> impl Future<Item = (), Error = Error> {
        self.do_connection("connect", opts)
    }

    /// Disconnect container to network
    pub fn disconnect(
        &self,
        opts: &ContainerConnectionOptions,
    ) -> impl Future<Item = (), Error = Error> {
        self.do_connection("disconnect", opts)
    }

    fn do_connection(
        &self,
        segment: &str,
        opts: &ContainerConnectionOptions,
    ) -> impl Future<Item = (), Error = Error> {
        let data = match opts.serialize() {
            Ok(data) => data,
            Err(e) => return Either::A(futures::future::err(e)),
        };
        let bytes = data.into_bytes();

        Either::B(
            self.docker
                .post(
                    &format!("/networks/{}/{}", self.id, segment)[..],
                    Some((bytes, mime::APPLICATION_JSON)),
                )
                .map(|_| ()),
        )
    }
}

/// Interface for docker volumes
pub struct Volumes<'a> {
    docker: &'a Docker,
}

impl<'a> Volumes<'a> {
    /// Exports an interface for interacting with docker volumes
    pub fn new(docker: &'a Docker) -> Volumes<'a> {
        Volumes { docker }
    }

    pub fn create(
        &self,
        opts: &VolumeCreateOptions,
    ) -> impl Future<Item = VolumeCreateInfo, Error = Error> {
        let data = match opts.serialize() {
            Ok(data) => data,
            Err(e) => return Either::A(futures::future::err(e)),
        };

        let bytes = data.into_bytes();
        let path = vec!["/volumes/create".to_owned()];

        Either::B(
            self.docker
                .post_json(&path.join("?"), Some((bytes, mime::APPLICATION_JSON))),
        )
    }

    /// Lists the docker volumes on the current docker host
    pub fn list(&self) -> impl Future<Item = Vec<VolumeRep>, Error = Error> {
        let path = vec!["/volumes".to_owned()];

        self.docker
            .get_json::<VolumesRep>(&path.join("?"))
            .map(|volumes: VolumesRep| match volumes.volumes {
                Some(volumes) => volumes,
                None => vec![],
            })
    }

    /// Returns a reference to a set of operations available for a named volume
    pub fn get<'b>(
        &self,
        name: &'b str,
    ) -> Volume<'a, 'b> {
        Volume::new(self.docker, name)
    }
}

/// Interface for accessing and manipulating a named docker volume
pub struct Volume<'a, 'b> {
    docker: &'a Docker,
    name: Cow<'b, str>,
}

impl<'a, 'b> Volume<'a, 'b> {
    /// Exports an interface for operations that may be performed against a named volume
    pub fn new<S>(
        docker: &'a Docker,
        name: S,
    ) -> Volume<'a, 'b>
    where
        S: Into<Cow<'b, str>>,
    {
        Volume {
            docker,
            name: name.into(),
        }
    }

    /// Deletes a volume
    pub fn delete(&self) -> impl Future<Item = (), Error = Error> {
        self.docker
            .delete(&format!("/volumes/{}", self.name)[..])
            .map(|_| ())
    }
}

fn get_http_connector() -> HttpConnector {
    let mut http = HttpConnector::new(1);
    http.enforce_http(false);

    http
}

#[cfg(feature = "tls")]
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
fn get_docker_for_tcp(tcp_host_str: String) -> Docker {
    let http = get_http_connector();
    Docker {
        transport: Transport::Tcp {
            client: Client::builder().build(http),
            host: tcp_host_str,
        },
    }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
    /// constructs a new Docker instance for a docker host listening at a url specified by an env var `DOCKER_HOST`,
    /// falling back on unix:///var/run/docker.sock
    pub fn new() -> Docker {
        match env::var("DOCKER_HOST").ok() {
            Some(host) => {
                let host = host.parse().expect("invalid url");
                Docker::host(host)
            }
            #[cfg(feature = "unix-socket")]
            None => Docker::unix("/var/run/docker.sock"),
            #[cfg(not(feature = "unix-socket"))]
            None => panic!("Unix socket support is disabled"),
        }
    }

    /// Creates a new docker instance for a docker host
    /// listening on a given Unix socket.
    #[cfg(feature = "unix-socket")]
    pub fn unix<S>(socket_path: S) -> Docker
    where
        S: Into<String>,
    {
        Docker {
            transport: Transport::Unix {
                client: Client::builder().keep_alive(false).build(UnixConnector),
                path: socket_path.into(),
            },
        }
    }

    /// constructs a new Docker instance for docker host listening at the given host url
    pub fn host(host: Uri) -> Docker {
        let tcp_host_str = format!(
            "{}://{}:{}",
            host.scheme_part().map(|s| s.as_str()).unwrap(),
            host.host().unwrap().to_owned(),
            host.port_u16().unwrap_or(80)
        );

        match host.scheme_part().map(|s| s.as_str()) {
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
    pub fn images(&self) -> Images {
        Images::new(self)
    }

    /// Exports an interface for interacting with docker containers
    pub fn containers(&self) -> Containers {
        Containers::new(self)
    }

    pub fn networks(&self) -> Networks {
        Networks::new(self)
    }

    pub fn volumes(&self) -> Volumes {
        Volumes::new(self)
    }

    /// Returns version information associated with the docker daemon
    pub fn version(&self) -> impl Future<Item = Version, Error = Error> {
        self.get_json("/version")
    }

    /// Returns information associated with the docker daemon
    pub fn info(&self) -> impl Future<Item = Info, Error = Error> {
        self.get_json("/info")
    }

    /// Returns a simple ping response indicating the docker daemon is accessible
    pub fn ping(&self) -> impl Future<Item = String, Error = Error> {
        self.get("/_ping")
    }

    /// Returns a stream of docker events
    pub fn events(
        &self,
        opts: &EventsOptions,
    ) -> impl Stream<Item = Event, Error = Error> {
        let mut path = vec!["/events".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let stream_of_chunks = self.stream_get(&path.join("?")[..]);
        let reader = StreamReader::new(stream_of_chunks);
        FramedRead::new(reader, LinesCodec::new())
            .map_err(Error::IO)
            .and_then(|line| serde_json::from_str::<Event>(&line).map_err(Error::from))
    }

    //
    // Utility functions to make requests
    //

    fn get(
        &self,
        endpoint: &str,
    ) -> impl Future<Item = String, Error = Error> {
        self.transport.request::<Body>(Method::GET, endpoint, None)
    }

    fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> impl Future<Item = T, Error = Error> {
        self.transport
            .request::<Body>(Method::GET, endpoint, None)
            .and_then(|v| {
                serde_json::from_str::<T>(&v)
                    .map_err(Error::SerdeJsonError)
                    .into_future()
            })
    }

    fn post<B>(
        &self,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Future<Item = String, Error = Error>
    where
        B: Into<Body>,
    {
        self.transport.request(Method::POST, endpoint, body)
    }

    fn put<B>(
        &self,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Future<Item = String, Error = Error>
    where
        B: Into<Body>,
    {
        self.transport.request(Method::PUT, endpoint, body)
    }

    fn post_json<B, T>(
        &self,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Future<Item = T, Error = Error>
    where
        B: Into<Body>,
        T: serde::de::DeserializeOwned,
    {
        self.transport
            .request(Method::POST, endpoint, body)
            .and_then(|v| {
                serde_json::from_str::<T>(&v)
                    .map_err(Error::SerdeJsonError)
                    .into_future()
            })
    }

    fn delete(
        &self,
        endpoint: &str,
    ) -> impl Future<Item = String, Error = Error> {
        self.transport
            .request::<Body>(Method::DELETE, endpoint, None)
    }

    fn delete_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> impl Future<Item = T, Error = Error> {
        self.transport
            .request::<Body>(Method::DELETE, endpoint, None)
            .and_then(|v| {
                serde_json::from_str::<T>(&v)
                    .map_err(Error::SerdeJsonError)
                    .into_future()
            })
    }

    fn stream_post<B, H>(
        &self,
        endpoint: &str,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = hyper::Chunk, Error = Error>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        self.transport
            .stream_chunks(Method::POST, endpoint, body, headers)
    }

    fn stream_get(
        &self,
        endpoint: &str,
    ) -> impl Stream<Item = hyper::Chunk, Error = Error> {
        self.transport
            .stream_chunks::<Body, iter::Empty<_>>(Method::GET, endpoint, None, None)
    }

    fn stream_post_upgrade_multiplexed<B>(
        &self,
        endpoint: &str,
        body: Option<(B, Mime)>,
    ) -> impl Future<Item = tty::Multiplexed, Error = Error>
    where
        B: Into<Body> + 'static,
    {
        self.transport
            .stream_upgrade_multiplexed(Method::POST, endpoint, body)
    }
}

impl Default for Docker {
    fn default() -> Self {
        Self::new()
    }
}
