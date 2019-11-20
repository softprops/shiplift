//  cargo run --example stats -- <container>
use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let containers = docker.containers();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example stats -- <container>");

    while let Some(result) = containers.get(&id).stats().next() {
        match result {
            Ok(stat) => println!("{:?}", stat),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
