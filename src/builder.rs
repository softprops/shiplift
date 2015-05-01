//! Interfaces for building [docker](https://www.docker.com/) containers

extern crate rustc_serialize;

use self::super::Docker;
use self::super::transport::Body;
use self::super::rep::ContainerCreateInfo;
use std::collections::BTreeMap;
use std::io::Result;
use rustc_serialize::json::{self, Json, ToJson};

/// Interface for building a new docker container from an existing image
pub struct ContainerBuilder<'a, 'b> {
  docker: &'a mut Docker,
  image: &'b str,
  hostname: Option<String>,
  user: Option<String>,
  memory: Option<u64>
}

impl<'a, 'b> ContainerBuilder<'a, 'b> {
  pub fn new(docker: &'a mut Docker, image: &'b str) -> ContainerBuilder<'a,'b> {
    ContainerBuilder {
      docker: docker, image: image,
      hostname: None, user: None,
      memory: None
    }
  }
  pub fn hostname(mut self, h: &str) -> ContainerBuilder<'a,'b> {
    self.hostname = Some(h.to_string());
    self
  }

  pub fn user(mut self, u: &str) -> ContainerBuilder<'a, 'b> {
    self.user = Some(u.to_string());
    self
  }

  pub fn build(self) -> Result<ContainerCreateInfo> {
    let mut body = BTreeMap::new();
    body.insert("Image".to_string(), self.image.to_json());
    let json_obj: Json = body.to_json();
    let data = json::encode(&json_obj).unwrap();
    let mut bytes = data.as_bytes();
    let raw = try!(self.docker.post("/containers/create", Some(Body::new(&mut Box::new(&mut bytes), bytes.len() as u64))));
    Ok(json::decode::<ContainerCreateInfo>(&raw).unwrap())
  }
}
