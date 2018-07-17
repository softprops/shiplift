use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::io::Read;

pub struct Tty {
    pub stdout: String,
    pub stderr: String,
}

// https://docs.docker.com/engine/api/v1.26/#operation/ContainerAttach
impl Tty {
    pub fn new(mut stream: Box<Read>) -> Tty {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
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
                                1 => stdout.append(&mut buffer),
                                // stderr
                                2 => stderr.append(&mut buffer),
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
        Tty {
            stdout: String::from_utf8_lossy(&stdout).to_string(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
        }
    }
}
