extern crate shiplift;

use shiplift::{BuildOptions, Docker};
use std::env;

fn main() {
    let docker = Docker::new();
    if let Some(path) = env::args().nth(1) {
        let image = docker.images()
            .build(&BuildOptions::builder(path).build())
            .unwrap();
        println!("{:?}", image);
    }
}
