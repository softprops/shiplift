extern crate shiplift;
extern crate env_logger;

use shiplift::Docker;

fn main() {
    env_logger::init().unwrap();
    let docker = Docker::new().unwrap();
    for c in docker.networks().list(&Default::default()).unwrap() {
        println!("network -> {:?}", c)
    }
}
