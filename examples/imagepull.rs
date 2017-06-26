extern crate shiplift;

use shiplift::{Docker, PullOptions};
use std::env;

fn main() {
    let docker = Docker::new();
    if let Some(img) = env::args().nth(1) {
        let image = docker
            .images()
            .pull(&PullOptions::builder().image(img).build())
            .unwrap();
        for output in image {
            println!("{:?}", output);
        }
    }
}
