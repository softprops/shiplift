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
                println!(
                    "{} {:?}",
                    i.id,
                    i.repo_tags.unwrap_or_else(|| vec!["none".into()])
                );
            }
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
