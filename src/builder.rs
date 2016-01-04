//! Interfaces for building [docker](https://www.docker.com/) containers

extern crate rustc_serialize;
extern crate jed;
extern crate url;

use self::super::{Docker, Result};
use self::super::transport::Body;
use self::super::rep::ContainerCreateInfo;
use std::collections::{BTreeMap, HashMap};
use rustc_serialize::json::{self, Json, ToJson};
use url::form_urlencoded;

#[derive(Default)]
pub struct ContainerListOptions {
    params: HashMap<&'static str, String>
}

impl ContainerListOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> ContainerListOptionsBuilder {
        ContainerListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() { None }
        else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

/// Filter options for container listings
pub enum ContainerFilter {
    ExitCode(u64),
    Status(String),
    LabelName(String),
    Label(String, String)
}

/// Interface for building container list request
#[derive(Default)]
pub struct ContainerListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl ContainerListOptionsBuilder {
    pub fn new() -> ContainerListOptionsBuilder {
        ContainerListOptionsBuilder {
            ..Default::default()
        }
    }

    pub fn filter(&mut self, filters: Vec<ContainerFilter>) -> &mut ContainerListOptionsBuilder {
        let mut param = HashMap::new();
        for f in filters {
            match f {
                ContainerFilter::ExitCode(c) => param.insert("exit", vec![c.to_string()]),
                ContainerFilter::Status(s) => param.insert("status", vec![s]),
                ContainerFilter::LabelName(n) => param.insert("label", vec![n]),
                ContainerFilter::Label(n,v) => param.insert("label", vec![format!("{}={}", n, v)])
            };

        }
        // structure is a a json encoded object mapping string keys to a list
        // of string values
        self.params.insert("filters", json::encode(&param).unwrap());
        self
    }

    pub fn all(&mut self) -> &mut ContainerListOptionsBuilder {
        self.params.insert("all", "true".to_owned());
        self
    }

    pub fn since(&mut self, since: &str) -> &mut ContainerListOptionsBuilder {
        self.params.insert("since", since.to_owned());
        self
    }

    pub fn before(&mut self, before: &str) -> &mut ContainerListOptionsBuilder {
        self.params.insert("before", before.to_owned());
        self
    }

    pub fn sized(&mut self) -> &mut ContainerListOptionsBuilder {
        self.params.insert("size", "true".to_owned());
        self
    }

    pub fn build(&self) -> ContainerListOptions {//Result<Vec<ContainerRep>> {
        ContainerListOptions {
            params: self.params.clone()
        }
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

#[derive(Default)]
pub struct EventsOptions {
    params: HashMap<&'static str, String>
}

impl EventsOptions {
    pub fn builder() -> EventsOptionsBuilder {
        EventsOptionsBuilder::new()
    }

    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() { None }
        else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

/// Interface for buiding an events request
#[derive(Default)]
pub struct EventsOptionsBuilder {
    params: HashMap<&'static str, String>
}

impl EventsOptionsBuilder {
    pub fn new() -> EventsOptionsBuilder {
        EventsOptionsBuilder {
            ..Default::default()
        }
    }

    /// Filter events since a given timestamp
    pub fn since(&mut self, ts: &u64) -> &mut EventsOptionsBuilder {
        self.params.insert("since", ts.to_string());
        self
    }

    /// Filter events until a given timestamp
    pub fn until(&mut self, ts: &u64) -> &mut EventsOptionsBuilder {
        self.params.insert("until", ts.to_string());
        self
    }

    pub fn build(&self) -> EventsOptions {
        EventsOptions {
            params: self.params.clone()
        }
    }
}


#[derive(Default)]
pub struct LogsOptions {
    params: HashMap<&'static str, String>
}

impl LogsOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> LogsOptionsBuilder {
        LogsOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() { None }
        else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

#[derive(Default)]
pub struct LogsOptionsBuilder {
    params: HashMap<&'static str, String>
}

impl LogsOptionsBuilder {
    pub fn new() -> LogsOptionsBuilder {
        LogsOptionsBuilder {
            ..Default::default()
        }
    }

    pub fn follow(&mut self, f: bool) -> &mut LogsOptionsBuilder {
        self.params.insert("follow", f.to_string());
        self
    }

    pub fn stdout(&mut self, s: bool) -> &mut LogsOptionsBuilder {
        self.params.insert("stdout", s.to_string());
        self
    }

    pub fn stderr(&mut self, s: bool) -> &mut LogsOptionsBuilder {
        self.params.insert("stderr", s.to_string());
        self
    }

    pub fn timestamps(&mut self, t: bool) -> &mut LogsOptionsBuilder {
        self.params.insert("timestamps", t.to_string());
        self
    }

    /// how_many can either by "all" or a to_string() of the number
    pub fn tail(&mut self, how_many: &str) -> &mut LogsOptionsBuilder {
        self.params.insert("tail", how_many.to_owned());
        self
    }

    pub fn build(&self) -> LogsOptions {
        LogsOptions {
            params: self.params.clone()
        }
    }
}
