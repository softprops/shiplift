use futures::StreamExt;
use shiplift::{tty::TtyChunk, Docker};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a container id");

    let tty_multiplexer = docker.containers().get(&id).attach().await?;

    let (mut reader, _writer) = tty_multiplexer.split();

    while let Some(tty_result) = reader.next().await {
        match tty_result {
            Ok(chunk) => print_chunk(chunk),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

fn print_chunk(chunk: TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => println!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdErr(bytes) => eprintln!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdIn(_) => unreachable!(),
    }
}
