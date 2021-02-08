//! Create and manage images.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Image>

use std::{io::Read, iter};

use futures_util::{stream::Stream, TryFutureExt, TryStreamExt};
use hyper::Body;
use serde_json::Value;
use url::form_urlencoded;

use crate::{
    builder::{BuildOptions, ImageListOptions, PullOptions, TagOptions},
    errors::Result,
    rep::{History, Image as ImageRep, ImageDetails, SearchResult, Status},
    tarball,
    transport::tar,
};

use crate::Docker;

/// Interface for accessing and manipulating a named docker image
pub struct Image<'docker> {
    docker: &'docker Docker,
    name: String,
}

impl<'docker> Image<'docker> {
    /// Exports an interface for operations that may be performed against a named image
    pub fn new<S>(
        docker: &'docker Docker,
        name: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Image {
            docker,
            name: name.into(),
        }
    }

    /// Inspects a named image's details
    pub async fn inspect(&self) -> Result<ImageDetails> {
        self.docker
            .get_json(&format!("/images/{}/json", self.name)[..])
            .await
    }

    /// Lists the history of the images set of changes
    pub async fn history(&self) -> Result<Vec<History>> {
        self.docker
            .get_json(&format!("/images/{}/history", self.name)[..])
            .await
    }

    /// Deletes an image
    pub async fn delete(&self) -> Result<Vec<Status>> {
        self.docker
            .delete_json::<Vec<Status>>(&format!("/images/{}", self.name)[..])
            .await
    }

    /// Export this image to a tarball
    pub fn export(&self) -> impl Stream<Item = Result<Vec<u8>>> + Unpin + 'docker {
        Box::pin(
            self.docker
                .stream_get(format!("/images/{}/get", self.name))
                .map_ok(|c| c.to_vec()),
        )
    }

    /// Adds a tag to an image
    pub async fn tag(
        &self,
        opts: &TagOptions,
    ) -> Result<()> {
        let mut path = vec![format!("/images/{}/tag", self.name)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        let _ = self.docker.post(&path.join("?"), None).await?;
        Ok(())
    }
}

/// Interface for docker images
pub struct Images<'docker> {
    docker: &'docker Docker,
}

impl<'docker> Images<'docker> {
    /// Exports an interface for interacting with docker images
    pub fn new(docker: &'docker Docker) -> Self {
        Images { docker }
    }

    /// Builds a new image build by reading a Dockerfile in a target directory
    pub fn build(
        &self,
        opts: &BuildOptions,
    ) -> impl Stream<Item = Result<Value>> + Unpin + 'docker {
        let mut endpoint = vec!["/build".to_owned()];
        if let Some(query) = opts.serialize() {
            endpoint.push(query)
        }

        // To not tie the lifetime of `opts` to the 'stream, we do the tarring work outside of the
        // stream. But for backwards compatability, we have to return the error inside of the
        // stream.
        let mut bytes = Vec::default();
        let tar_result = tarball::dir(&mut bytes, opts.path.as_str());

        // We must take ownership of the Docker reference. If we don't then the lifetime of 'stream
        // is incorrectly tied to `self`.
        let docker = self.docker;
        Box::pin(
            async move {
                // Bubble up error inside the stream for backwards compatability
                tar_result?;

                let value_stream = docker.stream_post_into_values(
                    endpoint.join("?"),
                    Some((Body::from(bytes), tar())),
                    None::<iter::Empty<_>>,
                );

                Ok(value_stream)
            }
            .try_flatten_stream(),
        )
    }

    /// Lists the docker images on the current docker host
    pub async fn list(
        &self,
        opts: &ImageListOptions,
    ) -> Result<Vec<ImageRep>> {
        let mut path = vec!["/images/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        self.docker.get_json::<Vec<ImageRep>>(&path.join("?")).await
    }

    /// Returns a reference to a set of operations available for a named image
    pub fn get<S>(
        &self,
        name: S,
    ) -> Image<'docker>
    where
        S: Into<String>,
    {
        Image::new(self.docker, name)
    }

    /// Search for docker images by term
    pub async fn search(
        &self,
        term: &str,
    ) -> Result<Vec<SearchResult>> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("term", term)
            .finish();
        self.docker
            .get_json::<Vec<SearchResult>>(&format!("/images/search?{}", query)[..])
            .await
    }

    /// Pull and create a new docker images from an existing image
    pub fn pull(
        &self,
        opts: &PullOptions,
    ) -> impl Stream<Item = Result<Value>> + Unpin + 'docker {
        let mut path = vec!["/images/create".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let headers = opts
            .auth_header()
            .map(|a| iter::once(("X-Registry-Auth", a)));

        Box::pin(
            self.docker
                .stream_post_into_values(path.join("?"), None, headers),
        )
    }

    /// exports a collection of named images,
    /// either by name, name:tag, or image id, into a tarball
    pub fn export(
        &self,
        names: Vec<&str>,
    ) -> impl Stream<Item = Result<Vec<u8>>> + 'docker {
        let params = names.iter().map(|n| ("names", *n));
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();
        self.docker
            .stream_get(format!("/images/get?{}", query))
            .map_ok(|c| c.to_vec())
    }

    /// imports an image or set of images from a given tarball source
    /// source can be uncompressed on compressed via gzip, bzip2 or xz
    pub fn import<R>(
        self,
        mut tarball: R,
    ) -> impl Stream<Item = Result<Value>> + Unpin + 'docker
    where
        R: Read + Send + 'docker,
    {
        Box::pin(
            async move {
                let mut bytes = Vec::default();

                tarball.read_to_end(&mut bytes)?;

                let value_stream = self.docker.stream_post_into_values(
                    "/images/load",
                    Some((Body::from(bytes), tar())),
                    None::<iter::Empty<_>>,
                );
                Ok(value_stream)
            }
            .try_flatten_stream(),
        )
    }
}
