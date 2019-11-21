use shiplift::{Docker, NetworkCreateOptions};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let network_name = env::args()
        .nth(1)
        .expect("You need to specify a network name");
    match docker
        .networks()
        .create(
            &NetworkCreateOptions::builder(network_name.as_ref())
                .driver("bridge")
                .build(),
        )
        .await
    {
        Ok(info) => println!("{:?}", info),
        Err(e) => eprintln!("Error: {}", e),
    }
}
