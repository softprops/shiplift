extern crate shiplift;

use shiplift::Docker;

fn main() {
  let mut docker = Docker::new();
  println!("info {:?}", docker.info().unwrap());
}
