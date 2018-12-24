use shiplift::Docker;
use std::env;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a network id");
    let fut = docker
        .networks()
        .get(&id)
        .delete()
        .map(|network| println!("{:?}", network))
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
