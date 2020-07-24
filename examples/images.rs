use shiplift::Docker;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    println!("docker images in stock");

    let result = docker.images().list(&Default::default()).await;

    match result {
        Ok(images) => {
            for i in images {
                println!(
                    "{} {} {:?}",
                    i.id,
                    i.created,
                    i.repo_tags.unwrap_or_else(|| vec!["none".into()])
                );
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
