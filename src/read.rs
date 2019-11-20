use crate::Result;
use futures::Stream;
use hyper::Chunk;
use std::{
    cmp, io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_io::AsyncRead;

/// The state of a stream returning Chunks.
///
enum ReadState {
    /// A chunk is ready to be read from.
    ///
    Ready(Chunk, usize),

    /// The next chunk isn't ready yet.
    ///
    NotReady,
}

/// Reads from a stream of chunks asynchronously.
pub struct StreamReader<S> {
    stream: S,
    state: ReadState,
}

impl<S> StreamReader<S>
where
    S: Stream<Item = Result<Chunk>>,
{
    #[inline]
    pub fn new(stream: S) -> StreamReader<S> {
        StreamReader {
            stream,
            state: ReadState::NotReady,
        }
    }
}

impl<S> AsyncRead for StreamReader<S>
where
    S: Stream<Item = Result<Chunk>>,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.project();

        loop {
            let ret;

            match self.state {
                // Stream yielded a Chunk to read.
                //
                ReadState::Ready(ref mut chunk, ref mut pos) => {
                    let chunk_start = *pos;
                    let len = cmp::min(buf.len(), chunk.len() - chunk_start);
                    let chunk_end = chunk_start + len;

                    buf[..len].copy_from_slice(&chunk[chunk_start..chunk_end]);
                    *pos += len;

                    if *pos == chunk.len() {
                        ret = len;
                    } else {
                        return Poll::Ready(Ok(len));
                    }
                }
                // Stream is not ready, and a Chunk needs to be read.
                //
                ReadState::NotReady => {
                    match this.stream.poll_next(cx) {
                        // Polling stream yielded a Chunk that can be read from.
                        //
                        Poll::Ready(Some(Ok(chunk))) => {
                            self.state = ReadState::Ready(chunk, 0);

                            continue;
                        }
                        // Polling stream yielded EOF.
                        //
                        Poll::Ready(None) => return Poll::Ready(Ok(0)),
                        // Stream could not be read from.
                        //
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Some(Err(e))) => {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::Other,
                                e.to_string(),
                            )))
                        }
                    }
                }
            }

            self.state = ReadState::NotReady;

            return Poll::Ready(Ok(ret));
        }
    }
}
