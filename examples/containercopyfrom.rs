fn main() {}

/* use futures::TryStreamExt;
use shiplift::Docker;
use std::{env, path};

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example containercopyfrom -- <container> <path in container>");
    let path = env::args()
        .nth(2)
        .expect("Usage: cargo run --example containercopyfrom -- <container> <path in container>");

    match docker
        .containers()
        .get(&id)
        .copy_from(path::Path::new(&path))
        .try_concat()
        .await
    {
        Ok(tar) => {
            let mut archive = tar::Archive::new(tar);
            archive.unpack(env::current_dir().unwrap()).unwrap();
            Ok(())
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
 */
