extern crate shiplift;
extern crate tokio;

use shiplift::{BuildOptions, Docker};
use std::env;
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let path = env::args().nth(1).expect("You need to specify a path");

    let fut = docker
        .images()
        .build(&BuildOptions::builder(path).tag("shiplift_test").build())
        .for_each(|output| {
            println!("{:?}", output);
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
