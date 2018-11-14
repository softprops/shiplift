extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    println!("docker images in stock");
    let fut = docker
        .images()
        .list(&Default::default())
        .map(|images| {
            for i in images {
                println!("{:?}", i.repo_tags);
            }
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
