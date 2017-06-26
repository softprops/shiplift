extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new();
    if let Some(id) = env::args().nth(1) {
        let image = docker.images().get(&id).inspect().unwrap();
        println!("{:?}", image);
    }
}
