extern crate shiplift;

use shiplift::{Docker, ExecContainerOptions};
use std::env;

fn main() {
    let docker = Docker::new().unwrap();
    let options = ExecContainerOptions::builder()
        .cmd(vec![
            "bash",
            "-c",
            "echo -n \"echo VAR=$VAR on stdout\"; echo -n \"echo VAR=$VAR on stderr\" >&2",
        ])
        .env(vec!["VAR=value"])
        .attach_stdout(true)
        .attach_stderr(true)
        .build();
    if let Some(id) = env::args().nth(1) {
        match docker.containers().get(&id).exec(&options) {
            Ok(res) => {
                println!("Stdout: {}", res.stdout);
                println!("Stderr: {}", res.stderr);
            }
            Err(err) => println!("An error occured: {:?}", err),
        }
    }
}
