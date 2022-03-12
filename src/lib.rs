extern crate rand;
extern crate bit_vec;

pub mod thread_pool;
pub mod http;
pub mod server;
pub mod endpoint;

use std::convert::TryInto;
pub use http::{Verb as HTTPVerb, Response as HTTPResponse, Request as HTTPRequest};
pub use server::Server as HTTPServer;
use crate::endpoint::{Endpoint, URLBindings, URLSegment};
use crate::http::Verb;

#[test]
fn test_server() {
    let e = Endpoint::new(Verb::GET, "/random").unwrap();

    let mut server = HTTPServer::new()
        .with_endpoint(Endpoint::new(Verb::GET, "/random").unwrap(), Box::new(gen_rand)).unwrap()
        .with_endpoint(Endpoint::new(Verb::GET, "/trash/random").unwrap(), Box::new(gen_rand)).unwrap()
        .with_endpoint(Endpoint::new(Verb::GET, "/trash/peggle2/random").unwrap(), Box::new(gen_rand)).unwrap()
        .with_endpoint(Endpoint::new(Verb::GET, "/print/<value>").unwrap(), Box::new(print)).unwrap()
        .with_endpoint(Endpoint::new(Verb::GET, "/print/<value>/<index>").unwrap(), Box::new(print_index)).unwrap()
        .with_endpoint(Endpoint::new(Verb::GET, "/print/red/<value>").unwrap(), Box::new(print_red)).unwrap();


    server.run();
}

fn gen_rand(req: &HTTPRequest, _: &URLBindings) -> Option<HTTPResponse> {
    let random = rand::random::<usize>();

    return Some(HTTPResponse::from_html(format!("<html><body><h1>{}</h1></body></html>", random).as_str()))
}

fn print(req: &HTTPRequest, bindings: &URLBindings) -> Option<HTTPResponse> {
    Some(
        HTTPResponse::from_html(
            &format!(
                "<html><body><h1>{}</h1></body></html>",
                bindings.get(&String::from("value")).unwrap()
            )
        )
    )
}

fn print_index(req: &HTTPRequest, bindings: &URLBindings) -> Option<HTTPResponse> {
    Some(
        HTTPResponse::from_html(
            &format!(
                "<html><body><h1>{}</h1></body></html>",
                bindings.get(&String::from("value")).unwrap().chars().nth(
                    bindings.get(&"index".to_string()).unwrap().parse::<usize>().unwrap()
                ).unwrap()
            )
        )
    )
}

fn print_red(_: &HTTPRequest, bindings: &URLBindings) -> Option<HTTPResponse> {
    Some(
        HTTPResponse::from_html(
            &format!(
                "<html><body><h1 style=\"color: red\">{}</h1></body></html>",
                bindings.get("value").unwrap()
            )
        )
    )
}

#[derive(Debug)]
pub enum Error {
    RequestParse{msg: &'static str, data: Vec<u8>},
    InvalidHeader{msg: &'static str},
    IOError(std::io::Error),
    InvalidEndpoint,
    DuplicateEndpoint,
}
