extern crate shiplift;

use shiplift::{Docker, ExecContainerOptions};
use std::env;

fn main() {
    let docker = Docker::new();
    let options = ExecContainerOptions::builder().cmd(vec!["ls"]).build();
    if let Some(id) = env::args().nth(1) {
        let container = docker.containers()
            .get(&id)
            .exec(&options)
            .unwrap();
        println!("{:?}", container);
    }
}
