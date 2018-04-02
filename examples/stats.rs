extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new().unwrap();
    let containers = docker.containers();
    if let Some(id) = env::args().nth(1) {
        let stats = containers.get(&id).stats();
        for s in stats.unwrap() {
            println!("{:?}", s);
        }
    }
}
