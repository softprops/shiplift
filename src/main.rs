extern crate shiplift;
extern crate jed;
extern crate rustc_serialize;

use rustc_serialize::json;
use shiplift::Docker;
use shiplift::rep::Stats;

fn main() {
  let mut docker = Docker::new();

  //let data = docker.containers().get("160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36").stats().unwrap();
  //for e in jed::Iter::new(data) {
  //  let s = json::encode(&e).unwrap();
  //  println!("\n -> {:?}", json::decode::<Stats>(&s).unwrap());
  //};

  //println!("-> {}", docker.containers().get("160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36").inspect().unwrap());
  
  println!("delete {:?}", docker.containers().get("d3135eba971a7ba14ab5cbbee6952e341565e3b59a6557bf7f6a4e06b00854b4").stop().unwrap());

  //println!("start {:?}", docker.containers().get("4a3cd446f5fbc3e1f0f6ecc00508ddf9b34d294371335744d5d712836058f311").start().unwrap());

  //let read = docker.images().create("redis:3.0.0").unwrap();
  //for e in jed::Iter::new(read) {
  //  println!("\n -> {:?}", e);
  //}  
}
