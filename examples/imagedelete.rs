extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use std::env;
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let img = env::args()
        .nth(1)
        .expect("You need to specify an image name");
    let fut = docker
        .images()
        .get(&img[..])
        .delete()
        .map(|statuses| {
            for status in statuses {
                println!("{:?}", status);
            }
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
