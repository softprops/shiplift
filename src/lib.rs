//! Shiplift is a multi-transport utility for maneuvering [docker](https://www.docker.com/) containers
//!
//! # examples
//!
//! ```no_run
//! # async {
//! let docker = shiplift::Docker::new();
//!
//! match docker.images().list(&Default::default()).await {
//!     Ok(images) => {
//!         for image in images {
//!             println!("{:?}", image.repo_tags);
//!         }
//!     },
//!     Err(e) => eprintln!("Something bad happened! {}", e),
//! }
//! # };
//! ```

pub mod builder;
pub mod errors;
pub mod rep;
pub mod transport;
pub mod tty;

pub mod container;
pub mod docker;
pub mod exec;
pub mod image;
pub mod network;
pub mod service;
pub mod volume;

mod tarball;

pub use hyper::Uri;

pub use crate::{
    builder::{
        BuildOptions, ContainerConnectionOptions, ContainerFilter, ContainerListOptions,
        ContainerOptions, EventsOptions, ExecContainerOptions, ExecResizeOptions, ImageFilter,
        ImageListOptions, LogsOptions, NetworkCreateOptions, NetworkListOptions, PullOptions,
        RegistryAuth, RmContainerOptions, ServiceFilter, ServiceListOptions, ServiceOptions,
        TagOptions, VolumeCreateOptions,
    },
    container::{Container, Containers},
    docker::Docker,
    errors::{Error, Result},
    exec::Exec,
    image::{Image, Images},
    network::{Network, Networks},
    service::{Service, Services},
    volume::{Volume, Volumes},
};
