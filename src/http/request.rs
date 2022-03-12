use std::io::{BufRead};
use std::convert::TryFrom;

use crate::{Error, HTTPVerb};
use super::{Header, Verb};

#[derive(Debug)]
pub struct Request {
    verb: HTTPVerb,
    resource: String,
    header: Header,
    body: Vec<u8>
}

impl Request {
    pub fn from_stream<F: BufRead>(stream: &mut F) -> Result<Self, Error> {
        let mut line = String::new();
        let mut lines = Vec::new();
        
        while let Ok(_) = stream.read_line(&mut line) {
            if line == "\r\n" {
                break;
            } else {
                lines.push(line.clone());
                line.clear();
            }
        }

        let (verb, resource) = {
            let top = lines.remove(0);
            let top: Vec<&str> = top.split(" ").collect();
            let verb = Verb::try_from(top[0])
                .map_err(|_| Error::RequestParse{
                    msg: "Invalid HTTP Verb in request",
                    data: lines.iter().map(|l| l.as_bytes().iter()).flatten().map(|b| *b).collect()
                })?;
            let resource = top[1].to_string();
            (verb, resource)
        };

        let mut header = Header::new();
        for line in lines.drain(..) {
            let colon = line.find(":").unwrap();
            let key = line[0..colon].to_owned();
            let value = line[colon + 1 ..].trim().to_owned();
            header.insert(key, value);
        }

        let body = {
            let content_length: usize = header.get("Content-Length")
                .unwrap_or(&"0".to_owned())
                .parse()
                .map_err(|_| Error::InvalidHeader {msg: "Header element Content-Length had invalid value"})?;
            let mut data = vec![0; content_length];
            stream.read_exact(&mut data[..]).map_err(|e| Error::IOError(e))?;
            data
        };

        let req = Self {
            verb,
            resource,
            header,
            body
        };

        return Ok(req);
    }

    pub fn verb(&self) -> Verb {
        self.verb
    }

    pub fn resource(&self) -> &String {
        &self.resource
    }
}