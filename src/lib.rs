extern crate rand;
extern crate bit_vec;

pub mod thread_pool;
pub mod http;
pub mod server;
pub mod endpoint;

pub use http::{Verb as HTTPVerb, Response as HTTPResponse, Request as HTTPRequest};
pub use server::Server as HTTPServer;

#[derive(Debug)]
pub enum Error {
    RequestParse{msg: &'static str, data: Vec<u8>},
    InvalidHeader{msg: &'static str},
    IOError(std::io::Error),
    InvalidEndpoint,
    DuplicateEndpoint,
    URLParse
}

#[cfg(test)]
mod tests {
    use crate::endpoint::{URLBindings, new_endpoint as e};
    use super::*;

    #[test]
    fn test_server() {
        let mut server = HTTPServer::new()
            .with_endpoint(e!(GET, "/random"), gen_rand)
            .with_endpoint(e!(GET, "/trash/random"), gen_rand)
            .with_endpoint(e!(GET, "/trash/peggle2/random"), gen_rand)
            .with_endpoint(e!(GET, "/print/<value>"), print)
            .with_endpoint(e!(GET, "/print/<value>/<index>"), print_index)
            .with_endpoint(e!(GET, "/print/color/<color>/<value>"), print_color);

        server.run();
    }

    fn gen_rand(_: &HTTPRequest, _: &URLBindings) -> Option<HTTPResponse> {
        let random = rand::random::<usize>();

        return Some(HTTPResponse::from_text("text/html", format!("<html><body><h1>{}</h1></body></html>", random).as_str()))
    }

    fn print(_: &HTTPRequest, bindings: &URLBindings) -> Option<HTTPResponse> {
        Some(
            HTTPResponse::from_text(
                "mime/html",
                &format!(
                    "<html><body><h1>{}</h1></body></html>",
                    bindings.get(&String::from("value"))?
                )
            )
        )
    }

    fn print_index(_: &HTTPRequest, bindings: &URLBindings) -> Option<HTTPResponse> {
        Some(
            HTTPResponse::from_text(
                "text/html",
                &format!(
                    "<html><body><h1>{}</h1></body></html>",
                    bindings.get(&String::from("value"))?.chars().nth(
                        bindings.get(&"index".to_string())?.parse::<usize>().ok()?
                    )?
                )
            )
        )
    }

    fn print_color(_: &HTTPRequest, bindings: &URLBindings) -> Option<HTTPResponse> {
        Some(
            HTTPResponse::from_text(
                "text/html",
                &format!(
                    "<html><body><h1 style=\"color: {}\">{}</h1></body></html>",
                    bindings.get("color")?,
                    bindings.get("value")?
                )
            )
        )
    }
}
