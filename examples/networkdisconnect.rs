use shiplift::{ContainerConnectionOptions, Docker};
use std::env;

async fn network_disconnect(
    container_id: &str,
    network_id: &str,
) {
    let docker = Docker::new();
    if let Err(e) = docker
        .networks()
        .get(network_id)
        .disconnect(&ContainerConnectionOptions::builder(container_id).build())
        .await
    {
        eprintln!("Error: {}", e)
    }
}

#[tokio::main]
async fn main() {
    match (env::args().nth(1), env::args().nth(2)) {
        (Some(container_id), Some(network_id)) => {
            network_disconnect(&container_id, &network_id).await;
        }
        _ => eprintln!("please provide a container_id and network_id"),
    }
}
