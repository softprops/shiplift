//! Source of code without generic types:
//! https://github.com/faradayio/boondock/blob/master/src/stats.rs

use std::iter;
use std::io::{BufRead, BufReader};
use hyper::client::response::Response;

use serde_json;
use errors::*;
use std::marker::PhantomData;
use serde::de::DeserializeOwned;

pub struct Bufreader<T: DeserializeOwned> {
    buf: BufReader<Response>,
    _phantom: PhantomData<T>
}

impl <T: DeserializeOwned> Bufreader<T> {
    pub fn new(r: Response) -> Bufreader<T> {
        Bufreader {
            buf: BufReader::new(r),
            _phantom: PhantomData
        }
    }
}

impl <T: DeserializeOwned> iter::Iterator for Bufreader<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        let mut line = String::new();

        if let Err(err) = self.buf.read_line(&mut line) {
            return Some(Err(err.into()));
        }

        println!("////////{}", line);
        if line.len() == 1 {
            return None
        }

        Some(serde_json::from_str::<T>(&line).map_err(Error::from))
    }
}