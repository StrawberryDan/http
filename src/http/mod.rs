mod request;
mod response;
mod server;

pub use request::*;
pub use response::*;
pub use server::*;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::Path;

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl TryFrom<&str> for Method {
    type Error = ();

    fn try_from(from: &str) -> Result<Self, Self::Error> {
        match &from[..] {
            "GET" => Ok(Self::GET),
            "HEAD" => Ok(Self::HEAD),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(())
        }
    }
}

pub type Header = HashMap<String, String>;

pub fn file_mime<R: AsRef<Path>>(path: R) -> Result<String, ()> {
    let path = path.as_ref();
    String::from_utf8(
        std::process::Command::new("file")
            .arg("--mime-type").arg("-b").arg(path.to_str().unwrap())
            .output().map_err(|_| ())?.stdout
    ).map(|s| s.trim().to_string()).map_err(|_| ())
}
