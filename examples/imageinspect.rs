use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example imageinspect -- <image>");

    match docker.images().get(&id).inspect().await {
        Ok(image) => println!("{:#?}", image),
        Err(e) => eprintln!("Error: {}", e),
    }
}
