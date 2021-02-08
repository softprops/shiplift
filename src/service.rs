//! Manage and inspect services within a swarm.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Service>

use std::iter;

use futures_util::stream::Stream;
use hyper::Body;

use crate::{
    builder::{ServiceListOptions, ServiceOptions},
    errors::Result,
    rep::{Service as ServiceInfo, ServiceCreateInfo, ServiceDetails},
    tty, Docker, LogsOptions,
};

/// Interface for docker services
pub struct Services<'docker> {
    docker: &'docker Docker,
}

impl<'docker> Services<'docker> {
    /// Exports an interface for interacting with docker services
    pub fn new(docker: &'docker Docker) -> Self {
        Services { docker }
    }

    /// Lists the docker services on the current docker host
    pub async fn list(
        &self,
        opts: &ServiceListOptions,
    ) -> Result<Vec<ServiceInfo>> {
        let mut path = vec!["/services".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }

        self.docker
            .get_json::<Vec<ServiceInfo>>(&path.join("?"))
            .await
    }

    /// Returns a reference to a set of operations available for a named service
    pub fn get(
        &self,
        name: &str,
    ) -> Service<'docker> {
        Service::new(self.docker, name)
    }
}

/// Interface for accessing and manipulating a named docker volume
pub struct Service<'docker> {
    docker: &'docker Docker,
    name: String,
}

impl<'docker> Service<'docker> {
    /// Exports an interface for operations that may be performed against a named service
    pub fn new<S>(
        docker: &'docker Docker,
        name: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Service {
            docker,
            name: name.into(),
        }
    }

    /// Creates a new service from ServiceOptions
    pub async fn create(
        &self,
        opts: &ServiceOptions,
    ) -> Result<ServiceCreateInfo> {
        let body: Body = opts.serialize()?.into();
        let path = vec!["/service/create".to_owned()];

        let headers = opts
            .auth_header()
            .map(|a| iter::once(("X-Registry-Auth", a)));

        self.docker
            .post_json_headers(
                &path.join("?"),
                Some((body, mime::APPLICATION_JSON)),
                headers,
            )
            .await
    }

    /// Inspects a named service's details
    pub async fn inspect(&self) -> Result<ServiceDetails> {
        self.docker
            .get_json(&format!("/services/{}", self.name)[..])
            .await
    }

    /// Deletes a service
    pub async fn delete(&self) -> Result<()> {
        self.docker
            .delete_json(&format!("/services/{}", self.name)[..])
            .await
    }

    /// Returns a stream of logs from a service
    pub fn logs(
        &self,
        opts: &LogsOptions,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + Unpin + 'docker {
        let mut path = vec![format!("/services/{}/logs", self.name)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        let stream = Box::pin(self.docker.stream_get(path.join("?")));

        Box::pin(tty::decode(stream))
    }
}
