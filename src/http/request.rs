use std::io::{BufRead};
use std::convert::TryFrom;

use crate::{Error, HTTPVerb, URL};
use super::{Header, Method};

#[derive(Debug)]
pub struct Request {
    verb: HTTPVerb,
    url: URL,
    header: Header,
    body: Vec<u8>
}

impl Request {
    pub fn from_stream<F: BufRead>(stream: &mut F) -> Result<Self, Error> {
        let mut line = String::new();
        let mut lines = Vec::new();

        loop {
            match stream.read_line(&mut line) {
                Ok(n) if n > 0 => {
                    if line == "\r\n" {
                        break;
                    } else {
                    lines.push(line.clone());
                    line.clear();
                    }
                }

                Ok(_) => {
                    return Err(Error::ConnectionClosed);
                }

                Err(e) => {
                    return Err(Error::IOError(e));
                }
            }
        }

        let (verb, url) = {
            let top = lines.get(0).ok_or(Error::RequestParse)?;
            let top: Vec<&str> = top.split(" ").collect();
            let verb = Method::try_from(top[0])
                .map_err(|_| Error::RequestParse)?;
            let resource = top[1].to_string();
            (verb, URL::from_string(&resource)?)
        };

        let mut header = Header::new();
        for line in lines.drain(..).skip(1) {
            let colon = line.find(":").unwrap();
            let key = line[0..colon].to_owned();
            let value = line[colon + 1 ..].trim().to_owned();
            header.insert(key, value);
        }

        let body = {
            let content_length: usize = header.get("Content-Length")
                .unwrap_or(&"0".to_owned())
                .parse()
                .map_err(|_| Error::InvalidHeader)?;
            let mut data = vec![0; content_length];
            stream.read_exact(&mut data[..]).map_err(|e| Error::IOError(e))?;
            data
        };

        let req = Self {
            verb,
            url,
            header,
            body
        };

        return Ok(req);
    }

    pub fn verb(&self) -> Method {
        self.verb
    }

    pub fn url(&self) -> &URL {
        &self.url
    }
}