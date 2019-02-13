use shiplift::Docker;
use std::{env, path};
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example containercopyfrom -- <container> <path in container>");
    let path = env::args()
        .nth(2)
        .expect("Usage: cargo run --example containercopyfrom -- <container> <path in container>");
    let fut = docker
        .containers()
        .get(&id)
        .copy_from(path::Path::new(&path))
        .collect()
        .and_then(|stream| {
            let tar = stream.concat();
            let mut archive = tar::Archive::new(tar.as_slice());
            archive.unpack(env::current_dir()?)?;
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));
    tokio::run(fut);
}
