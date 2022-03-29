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

use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::SocketAddr;

use crate::server::WebService;

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
    endpoints: EndpointTable,
}

impl WebServer {
    pub fn new() -> Self {
        Self {
            endpoints: EndpointTable::new(),
        }
    }

    pub fn with_endpoint<H: EndpointFunction + Send + Sync + 'static>(mut self, endpoint: Endpoint, handler: H) -> Self {
        self.endpoints.add(endpoint, Box::new(handler));
        self
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

            let response = self
                .endpoints
                .find_match(req.method(), req.url())
                .map(|(h, b)| h.handle(req.clone(), b))
                .unwrap_or(Response::new(404));


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
