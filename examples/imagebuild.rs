use futures::StreamExt;
use shiplift::{BuildOptions, Docker};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let path = env::args().nth(1).expect("You need to specify a path");

    let options = BuildOptions::builder(path).tag("shiplift_test").build();

    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(output) => println!("{:?}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
