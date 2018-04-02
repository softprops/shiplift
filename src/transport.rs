//! Transports for communicating with the docker daemon

extern crate hyper;

use self::hyper::buffer::BufReader;
use self::hyper::header::ContentType;
use self::hyper::status::StatusCode;
use errors::{Result, ErrorKind};

use hyper::Client;
use hyper::client::Body;
use hyper::client::response::Response;
use hyper::header;
use hyper::method::Method;
use hyper::mime;
use hyperlocal::DomainUrl;
use rustc_serialize::json;
use std::fmt;
use std::io::Read;

pub fn tar() -> ContentType {
    ContentType(mime::Mime(
        mime::TopLevel::Application,
        mime::SubLevel::Ext(String::from("tar")),
        vec![],
    ))
}

/// Transports are types which define the means of communication
/// with the docker daemon
pub enum Transport {
    /// A network tcp interface
    Tcp { client: Client, host: String },
    /// A Unix domain socket
    Unix { client: Client, path: String },
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Transport::Tcp { ref host, .. } => write!(f, "Tcp({})", host),
            Transport::Unix { ref path, .. } => write!(f, "Unix({})", path),
        }
    }
}

impl Transport {
    pub fn request<'a, B>(&'a self, method: Method, endpoint: &str, body: Option<(B, ContentType)>)
        -> Result<String>
        where
            B: Into<Body<'a>>
    {
        let mut res  = self.stream(method, endpoint, body)?;
        let mut body = String::new();
        let _        = res.read_to_string(&mut body)?;

        debug!("{} raw response: {}", endpoint, body);
        Ok(body)
    }

    pub fn stream<'c, B>(&'c self, method: Method, endpoint: &str, body: Option<(B, ContentType)>)
        -> Result<Box<Read>>
        where
            B: Into<Body<'c>>
    {
        let headers = {
            let mut headers = header::Headers::new();
            headers.set(header::Host {
                hostname: "".to_owned(),
                port: None,
            });
            headers
        };

        let req = match *self {
            Transport::Tcp { ref client, ref host, } => {
                client.request(method, &format!("{}{}", host, endpoint)[..])
            },
            Transport::Unix { ref client, ref path, } => {
                client.request(method, DomainUrl::new(&path, endpoint))
            },
        }.headers(headers);

        let embodied = match body {
            Some((b, c)) => req.header(c).body(b),
            _            => req,
        };

        let mut res = embodied.send()?;

        match res.status {
            StatusCode::Ok                 |
            StatusCode::Created            |
            StatusCode::SwitchingProtocols => Ok(Box::new(res)),
            StatusCode::NoContent          => Ok(Box::new(BufReader::new("".as_bytes()))),
            // todo: constantize these
            StatusCode::BadRequest          |
            StatusCode::NotFound            |
            StatusCode::NotAcceptable       |
            StatusCode::Conflict            |
            StatusCode::InternalServerError => Err(ErrorKind::HyperFault(res.status).into()),
            _ => unreachable!(),
        }
    }
}

/// Extract the error message content from an HTTP response that
/// contains a Docker JSON error structure.
fn get_error_message(res: &mut Response) -> Option<String> {
    let mut output = String::new();

    if res.read_to_string(&mut output).is_ok() {
        json::Json::from_str(output.as_str())
            .ok()
            .as_ref()
            .and_then(|x| x.as_object())
            .and_then(|x| x.get("message"))
            .and_then(|x| x.as_string())
            .map(|x| x.to_owned())
    } else {
        None
    }
}
