extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use std::env;
use tokio::prelude::*;

fn main() {
    let docker = Docker::new();
    let containers = docker.containers();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example -- <container>");
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
