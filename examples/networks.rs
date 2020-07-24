use shiplift::Docker;

#[tokio::main]
async fn main() {
    env_logger::init();

    let docker = Docker::new();

    match docker.networks().list(&Default::default()).await {
        Ok(networks) => {
            for network in networks {
                println!("network -> {:#?}", network)
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
