use std::io::Read;
pub struct Tty {
    pub stdout: String,
    pub stderr: String,
}

impl Tty {
    pub fn new(mut stream: Box<Read>) -> Tty {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        loop {
            let mut header = [0; 8];
            match stream.read(&mut header) {
                Ok(0) => break,
                Ok(_) => {
                    let mut body: Vec<u8> = vec![0; header[7] as usize];
                    if let Ok(_) = stream.read(&mut body) {
                        if header[0] == 1 {
                            stdout.append(&mut body);
                        }
                        if header[0] == 2 {
                            stderr.append(&mut body);
                        }
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
