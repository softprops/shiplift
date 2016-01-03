extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new();
    println!("listening for events");
    for e in docker.events().build().unwrap() {
        println!("event -> {:?}", e)
    }
}
