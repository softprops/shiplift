use futures::StreamExt;
use shiplift::{tty::TtyChunk, Docker, LogsOptions};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a container id");

    let containers = docker.containers();

    let mut logs_stream = containers
        .get(&id)
        .logs(&LogsOptions::builder().stdout(true).stderr(true).build());

    while let Some(log_result) = logs_stream.next().await {
        match log_result {
            Ok(chunk) => print_chunk(chunk),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn print_chunk(chunk: TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => println!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdErr(bytes) => eprintln!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdIn(_) => unreachable!(),
    }
}
