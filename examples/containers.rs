extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new();
    for c in docker.containers().list().sized().build().unwrap() {
        println!("container -> {:?}", c)
    }
}
