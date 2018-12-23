use crate::errors::Error;
use futures::{Async, Stream};
use hyper::Chunk;
use std::{
    cmp,
    io::{self, Read},
};
use tokio_io::AsyncRead;

/*
 * The following is taken from
 * https://github.com/ferristseng/rust-ipfs-api/blob/master/ipfs-api/src/read.rs.
 * TODO: see with upstream author to move to a separate crate.
 */

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
///
pub struct StreamReader<S> {
    stream: S,
    state: ReadState,
}

impl<S> StreamReader<S>
where
    S: Stream<Item = Chunk, Error = Error>,
{
    #[inline]
    pub fn new(stream: S) -> StreamReader<S> {
        StreamReader {
            stream,
            state: ReadState::NotReady,
        }
    }
}

impl<S> Read for StreamReader<S>
where
    S: Stream<Item = Chunk, Error = Error>,
{
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> io::Result<usize> {
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
                        return Ok(len);
                    }
                }
                // Stream is not ready, and a Chunk needs to be read.
                //
                ReadState::NotReady => {
                    match self.stream.poll() {
                        // Polling stream yielded a Chunk that can be read from.
                        //
                        Ok(Async::Ready(Some(chunk))) => {
                            self.state = ReadState::Ready(chunk, 0);

                            continue;
                        }
                        // Polling stream yielded EOF.
                        //
                        Ok(Async::Ready(None)) => return Ok(0),
                        // Stream could not be read from.
                        //
                        Ok(Async::NotReady) => return Err(io::ErrorKind::WouldBlock.into()),
                        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
                    }
                }
            }

            self.state = ReadState::NotReady;

            return Ok(ret);
        }
    }
}

impl<S> AsyncRead for StreamReader<S> where S: Stream<Item = Chunk, Error = Error> {}
