use shiplift::Docker;
use tokio::prelude::Future;

fn main() {
    env_logger::init();
    let docker = Docker::new();
    let fut = docker
        .version()
        .map(|ver| println!("version -> {:#?}", ver))
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
