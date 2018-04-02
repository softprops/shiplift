extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new().unwrap();
    println!("listening for events");
    for e in docker.events(&Default::default()).unwrap() {
        println!("event -> {:?}", e)
    }
}
