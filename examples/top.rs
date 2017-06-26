extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new();
    if let Some(id) = env::args().nth(1) {
        let top = docker
            .containers()
            .get(&id)
            .top(Default::default())
            .unwrap();
        println!("{:?}", top);
    }
}
