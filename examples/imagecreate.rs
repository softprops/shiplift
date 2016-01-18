extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new();
    if let Some(img) = env::args().nth(1) {
        let image = docker.images()
            .create(&img[..])
            .unwrap();
        for output in image {
            println!("{:?}", output);
        }
    }
}
