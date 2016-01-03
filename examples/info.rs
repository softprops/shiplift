extern crate shiplift;

use shiplift::Docker;

fn main() {
  let docker = Docker::new();
  println!("info {:?}", docker.info().unwrap());
}
