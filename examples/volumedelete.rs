use shiplift::Docker;
use std::env;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let volumes = docker.volumes();

    let volume_name = env::args()
        .nth(1)
        .expect("You need to specify an volume name");

    let fut = volumes
        .get(&volume_name)
        .delete()
        .map(|info| println!("{:?}", info))
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
