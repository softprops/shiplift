extern crate shiplift;

use shiplift::Docker;
use std::io::prelude::*;
use std::io::copy;
use std::fs::OpenOptions;

fn main() {
  let docker = Docker::new();
  let export = OpenOptions::new().write(true).create(true).open("export.tgz").unwrap();
  let images = docker.images();
  let mut exported = images.get("nginx").export().unwrap();
  println!("copying");
  copy(&mut exported, &mut export).unwrap();
  println!("copied");
}
