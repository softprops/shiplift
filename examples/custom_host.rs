use shiplift::Docker;

#[tokio::main]
async fn main() {
    let docker = Docker::host("http://yourhost".parse().unwrap());
    match docker.ping().await {
        Ok(pong) => println!("Ping: {}", pong),
        Err(e) => eprintln!("Error: {}", e),
    }
}
