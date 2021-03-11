//! Shiplift is a multi-transport utility for maneuvering [docker](https://www.docker.com/) containers
//!
//! # examples
//!
//! ```no_run
//! use shiplift::Docker;
//! # async {
//! let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
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

pub mod errors;
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

#[cfg(feature = "chrono")]
mod datetime;

pub use hyper::Uri;

pub use crate::{
    container::{
        Container, ContainerFilter, ContainerListOptions, ContainerOptions, Containers,
        LogsOptions, RmContainerOptions,
    },
    docker::{Docker, EventsOptions},
    errors::{Error, Result},
    exec::{Exec, ExecContainerOptions, ExecResizeOptions},
    image::{
        BuildOptions, Image, ImageFilter, ImageListOptions, Images, PullOptions, RegistryAuth,
        TagOptions,
    },
    network::{
        ContainerConnectionOptions, Network, NetworkCreateOptions, NetworkListOptions, Networks,
    },
    service::{Service, ServiceFilter, ServiceListOptions, ServiceOptions, Services},
    transport::Transport,
    volume::{Volume, VolumeCreateOptions, Volumes},
};

macro_rules! reexport {
    (@alias $module:ident :: $item:ident) => {
        reexport!(@alias $module::$item as $item);
    };
    (@alias $module:ident :: $item:ident as $old_item:ident) => {
        ::paste::paste! {
            #[deprecated(
                since = "0.8.0",
                note = "Please use `shiplift::" $module "::" $item "`. "
                       "This will be removed in 0.9.0."
            )]
            pub type $old_item = $crate::$module::$item;
        }
    };
    (mod $old_module:ident; $( $module:ident :: {$( $item:ident $(as $old_item:ident)? ),*$(,)?} ; )*) => {
        #[deprecated(
            since = "0.8.0",
            note = "Please use `shiplift::{container, docker, exec, image, network, service, volume}` as appropriate. This will be removed in 0.9.0."
        )]
        pub mod $old_module {
            $($( reexport!(@alias $module::$item $(as $old_item)?); )*)*
        }
    };
}

reexport! {
    mod builder;
    container::{
        ContainerListOptions, ContainerListOptionsBuilder, ContainerOptions,
        ContainerOptionsBuilder, LogsOptions, LogsOptionsBuilder, RmContainerOptions,
        RmContainerOptionsBuilder,
    };
    docker::{EventsOptions, EventFilterType, EventFilter, EventsOptionsBuilder};
    exec::{
        ExecContainerOptions, ExecContainerOptionsBuilder, ExecResizeOptions,
        ExecResizeOptionsBuilder,
    };
    image::{
        RegistryAuth, RegistryAuthBuilder, TagOptions, TagOptionsBuilder, PullOptions,
        PullOptionsBuilder, BuildOptions, BuildOptionsBuilder, ImageFilter, ImageListOptions,
        ImageListOptionsBuilder,
    };
    network::{
        NetworkListOptions, NetworkCreateOptions, NetworkCreateOptionsBuilder,
        ContainerConnectionOptions, ContainerConnectionOptionsBuilder,
    };
    service::{
        ServiceListOptions, ServiceFilter, ServiceListOptionsBuilder, ServiceOptions,
        ServiceOptionsBuilder,
    };
    volume::{VolumeCreateOptions, VolumeCreateOptionsBuilder};
}

reexport! {
    mod rep;
    container::{
        ContainerInfo as Container, ContainerDetails, Mount, State, HostConfig, Port, Stats,
        MemoryStats, MemoryStat, CpuStats, CpuUsage, ThrottlingData, BlkioStats, BlkioStat, Change,
        Top, ContainerCreateInfo, Exit,
    };
    docker::{Version, Info, Event, Actor};
    exec::{ExecDetails, ProcessConfig};
    image::{
        SearchResult, ImageInfo as Image, ImageDetails, Config, History, Status,
    };
    network::{
        NetworkSettings, NetworkEntry, NetworkInfo as Network, IPAM, NetworkDetails,
        NetworkContainerDetails, NetworkCreateInfo,
    };
    service::{
        ServicesInfo as Services, ServiceInfo as Service, ObjectVersion, Endpoint, EndpointSpec,
        EndpointPortConfig, UpdateStatus, ServiceStatus, JobStatus, ServiceDetails, ServiceSpec,
        TaskSpec, Mode, Replicated, ReplicatedJob, UpdateConfig, RollbackConfig,
        NetworkAttachmentConfig, ServiceCreateInfo,
    };
    volume::{VolumeCreateInfo, VolumesInfo as Volumes, VolumeInfo as Volume};
}
