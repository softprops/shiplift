use futures::StreamExt;
use shiplift::{errors::Error, Docker};
use std::{env, fs::OpenOptions, io::Write};

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args().nth(1).expect("You need to specify an image id");

    let mut export_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{}.tar", &id))
        .unwrap();

    let images = docker.images();

    while let Some(export_result) = images.get(&id).export().next().await {
        match export_result {
            Ok(bytes) => export_file
                .write(&bytes)
                .map(|n| println!("copied {} bytes", n)),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
