extern crate shiplift;

use shiplift::{BuildOptions, Docker};
use std::env;

fn main() {
    let docker = Docker::new().unwrap();
    if let Some(path) = env::args().nth(1) {
        let image = docker
            .images()
            .build(&BuildOptions::builder(path).tag("shiplift_test").build())
            .unwrap();
        for output in image {
            println!("{:?}", output);
        }
    }
}
