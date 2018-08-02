extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new();
    if let Some(id) = env::args().nth(1) {
        let volume = docker.volumes().get(&id).inspect().unwrap();
        println!("{:?}", volume);
    }
}
