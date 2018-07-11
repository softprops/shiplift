extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new(None).unwrap();
    if let Some(img) = env::args().nth(1) {
        let image = docker.images().get(&img[..]).delete().unwrap();
        for status in image {
            println!("{:?}", status);
        }
    }
}
