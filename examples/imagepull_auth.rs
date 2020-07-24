// cargo run --example imagepull_auth busybox username password

use futures::StreamExt;
use shiplift::{Docker, PullOptions, RegistryAuth};
use std::env;

#[tokio::main]
async fn main() {
    env_logger::init();
    let docker = Docker::new();
    let img = env::args()
        .nth(1)
        .expect("You need to specify an image name");
    let username = env::args().nth(2).expect("You need to specify an username");
    let password = env::args().nth(3).expect("You need to specify a password");
    let auth = RegistryAuth::builder()
        .username(username)
        .password(password)
        .build();

    let mut stream = docker
        .images()
        .pull(&PullOptions::builder().image(img).auth(auth).build());

    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(output) => println!("{:?}", output),
            Err(e) => eprintln!("{}", e),
        }
    }
}
