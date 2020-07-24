use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("You need to specify a network id");

    match docker.networks().get(&id).inspect().await {
        Ok(network_info) => println!("{:#?}", network_info),
        Err(e) => eprintln!("Error: {}", e),
    }
}
