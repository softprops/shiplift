extern crate env_logger;
extern crate shiplift;

use shiplift::Docker;

fn main() {
    env_logger::init();
    let docker = Docker::new();
    for c in docker.containers().list(&Default::default()).unwrap() {
        println!("container -> {:?}", c)
    }
}
