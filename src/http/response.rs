use super::*;

use std::fs::File;
use std::io::Read;
use super::{Header, Error};

pub struct Response {
    code: usize,
    header: Header,
    body: Vec<u8>
}

impl Response {
    pub fn new() -> Self {
        Response {
            code: 500,
            header: HashMap::new(),
            body: Vec::new()
        }
    }

    pub fn with_code(self, code: usize) -> Self {
        Self { code, .. self }
    }

    pub fn with_header(self, header: Header) -> Self {
        Self { header, .. self }
    }

    pub fn with_header_entry(mut self, key: &str, value: &str) -> Self {
        self.header.insert(key.to_owned(), value.to_owned());
        return self;
    }

    pub fn with_body(self, body: Vec<u8>) -> Self {
        Self { body, .. self }
    }

    pub fn from_html(html: &str) -> Self {
        Self::new().with_code(200)
            .with_header_entry("Content-Type", "text/html")
            .with_header_entry("Content-Length", html.as_bytes().len().to_string().as_str())
            .with_body(html.as_bytes().to_vec())
    }

    pub fn from_file(mime: &str, mut file: File) -> Result<Self, Error> {
        let mut body = Vec::new();
        file.read_to_end(&mut body).map_err(|e| Error::IOError(e))?;

        let mut header = Header::new();
        header.insert(
            String::from("Content-Type"),
            String::from(mime)
        );

        header.insert(
            String::from("Content-Length"),
            body.len().to_string()
        );

        Ok(
            Response::new()
                .with_code(200)
                .with_header(header)
                .with_body(body)
        )
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut string = String::new();

        string += &format!("HTTP/1.1 {}\n", self.code.to_string());

        for (key, value) in &self.header {
            string += &format!("{}: {}\n", key, value);
        }

        string += "\r\n";

        let mut bytes = string.as_bytes().to_owned();
        bytes.append(&mut self.body.clone());

        return bytes;
    }
}