use futures::StreamExt;
use shiplift::{BuildOptions, Docker};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let path = env::args().nth(1).expect("You need to specify a path");

    match docker
        .images()
        .build(&BuildOptions::builder(path).tag("shiplift_test").build())
    {
        Ok(output) => {
            while let Some(chunk_result) = output.next().await {
                match chunk_result {
                    Ok(output) => println!("{:?}", output),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    }
}
