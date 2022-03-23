use std::borrow::Borrow;
use std::io::{BufRead, BufReader, Read};
use std::convert::TryFrom;

use super::*;
use crate::url::URL;

#[derive(Debug)]
pub struct Request {
    verb: Method,
    url: URL,
    header: Header,
    body: Vec<u8>
}

impl Request {
    pub fn from_stream<F: Read>(stream: &mut F) -> Result<Self, Error> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let mut lines = Vec::new();

        loop {
            match reader.read_line(&mut line) {
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
            let resource = URL::from_string(top[1]).map_err(|_| Error::URLParse)?;
            (verb, resource)
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
            reader.read_exact(&mut data[..]).map_err(|e| Error::IOError(e))?;
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

    pub fn header(&self, key: impl Borrow<str>) -> Option<&str> {
        self.header.get(key.borrow()).map(|s| s.as_str())
    }
}