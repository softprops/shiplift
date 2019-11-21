use shiplift::{ContainerConnectionOptions, Docker};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let networks = docker.networks();

    match (env::args().nth(1), env::args().nth(2)) {
        (Some(container_id), Some(network_id)) => {
            if let Err(e) = networks
                .get(&network_id)
                .connect(&ContainerConnectionOptions::builder(&container_id).build())
                .await
            {
                eprintln!("Error: {}", e)
            }
        }
        _ => eprintln!("please provide a container_id and network_id"),
    }
}
