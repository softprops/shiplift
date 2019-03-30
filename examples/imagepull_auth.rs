// cargo run --example imagepull_auth busybox username password

use shiplift::{Docker, PullOptions, RegistryAuth};
use std::env;
use tokio::prelude::{Future, Stream};

fn main() {
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
    let fut = docker
        .images()
        .pull(&PullOptions::builder().image(img).auth(auth).build())
        .for_each(|output| {
            println!("{:?}", output);
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
