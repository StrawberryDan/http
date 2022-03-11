use super::Verb;
use crate::Endpoint;
use std::io::{Read, BufRead, BufReader};
use std::convert::TryFrom;
use super::Header;

#[derive(Debug)]
pub struct Request {
    endpoint: Endpoint,
    header: Header,
    body: Vec<u8>
}

impl Request {
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub fn from_reader<F: Read>(reader: &mut F) -> Result<Self, ()> {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let mut lines = Vec::new();
        
        while let Ok(_) = reader.read_line(&mut line) {
            if line == "\r\n" {
                break;
            } else {
                lines.push(line.clone());
                line.clear();
            }
        }

        let endpoint = {
            let top = lines.remove(0);
            let top: Vec<&str> = top.split(" ").collect();
            (Verb::try_from(top[0]).unwrap(), top[1].to_string())
        };

        let mut header = Header::new();
        for line in lines.drain(..) {
            let colon = line.find(":").unwrap();
            let key = line[0..colon].to_owned();
            let value = line[colon + 1 ..].trim().to_owned();
            header.insert(key, value);
        }

        let mut body = {
            let content_length: usize = header.get("Content-Length")
                .unwrap_or(&"0".to_owned())
                .parse().map_err(|_| ())?;
            let mut data = vec![0; content_length];
            reader.read_exact(&mut data[..]).map_err(|_| ())?;
            data
        };

        let req = Self {
            endpoint,
            header,
            body
        };

        return Ok(req);
    }
}