extern crate shiplift;

use shiplift::{ContainerOptions, Docker};
use std::env;

fn main() {
    let docker = Docker::new();
    let containers = docker.containers();
    if let Some(image) = env::args().nth(1) {
        let info = containers.create(
            &ContainerOptions::builder(image.as_ref()).build()
        ).unwrap();
        println!("{:?}", info);
    }
}
