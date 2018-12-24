use shiplift::Docker;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    println!("remote docker images in stock");
    let fut = docker
        .images()
        .search("rust")
        .map(|results| {
            for result in results {
                println!("{} - {}", result.name, result.description);
            }
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
