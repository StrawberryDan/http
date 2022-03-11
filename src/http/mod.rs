pub mod request;
pub mod response;

pub use request::*;
pub use response::*;

use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum Verb {
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

impl TryFrom<&str> for Verb {
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

#[derive(Debug)]
pub enum Error {
    RequestParse{msg: &'static str, data: Vec<u8>},
    InvalidHeader{msg: &'static str},
    IOError(std::io::Error),
}