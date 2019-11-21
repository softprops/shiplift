use shiplift::Docker;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    println!("remote docker images in stock");

    match docker.images().search("rust").await {
        Ok(results) => {
            for result in results {
                println!("{} - {}", result.name, result.description);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
