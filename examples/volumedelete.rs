use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();

    let volume_name = env::args()
        .nth(1)
        .expect("You need to specify an volume name");

    if let Err(e) = docker.volumes().get(&volume_name).delete().await {
        eprintln!("Error: {}", e)
    }
}
