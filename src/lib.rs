extern crate rand;
extern crate bit_vec;

pub mod thread_pool;
pub mod http;
pub mod endpoint;

pub use http::{Request as HTTPRequest, Response as HTTPResponse, Method as HTTPVerb};
pub use http::Server as HTTPServer;

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
    use crate::endpoint::{endpoint as e, URLBindings};
    use crate::http::{DefaultHandler};
    use super::*;

    #[test]
    fn test_server() {
        let handler = DefaultHandler::new()
            .with_endpoint(e!(GET, "/random"), gen_rand)
            .with_endpoint(e!(GET, "/print/<value>"), print)
            .with_endpoint(e!(GET, "/print/<value>/<index>"), print_index)
            .with_endpoint(e!(GET, "/print/color/<color>/<value>"), print_color);

        let mut server = HTTPServer::new(handler);
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
