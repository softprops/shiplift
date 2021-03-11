use shiplift::Docker;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
    match docker.version().await {
        Ok(ver) => println!("version -> {:#?}", ver),
        Err(e) => eprintln!("Error: {}", e),
    }
}
