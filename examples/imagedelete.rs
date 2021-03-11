use shiplift::Docker;
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new("tcp://127.0.0.1:80").unwrap();
    let img = env::args()
        .nth(1)
        .expect("You need to specify an image name");
    match docker.images().get(&img).delete().await {
        Ok(statuses) => {
            for status in statuses {
                println!("{:?}", status);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
