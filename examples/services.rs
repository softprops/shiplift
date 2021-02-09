use shiplift::{Docker, ServiceListOptions};

#[tokio::main]
async fn main() {
    env_logger::init();
    let docker = Docker::new();
    match docker
        .services()
        .list(&ServiceListOptions::builder().enable_status().build())
        .await
    {
        Ok(services) => {
            for s in services {
                println!("service -> {:#?}", s)
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
