extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use std::env;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example top -- <container>");
    let fut = docker
        .containers()
        .get(&id)
        .top(Default::default())
        .map(|top| println!("{:#?}", top))
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
