extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new();
    for c in docker.containers().
        list(&Default::default()).unwrap() {
        println!("container -> {:?}", c)
    }
}
