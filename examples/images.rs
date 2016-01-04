extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new();
    for i in docker.images().
        list(&Default::default()).unwrap() {
        println!("image -> {:?}", i)
    }
}
