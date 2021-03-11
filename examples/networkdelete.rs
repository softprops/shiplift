use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a network id");

    if let Err(e) = docker.networks().get(&id).delete().await {
        eprintln!("Error: {}", e)
    }
}
