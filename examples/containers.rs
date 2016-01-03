extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new();
    for c in docker.containers().list().build().unwrap() {
        println!("container -> {:?}", c)
    }
}
