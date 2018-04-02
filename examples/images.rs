extern crate shiplift;

fn main() {
    let docker = shiplift::Docker::new().unwrap();
    let images = docker.images().list(&Default::default()).unwrap();
    println!("docker images in stock");
    for i in images {
        println!("{:?}", i.RepoTags);
    }
}
