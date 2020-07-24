use shiplift::Docker;
use std::{env, fs::File, io::Read};

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let path = env::args()
        .nth(1)
        .expect("Usage: cargo run --example containercopyinto -- <local path> <container>");
    let id = env::args()
        .nth(2)
        .expect("Usage: cargo run --example containercopyinto -- <local path> <container>");

    let mut file = File::open(&path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .expect("Cannot read file on the localhost.");

    if let Err(e) = docker
        .containers()
        .get(&id)
        .copy_file_into(path, &bytes)
        .await
    {
        eprintln!("Error: {}", e)
    }
}
