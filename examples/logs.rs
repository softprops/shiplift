extern crate shiplift;
extern crate tokio;

use shiplift::{tty::StreamType, Docker, LogsOptions};
use std::env;
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a container id");
    let fut = docker
        .containers()
        .get(&id)
        .logs(&LogsOptions::builder().stdout(true).stderr(true).build())
        .for_each(|line| {
            match line.stream_type {
                StreamType::StdOut => println!("INFO : {}", line.data),
                StreamType::StdErr => eprintln!("ERROR: {}", line.data),
            }
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
