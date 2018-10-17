extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use std::env;
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example imageinspect -- <image>");
    let fut = docker
        .images()
        .get(&id)
        .inspect()
        .map(|image| println!("{:#?}", image))
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
