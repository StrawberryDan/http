extern crate rand;
extern crate bit_vec;
extern crate openssl;

mod thread_pool;

pub mod mime;
pub mod url;
pub mod ws;
pub mod http;
pub mod server;

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use crate::http::WebService;
    use crate::server::Server;
    use crate::ws::WebSocketService;

    #[test]
    fn webserver() {
        let mut server = Server::new(WebService::new());
        server.run();
    }

    #[test]
    fn websocket() {
        use std::thread::spawn;

        let mut httpserver = Server::new(WebService::new())
            .with_tls(true);

        let mut wsserver   = Server::new(WebSocketService{})
            .with_tls(true)
            .with_socket(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081));

        let httpserver = spawn(move || httpserver.run());
        let wsserver = spawn(move || wsserver.run());

        httpserver.join().unwrap();
        wsserver.join().unwrap();
    }
}
