extern crate shiplift;

use shiplift::Docker;

fn main() {
    let docker = Docker::new(None).unwrap();
    let images = docker.images().list(&Default::default()).unwrap();
    println!("docker images in stock");
    for i in images {
        println!("{:?}", i.RepoTags);
    }
}
