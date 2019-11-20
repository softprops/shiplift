use futures::TryStreamExt;
use shiplift::{tty::TtyChunk, Docker, LogsOptions};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a container id");

    let result = docker
        .containers()
        .get(&id)
        .logs(&LogsOptions::builder().stdout(true).stderr(true).build())
        .try_for_each(|chunk| match chunk {
            TtyChunk::StdOut(bytes) => println!("Stdout: {}", bytes.as_string_lossy()),
            TtyChunk::StdErr(bytes) => println!("Stderr: {}", bytes.as_string_lossy()),
            TtyChunk::Stdin(bytes) => unreachable!(),
        })
        .await;

    if let Err(e) = result {
        eprintln!("Error: {}", e)
    }
}
