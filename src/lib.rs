extern crate openssl;
extern crate rand;
extern crate core;

mod thread_pool;

pub mod http;
pub mod mime;
pub mod server;
pub mod url;
pub mod ws;

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use crate::http::{Bindings, Endpoint, EndpointFunction, Request, Response, WebService};
    use crate::server::{Server, Service};
    use crate::ws::Message;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use crate::http::Method::GET;

    struct Printer {}

    impl EndpointFunction for Printer {
        fn handle(&self, _: Request, bindings: Bindings) -> Option<Response> {
            Some(Response::from_text("text/plain", bindings.get("text").unwrap()))
        }
    }

    struct ColorPrinter {}

    impl EndpointFunction for ColorPrinter {
        fn handle(&self, _: Request, bindings: Bindings) -> Option<Response> {
            Some(Response::from_text("text/html",
                                     &format!("<html><body><h1 style=\"color:{}\">{}</h1></body></html>",
                                              bindings.get("color").unwrap(),
                                              bindings.get("text").unwrap(),
                                     ),
            ))
        }
    }

    #[test]
    fn webserver() {
        let service = WebService::new()
            .with_root("./site")
            .with_endpoint(Endpoint::new(GET, "/print/<text>"), Printer {})
            .with_endpoint(Endpoint::new(GET, "/print/<color>/<text>"), ColorPrinter {});
        let mut server = Server::new(service);
        server.run_secure();
    }

    pub struct WebSocketService {}

    impl Service for WebSocketService {
        fn handle_connection(&self, con: impl Read + Write, client: SocketAddr) {
            use std::io::ErrorKind::ConnectionAborted;
            use crate::http::Stream as HTTPStream;
            use crate::ws::Error;
            use crate::ws::Stream;

            let mut stream = Stream::await_handshake(HTTPStream::new(con)).unwrap();

            loop {
                let frame = match stream.recv() {
                    Ok(f) => f,
                    Err(Error::IOError(e)) if matches!(e.kind(), ConnectionAborted) => break,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        break;
                    }
                };

                stream.send(Message::String("Funny Monkey!".to_string())).unwrap();
            }
        }
    }

    #[test]
    fn websocket() {
        use std::thread::spawn;

        let mut httpserver = Server::new(WebService::new());

        let mut wsserver = Server::new(WebSocketService {})
            .with_socket(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081));

        let httpserver = spawn(move || httpserver.run_secure());
        let wsserver = spawn(move || wsserver.run_secure());

        httpserver.join().unwrap();
        wsserver.join().unwrap();
    }
}
