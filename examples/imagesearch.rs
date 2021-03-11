use shiplift::Docker;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
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
