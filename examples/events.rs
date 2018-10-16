extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use tokio::prelude::*;

fn main() {
    let docker = Docker::new();
    println!("listening for events");

    let fut = docker
        .events(&Default::default())
        .for_each(|e| {
            println!("event -> {:?}", e);
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
