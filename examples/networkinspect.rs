extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new(None).unwrap();
    if let Some(id) = env::args().nth(1) {
        let network = docker.networks().get(&id).inspect().unwrap();
        println!("{:?}", network);
    }
}
