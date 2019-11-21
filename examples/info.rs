use shiplift::Docker;

#[tokio::main]
async fn main() {
    let docker = Docker::new();

    match docker.info().await {
        Ok(info) => println!("info {:?}", info),
        Err(e) => eprintln!("Error: {}", e),
    }
}
