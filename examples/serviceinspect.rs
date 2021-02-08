use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example serviceinspect -- <service>");

    match docker.services().get(&id).inspect().await {
        Ok(service) => println!("{:#?}", service),
        Err(e) => eprintln!("Error: {}", e),
    }
}
