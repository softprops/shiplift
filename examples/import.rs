use shiplift::Docker;
use std::{env, fs::File};
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let path = env::args()
        .nth(1)
        .expect("You need to specify an image path");
    let f = File::open(path).expect("Unable to open file");

    let reader = Box::from(f);

    let fut = docker
        .images()
        .import(reader)
        .for_each(|output| {
            println!("{:?}", output);
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
