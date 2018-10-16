extern crate shiplift;
extern crate tokio;

use shiplift::{ContainerOptions, Docker};
use std::env;
use tokio::prelude::*;

fn main() {
    let docker = Docker::new();
    let image = env::args()
        .nth(1)
        .expect("You need to specify an image name");
    let fut = docker
        .containers()
        .create(&ContainerOptions::builder(image.as_ref()).build())
        .map(|info| println!("{:?}", info))
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
