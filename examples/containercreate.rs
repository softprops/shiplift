use shiplift::{ContainerOptions, Docker};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let image = env::args()
        .nth(1)
        .expect("You need to specify an image name");

    match docker
        .containers()
        .create(&ContainerOptions::builder(image.as_ref()).build())
        .await
    {
        Ok(info) => println!("{:?}", info),
        Err(e) => eprintln!("Error: {}", e),
    }
}
