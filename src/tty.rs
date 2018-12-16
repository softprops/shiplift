use byteorder::{BigEndian, ReadBytesExt};
use bytes::BytesMut;
use errors::Error;
use std::io::Cursor;
use tokio_codec::Decoder;

#[derive(Debug)]
pub struct TtyLine {
    pub stream_type: StreamType,
    pub data: String,
}

#[derive(Debug, Clone, Copy)]
pub enum StreamType {
    StdOut,
    StdErr,
}

/// Represent the current state of the decoding of a TTY frame
enum TtyDecoderState {
    /// We have yet to read a frame header
    WaitingHeader,
    /// We have read a header and extracted the payload size and stream type,
    /// and are now waiting to read the corresponding payload
    WaitingPayload(usize, StreamType),
}

pub struct TtyDecoder {
    state: TtyDecoderState,
}

impl TtyDecoder {
    pub fn new() -> Self {
        Self {
            state: TtyDecoderState::WaitingHeader,
        }
    }
}

impl Decoder for TtyDecoder {
    type Item = TtyLine;
    type Error = Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                TtyDecoderState::WaitingHeader => {
                    if src.len() < 8 {
                        trace!("Not enough data to read a header");
                        return Ok(None);
                    } else {
                        trace!("Reading header");
                        let header_bytes = src.split_to(8);
                        let payload_size: Vec<u8> = header_bytes[4..8].to_vec();
                        let stream_type = match header_bytes[0] {
                            0 => {
                                return Err(Error::InvalidResponse(
                                    "Unsupported stream of type stdin".to_string(),
                                ));
                            }
                            1 => StreamType::StdOut,
                            2 => StreamType::StdErr,
                            n => {
                                return Err(Error::InvalidResponse(format!(
                                    "Unsupported stream of type {}",
                                    n
                                )));
                            }
                        };

                        let length =
                            Cursor::new(&payload_size).read_u32::<BigEndian>().unwrap() as usize;
                        trace!(
                            "Read header: length = {}, line_type = {:?}",
                            length,
                            stream_type
                        );
                        // We've successfully read a header, now we wait for the payload
                        self.state = TtyDecoderState::WaitingPayload(length, stream_type);
                        continue;
                    }
                }
                TtyDecoderState::WaitingPayload(len, stream_type) => {
                    if src.len() < len {
                        trace!(
                            "Not enough data to read payload. Need {} but only {} available",
                            len,
                            src.len()
                        );
                        return Ok(None);
                    } else {
                        trace!("Reading payload");
                        let payload = src.split_to(len);
                        let data = String::from_utf8_lossy(&payload).trim().to_string();
                        let tty_line = TtyLine { stream_type, data };

                        // We've successfully read a full frame, now we go back to waiting for the next
                        // header
                        self.state = TtyDecoderState::WaitingHeader;
                        return Ok(Some(tty_line));
                    }
                }
            }
        }
    }
}
