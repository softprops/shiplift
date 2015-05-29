extern crate shiplift;

use shiplift::Docker;

fn main() {
  let mut docker = Docker::new();
  println!("{:?}", docker.info().unwrap());
  //for e in docker.events().get().unwrap() {
  //  println!("{:?}", e);
  //}
}
