mod request;
mod response;
mod endpoint;
mod stream;

pub use stream::*;
pub use request::*;
pub use response::*;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use endpoint::*;

use crate::server::Service;
use crate::URL;

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

#[derive(Debug)]
pub enum Error {
    RequestParse,
    InvalidHeader,
    IOError(std::io::Error),
    InvalidEndpoint,
    DuplicateEndpoint,
    URLParse,
    ConnectionClosed,
}

pub struct WebService {
    endpoints: EndpointTable,
}

impl WebService {
    pub fn new() -> Self {
        Self {
            endpoints: EndpointTable::new()
        }
    }

    pub fn with_endpoint(mut self, endpoint: Endpoint, callback: Callback) -> Self {
        self.endpoints.add(endpoint, callback).unwrap();
        self
    }

    pub fn handle_file_request(req: &Request, _: &Bindings) -> Option<Response> {
        return match &req.verb() {
            Method::GET => {
                let path = Self::find_requested_path(req.url())?;
                Response::from_file(path).ok()
            }

            Method::TRACE => {
                let path = Self::find_requested_path(req.url())?;
                Some(Response::from_file(path).ok()?.with_body(Vec::new()))
            }

            _ => {
                None
            }
        }
    }

    fn find_requested_path(url: &URL) -> Option<PathBuf> {
        let mut path = if url.resource() == "/" {
            PathBuf::from("./index")
        } else {
            PathBuf::from(format!(".{}", url.resource()))
        };

        if !path.exists() {
            let stem = path.file_stem()?;
            if path.file_stem().is_some() && path.extension().is_none() {
                let dir = path.parent()?.read_dir().ok()?;
                let candidates: Vec<_> = dir
                    .filter(|f| f.is_ok())
                    .map(|f| unsafe { f.unwrap_unchecked().path() } )
                    .filter(|f| f.file_stem().map(|f| f == stem).unwrap_or(false))
                    .collect();
                if candidates.is_empty() { return None; }
                path = candidates[0].clone();
            }
        }

        return Some(path);
    }

    fn not_found_response(_: &Request, _: &Bindings) -> Option<Response> {
        Some(Response::from_text("text/html", "<html><body><h1>Not Found</h1></body></html>").with_code(404))
    }
}

impl Service for WebService {
    fn handle_connection(&self, con: TcpStream, client: SocketAddr) {
        println!("Started serving client: {}", client);
        let mut stream = Stream::new(con);
        loop {
            let req = match stream.recv() {
                Ok(x) => x,
                Err(e) => {
                    if let Error::ConnectionClosed = e {
                        break;
                    } else {
                        eprintln!("Error receiving request! Error: {:?}", e);
                        break;
                    }
                }
            };

            let callback = self.endpoints.find_match(req.url()).map(|(c, b)| c(&req, &b)).flatten();

            let response = match callback {
                Some(res) => res,
                None => Self::handle_file_request(&req, &HashMap::new()).or(Self::not_found_response(&req, &HashMap::new())).unwrap()
            };

            match stream.send(response) {
                Ok(_) => (),
                Err(Error::ConnectionClosed) => { break; }
                Err(e) =>  {
                    eprintln!("Error sending response! Error: {:?}", e);
                    break;
                }
            }
        }

        println!("Stopped serving client: {}", client);
    }
}

pub fn file_mime<R: AsRef<Path>>(path: R) -> Result<String, ()> {
    let path = path.as_ref();
    String::from_utf8(
        std::process::Command::new("file")
            .arg("--mime-type").arg("-b").arg(path.to_str().unwrap())
            .output().map_err(|_| ())?.stdout
    ).map(|s| s.trim().to_string()).map_err(|_| ())
}
