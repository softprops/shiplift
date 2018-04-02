extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new().unwrap();
    if let Some(id) = env::args().nth(1) {
        let container = docker.containers().get(&id).inspect().unwrap();
        println!("{:?}", container);
    }
}
