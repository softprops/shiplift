use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify an service name");

    if let Err(e) = docker.services().get(&id).delete().await {
        eprintln!("Error: {}", e)
    }
}
