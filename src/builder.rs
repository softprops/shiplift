//! Interfaces for building various structures

use self::super::Result;
use std::collections::{BTreeMap, HashMap};
use rustc_serialize::json::{self, Json, ToJson};
use url::form_urlencoded;

#[derive(Default)]
pub struct BuildOptions {
    pub path: String,
    params: HashMap<&'static str, String>,
}

impl BuildOptions {
    /// return a new instance of a builder for options
    pub fn builder<S>(path: S) -> BuildOptionsBuilder where S: Into<String> {
        BuildOptionsBuilder::new(path)
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

#[derive(Default)]
pub struct BuildOptionsBuilder {
    path: String,
    params: HashMap<&'static str, String>,
}

impl BuildOptionsBuilder {
    pub fn new<S>(path: S) -> BuildOptionsBuilder where S: Into<String>{
        BuildOptionsBuilder {
            path: path.into(),
            ..Default::default()
        }
    }

    pub fn dockerfile<P>(&mut self, path: P) -> &mut BuildOptionsBuilder where P: Into<String> {
        self.params.insert("dockerfile", path.into());
        self
    }

    pub fn tag<T>(&mut self, t: T) -> &mut BuildOptionsBuilder where T: Into<String> {
        self.params.insert("t", t.into());
        self
    }

    pub fn remote<R>(&mut self, r: R) -> &mut BuildOptionsBuilder where R: Into<String> {
        self.params.insert("remote", r.into());
        self
    }

    pub fn nocache<R>(&mut self, nc: bool) -> &mut BuildOptionsBuilder {
        self.params.insert("nocache", nc.to_string());
        self
    }

    pub fn rm(&mut self, r: bool) -> &mut BuildOptionsBuilder {
        self.params.insert("rm", r.to_string());
        self
    }

    pub fn forcerm(&mut self, fr: bool) -> &mut BuildOptionsBuilder {
        self.params.insert("forcerm", fr.to_string());
        self
    }

    // todo: memory
    // todo: memswap
    // todo: cpushares
    // todo: cpusetcpus
    // todo: cpuperiod
    // todo: cpuquota
    // todo: buildargs

    pub fn build(&self) -> BuildOptions {
        BuildOptions { path: self.path.clone(), params: self.params.clone() }
    }
}

/// Options for filtering container list results
#[derive(Default)]
pub struct ContainerListOptions {
    params: HashMap<&'static str, String>,
}

impl ContainerListOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> ContainerListOptionsBuilder {
        ContainerListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

/// Filter options for container listings
pub enum ContainerFilter {
    ExitCode(u64),
    Status(String),
    LabelName(String),
    Label(String, String),
}

/// Builder interface for `ContainerListOptions`
#[derive(Default)]
pub struct ContainerListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl ContainerListOptionsBuilder {
    pub fn new() -> ContainerListOptionsBuilder {
        ContainerListOptionsBuilder { ..Default::default() }
    }

    pub fn filter(&mut self, filters: Vec<ContainerFilter>) -> &mut ContainerListOptionsBuilder {
        let mut param = HashMap::new();
        for f in filters {
            match f {
                ContainerFilter::ExitCode(c) => param.insert("exit", vec![c.to_string()]),
                ContainerFilter::Status(s) => param.insert("status", vec![s]),
                ContainerFilter::LabelName(n) => param.insert("label", vec![n]),
                ContainerFilter::Label(n, v) => param.insert("label", vec![format!("{}={}", n, v)]),
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

    pub fn build(&self) -> ContainerListOptions {
        ContainerListOptions { params: self.params.clone() }
    }
}

/// Interface for building a new docker container from an existing image
pub struct ContainerOptions {
    params: HashMap<&'static str, String>
}

impl ContainerOptions {
    /// return a new instance of a builder for options
    pub fn builder(name: &str) -> ContainerOptionsBuilder {
        ContainerOptionsBuilder::new(name)
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        let mut body = BTreeMap::new();
        if let Some(image) = self.params.get("Image") {
            body.insert("Image".to_owned(), image.to_json());
        }
        let json_obj: Json = body.to_json();
        Ok(try!(json::encode(&json_obj)))
    }
}

#[derive(Default)]
pub struct ContainerOptionsBuilder {
    params: HashMap<&'static str, String>
}

impl ContainerOptionsBuilder {
    pub fn new(name: &str) -> ContainerOptionsBuilder {
        let mut params = HashMap::new();
        params.insert("Image", name.to_owned());
        ContainerOptionsBuilder {
            params: params
        }
    }

    pub fn build(&self) -> ContainerOptions {
        ContainerOptions {
            params: self.params.clone()
        }
    }
}


/// Options for filtering streams of Docker events
#[derive(Default)]
pub struct EventsOptions {
    params: HashMap<&'static str, String>,
}

impl EventsOptions {
    pub fn builder() -> EventsOptionsBuilder {
        EventsOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

/// Builder interface for `EventOptions`
#[derive(Default)]
pub struct EventsOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl EventsOptionsBuilder {
    pub fn new() -> EventsOptionsBuilder {
        EventsOptionsBuilder { ..Default::default() }
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
        EventsOptions { params: self.params.clone() }
    }
}


/// Options for controlling log request results
#[derive(Default)]
pub struct LogsOptions {
    params: HashMap<&'static str, String>,
}

impl LogsOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> LogsOptionsBuilder {
        LogsOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

/// Builder interface for `LogsOptions`
#[derive(Default)]
pub struct LogsOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl LogsOptionsBuilder {
    pub fn new() -> LogsOptionsBuilder {
        LogsOptionsBuilder { ..Default::default() }
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
        LogsOptions { params: self.params.clone() }
    }
}


/// Filter options for image listings
pub enum ImageFilter {
    Dangling,
    LabelName(String),
    Label(String, String),
}

/// Options for filtering image list results
#[derive(Default)]
pub struct ImageListOptions {
    params: HashMap<&'static str, String>,
}

impl ImageListOptions {
    pub fn builder() -> ImageListOptionsBuilder {
        ImageListOptionsBuilder::new()
    }
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(form_urlencoded::serialize(&self.params))
        }
    }
}

/// Builder interface for `ImageListOptions`
#[derive(Default)]
pub struct ImageListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl ImageListOptionsBuilder {
    pub fn new() -> ImageListOptionsBuilder {
        ImageListOptionsBuilder { ..Default::default() }
    }

    pub fn digests(&mut self, d: bool) -> &mut ImageListOptionsBuilder {
        self.params.insert("digests", d.to_string());
        self
    }

    pub fn all(&mut self, a: bool) -> &mut ImageListOptionsBuilder {
        self.params.insert("all", a.to_string());
        self
    }

    pub fn filter_name(&mut self, name: &str) -> &mut ImageListOptionsBuilder {
        self.params.insert("filter", name.to_owned());
        self
    }

    pub fn filter(&mut self, filters: Vec<ImageFilter>) -> &mut ImageListOptionsBuilder {
        let mut param = HashMap::new();
        for f in filters {
            match f {
                ImageFilter::Dangling => param.insert("dangling", vec![true.to_string()]),
                ImageFilter::LabelName(n) => param.insert("label", vec![n]),
                ImageFilter::Label(n, v) => param.insert("label", vec![format!("{}={}", n, v)]),
            };

        }
        // structure is a a json encoded object mapping string keys to a list
        // of string values
        self.params.insert("filters", json::encode(&param).unwrap());
        self
    }

    pub fn build(&self) -> ImageListOptions {
        ImageListOptions { params: self.params.clone() }
    }
}
