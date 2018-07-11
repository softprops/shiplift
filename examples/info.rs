extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new(None).unwrap();
    println!("info {:?}", docker.info().unwrap());
}
