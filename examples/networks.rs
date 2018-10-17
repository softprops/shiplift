extern crate env_logger;
extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use tokio::prelude::Future;

fn main() {
    env_logger::init();
    let docker = Docker::new();
    let fut = docker
        .networks()
        .list(&Default::default())
        .map(|networks| {
            for network in networks {
                println!("network -> {:#?}", network);
            }
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
