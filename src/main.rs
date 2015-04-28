extern crate shiplift;
extern crate jed;

use shiplift::Docker;

fn main() {
  let mut docker = Docker::new();
  let info = docker.info().unwrap();
  println!("info -> {:?}", info);
  let read = docker.images().create("redis:3.0.0").unwrap();
  for e in jed::Iter::new(read) {
    println!("\n -> {:?}", e);
  }  
}
