use crate::{Error, Result};
use bytes::{BigEndian, ByteOrder};
use futures_util::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    stream::{Stream, TryStreamExt},
};
use pin_project::pin_project;
use std::io;

#[derive(Debug, Clone)]
pub enum TtyChunk {
    StdIn(Vec<u8>),
    StdOut(Vec<u8>),
    StdErr(Vec<u8>),
}

async fn decode_chunk<S>(mut stream: S) -> Option<(Result<TtyChunk>, S)>
where
    S: AsyncRead + Unpin,
{
    let mut header_bytes = vec![0u8; 8];

    match stream.read_exact(&mut header_bytes).await {
        Err(e) if e.kind() == futures_util::io::ErrorKind::UnexpectedEof => return None,
        Err(e) => return Some((Err(Error::IO(e)), stream)),
        _ => (),
    }

    let size_bytes = &header_bytes[4..];
    let data_length = BigEndian::read_u32(size_bytes);

    let mut data = vec![0u8; data_length as usize];

    if stream.read_exact(&mut data).await.is_err() {
        return None;
    }

    let chunk = match header_bytes[0] {
        0 => TtyChunk::StdIn(data),
        1 => TtyChunk::StdOut(data),
        2 => TtyChunk::StdErr(data),
        n => panic!("invalid stream number from docker daemon: '{}'", n),
    };

    Some((Ok(chunk), stream))
}

pub fn decode<S>(hyper_chunk_stream: S) -> impl Stream<Item = Result<TtyChunk>>
where
    S: Stream<Item = Result<hyper::Chunk>> + Unpin,
{
    let stream = hyper_chunk_stream
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .into_async_read();

    futures_util::stream::unfold(stream, decode_chunk)
}

type TtyReader<'a> = Pin<Box<dyn Stream<Item = Result<TtyChunk>> + 'a>>;
type TtyWriter<'a> = Pin<Box<dyn AsyncWrite + 'a>>;

/// TTY multiplexer returned by the `attach` method.
///
/// This object can emit a stream of `[TtyChunk]`s and also implements AsyncRead for streaming bytes to Stdin.
#[pin_project]
pub struct Multiplexer<'a> {
    #[pin]
    reader: TtyReader<'a>,
    #[pin]
    writer: TtyWriter<'a>,
}

impl<'a> Multiplexer<'a> {
    pub(crate) fn new<T>(tcp_connection: T) -> Self
    where
        T: AsyncRead + AsyncWrite + 'a,
    {
        let (reader, writer) = tcp_connection.split();

        Self {
            reader: Box::pin(futures_util::stream::unfold(reader, |reader| {
                decode_chunk(reader)
            })),
            writer: Box::pin(writer),
        }
    }
}

use std::{
    pin::Pin,
    task::{Context, Poll},
};

impl<'a> Stream for Multiplexer<'a> {
    type Item = Result<TtyChunk>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.project().reader.poll_next(cx)
    }
}

impl<'a> AsyncWrite for Multiplexer<'a> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().writer.poll_write(cx, buf)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().writer.poll_flush(cx)
    }
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().writer.poll_close(cx)
    }
}

impl<'a> Multiplexer<'a> {
    pub fn split(
        self
    ) -> (
        impl Stream<Item = Result<TtyChunk>> + 'a,
        impl AsyncWrite + 'a,
    ) {
        (self.reader, self.writer)
    }
}
