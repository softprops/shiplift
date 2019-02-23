use shiplift::Docker;
use std::{env, path};
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let path = env::args()
        .nth(1)
        .expect("Usage: cargo run --example containercopyinto -- <local path> <container>");
    let id = env::args()
        .nth(2)
        .expect("Usage: cargo run --example containercopyinto -- <local path> <container>");


    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open(&path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Cannot read file on the localhost.");

    let fut = docker
        .containers()
        .get(&id)
        .copy_file_into(path::Path::new(&path), &bytes[..])
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
