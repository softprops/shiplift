//! Interfaces for building [docker](https://www.docker.com/) containers

extern crate rustc_serialize;
extern crate jed;
extern crate url;

use self::super::Docker;
use self::super::transport::Body;
use self::super::rep::{ ContainerCreateInfo, Event };
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

/// Interface for buiding an events request
pub struct Events<'a,'b,'c> {
  docker: &'a mut Docker,
  since: Option<&'b u64>,
  until: Option<&'c u64>
}

impl<'a,'b,'c> Events<'a,'b,'c> {
  pub fn new(docker: &'a mut Docker) -> Events<'a,'b,'c> {
    Events {
      docker: docker,
      since: None,
      until: None
    }
  }

  /// Filter events since a given timestamp
  pub fn since(mut self, ts: &'b u64) -> Events<'a,'b,'c> {
    self.since = Some(ts);
    self
  }

  /// Filter events until a given timestamp
  pub fn until(mut self, ts: &'c u64) -> Events<'a,'b,'c> {
    self.until = Some(ts);
    self
  }

  pub fn get(mut self) -> Result<Box<Iterator<Item=Event>>> {
    let mut params = Vec::new();
    if let Some(s) = self.since {
      params.push(format!("since={}", s));
    }
    if let Some(u) = self.until {
      params.push(format!("until={}", u));
    }
    let mut path = vec!("/events".to_string());
    if (!params.is_empty()) {
      path.push(params.connect("&"))
    }
    let raw = try!(self.docker.stream_get(&path.connect("?")[..]));
    let it = jed::Iter::new(raw).into_iter().map(|j| {
     let s = json::encode(&j).unwrap();
     json::decode::<Event>(&s).unwrap()
    });
    Ok(Box::new(it))
  }
}
