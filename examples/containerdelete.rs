use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify an container id");

    if let Err(e) = docker.containers().get(&id).delete().await {
        eprintln!("Error: {}", e)
    }
}
