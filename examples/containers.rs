extern crate shiplift;

use shiplift::{ContainerFilter, Docker};

fn main() {
    let docker = Docker::new();
    for c in docker.containers().
        list()
        .sized()
        .filter(vec![
            ContainerFilter::Label("foo".to_owned(), "bar".to_owned())
                ]).build().unwrap() {
        println!("container -> {:?}", c)
    }
}
