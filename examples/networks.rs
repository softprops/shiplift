extern crate env_logger;
extern crate shiplift;

use shiplift::Docker;

fn main() {
    env_logger::init();
    let docker = Docker::new();
    for c in docker.networks().list(&Default::default()).unwrap() {
        println!("network -> {:?}", c)
    }
}
