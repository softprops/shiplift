use futures::StreamExt;
use shiplift::Docker;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
    println!("listening for events");

    while let Some(event_result) = docker.events(&Default::default()).next().await {
        match event_result {
            Ok(event) => println!("event -> {:?}", event),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
