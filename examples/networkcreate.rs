extern crate shiplift;

use shiplift::{Docker, NetworkCreateOptions};
use std::env;

fn main() {
    let docker = Docker::new(None).unwrap();
    let networks = docker.networks();
    if let Some(network_name) = env::args().nth(1) {
        let info = networks
            .create(
                &NetworkCreateOptions::builder(network_name.as_ref())
                    .driver("bridge")
                    .build(),
            )
            .unwrap();
        println!("{:?}", info);
    }
}
