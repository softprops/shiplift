use futures::StreamExt;
use std::{env, fs::OpenOptions, io::Write};

use shiplift::{errors::Error, Docker};

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args().nth(1).expect("You need to specify an image id");

    let mut export_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{}.tar", &id))
        .unwrap();

    while let Some(export_result) = docker.images().get(&id).export().next().await {
        match export_result.and_then(|bytes| export_file.write(&bytes).map_err(Error::from)) {
            Ok(n) => println!("copied {} bytes", n),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
