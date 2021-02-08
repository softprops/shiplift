//! Run new commands inside running containers.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Exec>

use std::iter;

use futures_util::{stream::Stream, TryFutureExt};
use hyper::Body;

use crate::{
    builder::{ExecContainerOptions, ExecResizeOptions},
    errors::Result,
    rep::ExecDetails,
    tty, Docker,
};

/// Interface for docker exec instance
pub struct Exec<'docker> {
    docker: &'docker Docker,
    id: String,
}

impl<'docker> Exec<'docker> {
    fn new<S>(
        docker: &'docker Docker,
        id: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Exec {
            docker,
            id: id.into(),
        }
    }

    /// Creates a new exec instance that will be executed in a container with id == container_id
    pub async fn create(
        docker: &'docker Docker,
        container_id: &str,
        opts: &ExecContainerOptions,
    ) -> Result<Exec<'docker>> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Response {
            id: String,
        }

        let body: Body = opts.serialize()?.into();

        let id = docker
            .post_json(
                &format!("/containers/{}/exec", container_id),
                Some((body, mime::APPLICATION_JSON)),
            )
            .await
            .map(|resp: Response| resp.id)?;

        Ok(Exec::new(docker, id))
    }

    // This exists for Container::exec()
    //
    // We need to combine `Exec::create` and `Exec::start` into one method because otherwise you
    // needlessly tie the Stream to the lifetime of `container_id` and `opts`. This is because
    // `Exec::create` is async so it must occur inside of the `async move` block. However, this
    // means that `container_id` and `opts` are both expected to be alive in the returned stream
    // because we can't do the work of creating an endpoint from `container_id` or serializing
    // `opts`. By doing this work outside of the stream, we get owned values that we can then move
    // into the stream and have the lifetimes work out as you would expect.
    //
    // Yes, it is sad that we can't do the easy method and thus have some duplicated code.
    pub(crate) fn create_and_start(
        docker: &'docker Docker,
        container_id: &str,
        opts: &ExecContainerOptions,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + Unpin + 'docker {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Response {
            id: String,
        }

        // To not tie the lifetime of `opts` to the stream, we do the serializing work outside of
        // the stream. But for backwards compatability, we have to return the error inside of the
        // stream.
        let body_result = opts.serialize();

        // To not tie the lifetime of `container_id` to the stream, we convert it to an (owned)
        // endpoint outside of the stream.
        let container_endpoint = format!("/containers/{}/exec", container_id);

        Box::pin(
            async move {
                // Bubble up the error inside the stream for backwards compatability
                let body: Body = body_result?.into();

                let exec_id = docker
                    .post_json(&container_endpoint, Some((body, mime::APPLICATION_JSON)))
                    .await
                    .map(|resp: Response| resp.id)?;

                let stream = Box::pin(docker.stream_post(
                    format!("/exec/{}/start", exec_id),
                    Some(("{}".into(), mime::APPLICATION_JSON)),
                    None::<iter::Empty<_>>,
                ));

                Ok(tty::decode(stream))
            }
            .try_flatten_stream(),
        )
    }

    /// Get a reference to a set of operations available to an already created exec instance.
    ///
    /// It's in callers responsibility to ensure that exec instance with specified id actually
    /// exists. Use [Exec::create](Exec::create) to ensure that the exec instance is created
    /// beforehand.
    pub async fn get<S>(
        docker: &'docker Docker,
        id: S,
    ) -> Exec<'docker>
    where
        S: Into<String>,
    {
        Exec::new(docker, id)
    }

    /// Starts this exec instance returning a multiplexed tty stream
    pub fn start(&self) -> impl Stream<Item = Result<tty::TtyChunk>> + 'docker {
        // We must take ownership of the docker reference to not needlessly tie the stream to the
        // lifetime of `self`.
        let docker = self.docker;
        // We convert `self.id` into the (owned) endpoint outside of the stream to not needlessly
        // tie the stream to the lifetime of `self`.
        let endpoint = format!("/exec/{}/start", &self.id);
        Box::pin(
            async move {
                let stream = Box::pin(docker.stream_post(
                    endpoint,
                    Some(("{}".into(), mime::APPLICATION_JSON)),
                    None::<iter::Empty<_>>,
                ));

                Ok(tty::decode(stream))
            }
            .try_flatten_stream(),
        )
    }

    /// Inspect this exec instance to aquire detailed information
    pub async fn inspect(&self) -> Result<ExecDetails> {
        self.docker
            .get_json(&format!("/exec/{}/json", &self.id)[..])
            .await
    }

    pub async fn resize(
        &self,
        opts: &ExecResizeOptions,
    ) -> Result<()> {
        let body: Body = opts.serialize()?.into();

        self.docker
            .post_json(
                &format!("/exec/{}/resize", &self.id)[..],
                Some((body, mime::APPLICATION_JSON)),
            )
            .await
    }
}
