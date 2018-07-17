extern crate shiplift;

use shiplift::{Docker, ContainerArchiveOptions};
use std::env;

// args:
//      container
//      docker container directory path
//      local file/directory path
fn main() {
    let docker = Docker::new(None).unwrap();

    if env::args().len() < 3 {
        return;
    }

    let options = ContainerArchiveOptions::builder()
        .local_path(env::args().nth(3).unwrap())
        .path(env::args().nth(2).unwrap())
        .build();


    let id = env::args().nth(1).unwrap();
    docker.containers().get(&id).archive_put(&options);
}
