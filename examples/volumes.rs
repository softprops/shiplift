use shiplift::Docker;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let volumes = docker.volumes();

    let fut = volumes
        .list()
        .map(|volumes| {
            for v in volumes {
                println!("volume -> {:#?}", v)
            }
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
