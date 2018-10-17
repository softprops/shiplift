extern crate shiplift;
extern crate tokio;

use shiplift::{ContainerConnectionOptions, Docker};
use std::env;
use tokio::prelude::*;

fn main() {
    let docker = Docker::new();
    let networks = docker.networks();
    match (env::args().nth(1), env::args().nth(2)) {
        (Some(container_id), Some(network_id)) => {
            let fut = networks
                .get(&network_id)
                .disconnect(&ContainerConnectionOptions::new(&container_id))
                .map(|v| println!("{:?}", v))
                .map_err(|e| eprintln!("Error: {}", e));
            tokio::run(fut);
        }
        _ => eprintln!("please provide a container_id and network_id"),
    }
}
