extern crate bit_vec;
extern crate openssl;
extern crate rand;

mod thread_pool;

pub mod http;
pub mod mime;
pub mod server;
pub mod url;
pub mod ws;

#[cfg(test)]
mod tests {
    use crate::http::WebService;
    use crate::server::Server;
    use crate::ws::WebSocketService;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn webserver() {
        let mut server = Server::new(WebService::new());
        server.run_secure();
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
