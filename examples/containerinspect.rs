use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example containerinspect -- <container>");

    match docker.containers().get(&id).inspect().await {
        Ok(container) => println!("{:#?}", container),
        Err(e) => eprintln!("Error: {}", e),
    }
}
