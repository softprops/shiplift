extern crate shiplift;

use shiplift::{ContainerConnectionOptions, Docker};
use std::env;

fn main() {
    let docker = Docker::new(None).unwrap();
    let networks = docker.networks();
    let container_id = env::args().nth(1).unwrap();
    let network_id = env::args().nth(2).unwrap();
    let info = networks
        .get(&network_id)
        .connect(&ContainerConnectionOptions::new(&container_id))
        .unwrap();
    println!("{:?}", info);
}
