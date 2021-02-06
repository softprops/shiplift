use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example top -- <container>");

    match docker.containers().get(&id).top(None).await {
        Ok(top) => println!("{:#?}", top),
        Err(e) => eprintln!("Error: {}", e),
    }
}
