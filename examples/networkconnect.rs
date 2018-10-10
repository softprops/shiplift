extern crate shiplift;

use shiplift::{ContainerConnectionOptions, Docker};
use std::env;

fn main() {
    let docker = Docker::new();
    let networks = docker.networks();
    match (env::args().nth(1), env::args().nth(2)) {
        (Some(container_id), Some(network_id)) => println!(
            "{:?}",
            networks
                .get(&network_id)
                .connect(&ContainerConnectionOptions::new(&container_id))
        ),
        _ => eprintln!("please provide a container_id and network_id"),
    }
}
