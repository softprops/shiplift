
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::io::Read;

#[derive(Debug)]
pub enum TtyLine {
    StdOut(String),
    StdErr(String),
}

pub struct Tty {
    pub stdout: String,
    pub stderr: String,
}

// https://docs.docker.com/engine/api/v1.26/#operation/ContainerAttach
impl Tty {
    pub fn new(stream: impl Read) -> Tty {
        let mut stdout: Vec<String> = vec![];
        let mut stderr: Vec<String> = vec![];

        let lines = demux(stream);
        for line in lines {
            match line {
                TtyLine::StdOut(s) => stdout.push(s),
                TtyLine::StdErr(s) => stderr.push(s),
            }
        }

        Tty {
            stdout: stdout.concat(),
            stderr: stderr.concat(),
        }
    }
}

/// Used to demux the output of Docker log, but still keep lines from stdout and stderr interlaced
/// in the right order.
pub struct InterlacedTty {
    pub lines: Vec<TtyLine>,
}

// https://docs.docker.com/engine/api/v1.26/#operation/ContainerAttach
impl InterlacedTty {
    pub fn new(stream: impl Read) -> InterlacedTty {
        let lines = demux(stream);

        InterlacedTty { lines }
    }
}

fn demux(mut stream: impl Read) -> Vec<TtyLine> {
    let mut lines: Vec<TtyLine> = vec![];
    loop {
        // 8 byte header [ STREAM_TYPE, 0, 0, 0, SIZE1, SIZE2, SIZE3, SIZE4 ]
        let mut header = [0; 8];
        match stream.read_exact(&mut header) {
            Ok(_) => {
                let payload_size: Vec<u8> = header[4..8].to_vec();
                let mut buffer = vec![
                    0;
                    Cursor::new(&payload_size).read_u32::<BigEndian>().unwrap()
                        as usize
                ];
                match stream.read_exact(&mut buffer) {
                    Ok(_) => {
                        match header[0] {
                            // stdin, unhandled
                            0 => break,
                            // stdout
                            1 => lines.push(TtyLine::StdOut(
                                String::from_utf8_lossy(&buffer).trim().to_string(),
                            )),
                            // stderr
                            2 => lines.push(TtyLine::StdErr(
                                String::from_utf8_lossy(&buffer).trim().to_string(),
                            )),
                            //unhandled
                            _ => break,
                        }
                    }
                    Err(_) => break,
                };
            }
            Err(_) => break,
        }
    }
    lines
}
