extern crate shiplift;

use shiplift::Docker;
use std::env;
use shiplift::builder::VolumeCreateOptions;

fn main() {
    let docker = Docker::new();
    let volumes = docker.volumes();
    if let Some(name) = env::args().nth(1) {
        let prior = volumes.get(&name).inspect();
        println!("PRIOR : {:?}", prior);

        let volume_create_options = VolumeCreateOptions::builder(&name)
            .build();

        let info = volumes
            .create(&volume_create_options)
            .unwrap();
        println!("{:?}", info);

        let post_create = volumes.get(&name).inspect();
        println!("POST CREATE : {:?}", post_create);

        let _ = volumes.get(&name).delete();

        let post_delete = volumes.get(&name).inspect();
        println!("POST DELETE : {:?}", post_delete);
    }
}
