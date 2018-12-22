extern crate shiplift;
extern crate tokio;

use shiplift::builder::VolumeCreateOptions;
use shiplift::Docker;
use std::collections::HashMap;
use std::env;
use tokio::prelude::Future;

fn main() {
    let docker = Docker::new();
    let volumes = docker.volumes();

    let volume_name = env::args()
        .nth(1)
        .expect("You need to specify an volume name");

    let mut labels = HashMap::new();
    labels.insert("com.github.softprops", "shiplift");

    let fut = volumes
        .create(
            &VolumeCreateOptions::builder()
                .name(volume_name.as_ref())
                .labels(&labels)
                .build(),
        )
        .map(|info| println!("{:?}", info))
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
