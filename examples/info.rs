extern crate shiplift;
extern crate tokio;

use shiplift::Docker;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    tokio::run(
        docker
            .info()
            .map(|info| println!("info {:?}", info))
            .map_err(|e| eprintln!("Error: {}", e)),
    );
}
