extern crate shiplift;
extern crate env_logger;

use shiplift::Docker;

fn main() {
    env_logger::init().unwrap();
    let docker = Docker::new();
    for i in docker.images().
        list(&Default::default()).unwrap() {
        println!("image -> {:?}", i)
    }
}
