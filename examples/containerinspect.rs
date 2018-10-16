extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use std::env;
use tokio::prelude::*;

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example containerinspect -- <container>");
    let fut = docker
        .containers()
        .get(&id)
        .inspect()
        .map(|container| println!("{:#?}", container))
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
