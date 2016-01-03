//! Interfaces for building [docker](https://www.docker.com/) containers

extern crate rustc_serialize;
extern crate jed;
extern crate url;

use self::super::Docker;
use self::super::transport::Body;
use self::super::rep::{ContainerCreateInfo, Event};
use self::super::rep::Container as ContainerRep;
use std::collections::{BTreeMap, HashMap};
use std::io::Result;
use rustc_serialize::json::{self, Json, ToJson};

/// Interface for building container list request
pub struct ContainerListBuilder<'a> {
    docker: &'a mut Docker,
    params: HashMap<&'static str, String>,
}

impl<'a> ContainerListBuilder<'a> {
    pub fn new(docker: &'a mut Docker) -> ContainerListBuilder<'a> {
        ContainerListBuilder {
            docker: docker,
            params: HashMap::new(),
        }
    }

    pub fn all(mut self) -> ContainerListBuilder<'a> {
        self.params.insert("all", "true".to_owned());
        self
    }

    pub fn since(mut self, since: &str) -> ContainerListBuilder<'a> {
        self.params.insert("since", since.to_owned());
        self
    }

    pub fn before(mut self, before: &str) -> ContainerListBuilder<'a> {
        self.params.insert("before", before.to_owned());
        self
    }

    pub fn sized(mut self) -> ContainerListBuilder<'a> {
        self.params.insert("size", "true".to_owned());
        self
    }

    pub fn get(self) -> Result<Vec<ContainerRep>> {
        let mut params = Vec::new();
        for (k, v) in self.params {
            params.push(format!("{}={}", k, v))
        }
        let mut path = vec!["/containers/json".to_owned()];
        if !params.is_empty() {
            path.push(params.join("&"))
        }
        let raw = try!(self.docker.get(&path.join("?")));
        Ok(json::decode::<Vec<ContainerRep>>(&raw).unwrap())
    }
}

/// Interface for building a new docker container from an existing image
pub struct ContainerBuilder<'a, 'b> {
    docker: &'a mut Docker,
    image: &'b str,
    hostname: Option<String>,
    user: Option<String>,
    memory: Option<u64>,
}

impl<'a, 'b> ContainerBuilder<'a, 'b> {
    pub fn new(docker: &'a mut Docker, image: &'b str) -> ContainerBuilder<'a, 'b> {
        ContainerBuilder {
            docker: docker,
            image: image,
            hostname: None,
            user: None,
            memory: None,
        }
    }
    pub fn hostname(mut self, h: &str) -> ContainerBuilder<'a, 'b> {
        self.hostname = Some(h.to_owned());
        self
    }

    pub fn user(mut self, u: &str) -> ContainerBuilder<'a, 'b> {
        self.user = Some(u.to_owned());
        self
    }

    pub fn build(self) -> Result<ContainerCreateInfo> {
        let mut body = BTreeMap::new();
        body.insert("Image".to_owned(), self.image.to_json());
        let json_obj: Json = body.to_json();
        let data = json::encode(&json_obj).unwrap();
        let mut bytes = data.as_bytes();
        let raw = try!(self.docker.post("/containers/create",
                                        Some(Body::new(&mut Box::new(&mut bytes),
                                                       bytes.len() as u64))));
        Ok(json::decode::<ContainerCreateInfo>(&raw).unwrap())
    }
}

/// Interface for buiding an events request
pub struct Events<'a, 'b, 'c> {
    docker: &'a mut Docker,
    since: Option<&'b u64>,
    until: Option<&'c u64>,
}

impl<'a, 'b, 'c> Events<'a, 'b, 'c> {
    pub fn new(docker: &'a mut Docker) -> Events<'a, 'b, 'c> {
        Events {
            docker: docker,
            since: None,
            until: None,
        }
    }

    /// Filter events since a given timestamp
    pub fn since(mut self, ts: &'b u64) -> Events<'a, 'b, 'c> {
        self.since = Some(ts);
        self
    }

    /// Filter events until a given timestamp
    pub fn until(mut self, ts: &'c u64) -> Events<'a, 'b, 'c> {
        self.until = Some(ts);
        self
    }

    pub fn get(mut self) -> Result<Box<Iterator<Item = Event>>> {
        let mut params = Vec::new();
        if let Some(s) = self.since {
            params.push(format!("since={}", s));
        }
        if let Some(u) = self.until {
            params.push(format!("until={}", u));
        }
        let mut path = vec!["/events".to_owned()];
        if !params.is_empty() {
            path.push(params.join("&"))
        }
        let raw = try!(self.docker.stream_get(&path.join("?")[..]));
        let it = jed::Iter::new(raw).into_iter().map(|j| {
            let s = json::encode(&j).unwrap();
            json::decode::<Event>(&s).unwrap()
        });
        Ok(Box::new(it))
    }
}
