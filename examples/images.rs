extern crate shiplift;

use shiplift::{Docker, ImageListOptions, ImageFilter};

fn main() {
    let docker = Docker::new();
    for i in docker.images().
        list(
            &ImageListOptions::builder().filter(
                vec![ImageFilter::Dangling]
            ).build()
        ).unwrap() {
        println!("image -> {:?}", i)
    }
}
