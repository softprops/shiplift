use futures::TryStreamExt;
use shiplift::Docker;
use std::{env, path};
use tar::Archive;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example containercopyfrom -- <container> <path in container>");
    let path = env::args()
        .nth(2)
        .expect("Usage: cargo run --example containercopyfrom -- <container> <path in container>");

    let bytes = docker
        .containers()
        .get(&id)
        .copy_from(path::Path::new(&path))
        .try_concat()
        .await?;

    let mut archive = Archive::new(&bytes[..]);
    archive.unpack(env::current_dir()?)?;

    Ok(())
}
