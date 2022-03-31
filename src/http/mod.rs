// Modules
mod endpoint;
mod request;
mod response;
mod stream;
mod cookie;
mod header;

// Exports
pub use header::*;
pub use cookie::*;
pub use request::*;
pub use response::*;
pub use stream::*;
pub use endpoint::*;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use crate::server::WebService;
use crate::url::URL;

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Eq, Ord)]
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
        match from[..].to_uppercase().as_str() {
            "GET" => Ok(Self::GET),
            "HEAD" => Ok(Self::HEAD),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(()),
        }
    }
}

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

pub struct WebServer {
    root: PathBuf,
    endpoints: EndpointTable,
    file_masks: HashMap<PathBuf, Box<dyn FileResponder + Send + Sync + 'static>>,
}

impl WebServer {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("./"),
            endpoints: EndpointTable::new(),
            file_masks: HashMap::new(),
        }
    }

    pub fn with_root(self, root: impl AsRef<Path>) -> Self {
        Self { root: std::fs::canonicalize(root.as_ref()).unwrap(), ..self }
    }

    pub fn with_endpoint<S, H>(mut self, method: Method, endpoint: &S, handler: H) -> Self
        where S: Borrow<str> + ?Sized, H: EndpointResponder + Send + Sync + 'static
    {
        self.endpoints.add(Endpoint::new(method, endpoint.borrow()), Box::new(handler));
        self
    }

    pub fn with_file_mask<P, H>(mut self, file: &P, handler: H) -> Self
        where P: AsRef<Path> + ?Sized, H: FileResponder + Send + Sync + 'static
    {
        let path = self.root.join(file.as_ref());
        let path = std::fs::canonicalize(path).unwrap();
        self.file_masks.insert(path, Box::new(handler));
        self
    }

    pub fn handle_file_request(&self, req: Request) -> Option<Response> {
        let path = self.find_requested_path(req.url())?;

        if let Some(handler) = self.file_masks.get(&path) {
            return Some(handler.response(req, path));
        }

        return match &req.method() {
            Method::GET => {
                Response::from_file(200, None, path).ok()
            }

            Method::TRACE => {
                Some(Response::from_file(200, None, path).ok()?.with_body("application/octet-stream", Vec::new()))
            }

            _ => None,
        };
    }

    fn find_requested_path(&self, url: &URL) -> Option<PathBuf> {
        let resource = if url.resource().len() == 0 {
            PathBuf::from("index")
        } else {
            PathBuf::from(url.resource_string())
        };

        let mut path = self.root.join(resource);

        if !path.exists() {
            let stem = path.file_stem()?;
            if path.file_stem().is_some() && path.extension().is_none() {
                let dir = path.parent()?.read_dir().ok()?;
                let candidates: Vec<_> = dir
                    .filter_map(|f| f.ok().map(|f| f.path()))
                    .filter(|f| f.file_stem().map(|f| f == stem).unwrap_or(false))
                    .collect();
                if candidates.is_empty() {
                    return None;
                }

                path = candidates[0].clone();
            }
        }

        let canonicalised = std::fs::canonicalize(path).ok()?;
        if !canonicalised.starts_with(&self.root) {
            return None;
        }

        return Some(canonicalised);
    }

    fn not_found_response() -> Response {
            Response::from_text(404, "text/html", "<html><body><h1>Not Found</h1></body></html>")
    }
}

impl WebService for WebServer {
    fn handle_connection(&self, con: impl Read + Write, client: SocketAddr) {
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

            let callback = self
                .endpoints
                .find_match(req.method(), req.url())
                .map(|(h, b)| h.response(req.clone(), b));

            let response = match callback {
                Some(res) => res,
                None => self.handle_file_request(req)
                    .unwrap_or(WebServer::not_found_response()),
            };

            match stream.send(response) {
                Ok(_) => (),
                Err(Error::ConnectionClosed) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Error sending response! Error: {:?}", e);
                    break;
                }
            }
        }

        println!("Stopped serving client: {}", client);
    }
}

pub trait FileResponder {
    fn response(&self, req: Request, file: PathBuf) -> Response;
}