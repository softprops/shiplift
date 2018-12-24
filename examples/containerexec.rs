use shiplift::{tty::StreamType, Docker, ExecContainerOptions};
use std::env;
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a container id");

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
    let fut = docker
        .containers()
        .get(&id)
        .exec(&options)
        .for_each(|chunk| {
            match chunk.stream_type {
                StreamType::StdOut => println!("Stdout: {}", chunk.as_string_lossy()),
                StreamType::StdErr => eprintln!("Stderr: {}", chunk.as_string_lossy()),
                StreamType::StdIn => unreachable!(),
            }
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
