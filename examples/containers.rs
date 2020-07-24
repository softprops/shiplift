use shiplift::Docker;

#[tokio::main]
async fn main() {
    env_logger::init();
    let docker = Docker::new();
    match docker.containers().list(&Default::default()).await {
        Ok(containers) => {
            for c in containers {
                println!("container -> {:#?}", c)
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
