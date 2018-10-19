extern crate shiplift;
extern crate tokio;

use shiplift::{errors::Error, Docker};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::prelude::{Future, Stream};

fn main() {
    let docker = Docker::new();
    let id = env::args().nth(1).expect("You need to specify an image id");

    let mut export_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{}.tar", &id))
        .unwrap();
    let images = docker.images();
    let fut = images
        .get(&id)
        .export()
        .for_each(move |bytes| {
            export_file
                .write(&bytes[..])
                .map(|n| println!("copied {} bytes", n))
                .map_err(Error::IO)
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut)
}
