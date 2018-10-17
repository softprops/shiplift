extern crate shiplift;
extern crate tokio;

use shiplift::{errors::Error, Docker, LogsOptions};
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
        .logs(&LogsOptions::builder().stdout(true).build())
        .for_each(|bytes| {
            let mut logs = &bytes[..];
            std::io::copy(&mut logs, &mut std::io::stdout())
                .map(|_| ())
                .map_err(Error::IO)
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
