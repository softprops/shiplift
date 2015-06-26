extern crate shiplift;

use shiplift::Docker;
use std::io::prelude::*;
use std::io::copy;
use std::fs::OpenOptions;

fn main() {
  let mut docker = Docker::new();
  println!("{:?}", docker.info().unwrap());

  let mut export = OpenOptions::new().write(true).create(true).open("export.tgz").unwrap();
  let mut images = docker.images();
  let mut exported = images.get("nginx").export().unwrap();
  println!("copying");
  copy(&mut exported, &mut export).unwrap();
  println!("copied");

  //let mut containers = docker.containers();
  //let stats = containers.get("f527f9be52b2").stats();
  //for s in stats.unwrap() {
  //  println!("{:?}", s);
  //}
}
