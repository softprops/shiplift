use shiplift::{Docker, NetworkCreateOptions};
use std::env;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let network_name = env::args()
        .nth(1)
        .expect("You need to specify a network name");
    let fut = docker
        .networks()
        .create(
            &NetworkCreateOptions::builder(network_name.as_ref())
                .driver("bridge")
                .build(),
        )
        .map(|info| println!("{:?}", info))
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
