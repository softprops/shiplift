use shiplift::Docker;
use std::env;
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let containers = docker.containers();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example stats -- <container>");
    let fut = containers
        .get(&id)
        .stats()
        .for_each(|stat| {
            println!("{:?}", stat);
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
