extern crate shiplift;
extern crate env_logger;

use shiplift::Docker;

fn main() {
    let docker = Docker::new(None).unwrap();
    let docker = Docker::new();
    for c in docker.containers().list(&Default::default()).unwrap() {
        println!("container -> {:?}", c)
    }
}
