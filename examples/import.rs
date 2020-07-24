use futures::StreamExt;
use shiplift::Docker;
use std::{env, fs::File};

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let path = env::args()
        .nth(1)
        .expect("You need to specify an image path");
    let f = File::open(path).expect("Unable to open file");

    let reader = Box::from(f);

    let mut stream = docker.images().import(reader);

    while let Some(import_result) = stream.next().await {
        match import_result {
            Ok(output) => println!("{:?}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
