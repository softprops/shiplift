use crate::{Error, Result};
use bytes::{BigEndian, ByteOrder};
use futures_util::{
    io::{AsyncRead, AsyncReadExt},
    stream::{Stream, TryStreamExt},
};
use std::io;

#[derive(Debug, Clone)]
pub enum TtyChunk {
    StdIn(Vec<u8>),
    StdOut(Vec<u8>),
    StdErr(Vec<u8>),
}

async fn chunk<S>(mut stream: S) -> Option<(Result<TtyChunk>, S)>
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

    if let Err(_) = stream.read_exact(&mut data).await {
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

pub fn chunks<S>(stream: S) -> impl Stream<Item = Result<TtyChunk>> + Unpin
where
    S: Stream<Item = Result<hyper::Chunk>> + Unpin,
{
    let stream = stream
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .into_async_read();

    Box::pin(futures_util::stream::unfold(stream, |stream| chunk(stream)))
}
