extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use std::env;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify an container id");
    let fut = docker
        .containers()
        .get(&id)
        .delete()
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
