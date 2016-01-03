//! Interfaces for building [docker](https://www.docker.com/) containers

extern crate rustc_serialize;
extern crate jed;
extern crate url;

use self::super::{Docker, Result};
use self::super::transport::Body;
use self::super::rep::{ContainerCreateInfo, Event};
use self::super::rep::Container as ContainerRep;
use std::collections::{BTreeMap, HashMap};
use rustc_serialize::json::{self, Json, ToJson};
use url::form_urlencoded;

/// Interface for building container list request
pub struct ContainerListBuilder<'a> {
    docker: &'a Docker,
    params: HashMap<&'static str, String>,
}

impl<'a> ContainerListBuilder<'a> {
    pub fn new(docker: &'a Docker) -> ContainerListBuilder<'a> {
        ContainerListBuilder {
            docker: docker,
            params: HashMap::new(),
        }
    }

    pub fn all(&mut self) -> &mut ContainerListBuilder<'a> {
        self.params.insert("all", "true".to_owned());
        self
    }

    pub fn since(&mut self, since: &str) -> &mut ContainerListBuilder<'a> {
        self.params.insert("since", since.to_owned());
        self
    }

    pub fn before(&mut self, before: &str) -> &mut ContainerListBuilder<'a> {
        self.params.insert("before", before.to_owned());
        self
    }

    pub fn sized(&mut self) -> &mut ContainerListBuilder<'a> {
        self.params.insert("size", "true".to_owned());
        self
    }

    pub fn build(self) -> Result<Vec<ContainerRep>> {
        let mut path = vec!["/containers/json".to_owned()];
        if !self.params.is_empty() {
            let encoded = form_urlencoded::serialize(self.params);
            path.push(encoded)
        }
        let raw = try!(self.docker.get(&path.join("?")));
        Ok(try!(json::decode::<Vec<ContainerRep>>(&raw)))
    }
}

/// Interface for building a new docker container from an existing image
pub struct ContainerBuilder<'a, 'b> {
    docker: &'a Docker,
    image: &'b str,
    hostname: Option<String>,
    user: Option<String>,
    memory: Option<u64>,
}

impl<'a, 'b> ContainerBuilder<'a, 'b> {
    pub fn new(docker: &'a Docker, image: &'b str) -> ContainerBuilder<'a, 'b> {
        ContainerBuilder {
            docker: docker,
            image: image,
            hostname: None,
            user: None,
            memory: None,
        }
    }

    pub fn hostname(&mut self, h: &str) -> &mut ContainerBuilder<'a, 'b> {
        self.hostname = Some(h.to_owned());
        self
    }

    pub fn user(&mut self, u: &str) -> &mut ContainerBuilder<'a, 'b> {
        self.user = Some(u.to_owned());
        self
    }

    pub fn build(&self) -> Result<ContainerCreateInfo> {
        let mut body = BTreeMap::new();
        body.insert("Image".to_owned(), self.image.to_json());
        let json_obj: Json = body.to_json();
        let data = try!(json::encode(&json_obj));
        let mut bytes = data.as_bytes();
        let raw = try!(self.docker.post("/containers/create",
                                        Some(Body::new(&mut Box::new(&mut bytes),
                                                       bytes.len() as u64))));
        Ok(try!(json::decode::<ContainerCreateInfo>(&raw)))
    }
}

/// Interface for buiding an events request
pub struct EventsBuilder<'a, 'b, 'c> {
    docker: &'a Docker,
    since: Option<&'b u64>,
    until: Option<&'c u64>,
}

impl<'a, 'b, 'c> EventsBuilder<'a, 'b, 'c> {
    pub fn new(docker: &'a Docker) -> EventsBuilder<'a, 'b, 'c> {
        EventsBuilder {
            docker: docker,
            since: None,
            until: None,
        }
    }

    /// Filter events since a given timestamp
    pub fn since(&mut self, ts: &'b u64) -> &mut EventsBuilder<'a, 'b, 'c> {
        self.since = Some(ts);
        self
    }

    /// Filter events until a given timestamp
    pub fn until(&mut self, ts: &'c u64) -> &mut EventsBuilder<'a, 'b, 'c> {
        self.until = Some(ts);
        self
    }

    /// Returns an interator over streamed docker events
    pub fn build(&self) -> Result<Box<Iterator<Item = Event>>> {
        let mut params = Vec::new();
        if let Some(s) = self.since {
            params.push(("since", s.to_string()));
        }
        if let Some(u) = self.until {
            params.push(("until", u.to_string()));
        }
        let mut path = vec!["/events".to_owned()];
        if !params.is_empty() {
            let encoded = form_urlencoded::serialize(params);
            path.push(encoded)
        }
        let raw = try!(self.docker.stream_get(&path.join("?")[..]));
        let it = jed::Iter::new(raw).into_iter().map(|j| {
            // fixme: better error handling
            let s = json::encode(&j).unwrap();
            json::decode::<Event>(&s).unwrap()
        });
        Ok(Box::new(it))
    }
}
