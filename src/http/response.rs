use super::Error;
use super::Header;
use crate::mime::extension_to_mime;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::http::Cookie;

#[derive(Debug, Clone)]
pub struct Response {
    code: usize,
    header: Header,
    body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Response {
            code: 500,
            header: Header::new(),
            body: Vec::new(),
        }.with_header("Content-Length", "0")
    }

    pub fn with_code(self, code: usize) -> Self {
        Self { code, ..self }
    }

    pub fn with_header(mut self, key: &str, value: impl Borrow<str>) -> Self {
        self.header.add(key, value);
        return self;
    }

    pub fn with_cookie(mut self, cookie: &Cookie) -> Self {
        self.header.add("Set-Cookie", cookie.to_string());
        return self;
    }

    pub fn with_body(mut self, content_type: &str, body: Vec<u8>) -> Self {
        let len = body.len();
        self.header.replace("Content-Type", content_type);
        self.header.replace("Content-Length", len.to_string());
        Self { body, ..self }
    }

    pub fn from_text(mime: &str, text: &str) -> Self {
        Self::new()
            .with_code(200)
            .with_body(mime, text.as_bytes().to_vec())
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut file = File::open(path.as_ref()).map_err(|e| Error::IOError(e))?;
        let mime = extension_to_mime(
            path.as_ref()
                .extension()
                .map(|s| s.to_str())
                .flatten()
                .unwrap_or(""),
        );

        let mut body = Vec::new();
        file.read_to_end(&mut body).map_err(|e| Error::IOError(e))?;

        Ok(Response::new()
            .with_code(200)
            .with_body(mime, body))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut string = String::new();

        string += &format!("HTTP/1.1 {}\r\n", self.code.to_string());

        for (key, value) in &self.header {
            string += &format!("{}: {}\r\n", key, value);
        }

        string += "\r\n";

        let mut bytes = string.as_bytes().to_owned();
        bytes.append(&mut self.body.clone());

        return bytes;
    }
}
