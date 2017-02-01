extern crate shiplift;

use shiplift::{Docker, ExecContainerOptions};
use std::env;

fn main() {
    let docker = Docker::new();
    let options = ExecContainerOptions::builder()
        .cmd(vec!["ls"])
        .env(vec!["VAR=value"])
        .build();
    if let Some(id) = env::args().nth(1) {
        match docker.containers()
            .get(&id)
            .exec(&options) {
            Ok(res) => println!("Success: {:?}", res),
            Err(err) => println!("An error occured: {:?}", err),
        }
    }
}
