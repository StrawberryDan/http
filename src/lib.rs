extern crate rand;
extern crate bit_vec;

pub mod thread_pool;
pub mod http;
pub mod url;

pub use http::{Method as HTTPVerb, Request as HTTPRequest, Response as HTTPResponse, Server as HTTPServer};
pub use url::URL;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server() {
        let mut server = HTTPServer::default_web();
        server.run();
    }
}
