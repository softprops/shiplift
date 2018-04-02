extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new().unwrap();
    if let Some(id) = env::args().nth(1) {
        let status = docker.networks().get(&id).delete().unwrap();
        println!("{:?}", status);
    }
}
