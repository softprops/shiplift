extern crate shiplift;
extern crate jed;
extern crate rustc_serialize;

use rustc_serialize::json;
use shiplift::Docker;
use shiplift::rep::Stats;

fn main() {
  let mut docker = Docker::new();

  let data = docker.containers().get("160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36").stats().unwrap();
  for e in jed::Iter::new(data) {
    let s = json::encode(&e).unwrap();
    println!("\n -> {:?}", json::decode::<Stats>(&s).unwrap());
  };

  //let read = docker.images().create("redis:3.0.0").unwrap();
  //for e in jed::Iter::new(read) {
  //  println!("\n -> {:?}", e);
  //}  
}
