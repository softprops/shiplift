extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use std::env;
use tokio::prelude::*;

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a network id");
    let fut = docker
        .networks()
        .get(&id)
        .inspect()
        .map(|network| println!("{:#?}", network))
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
