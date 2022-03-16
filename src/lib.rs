extern crate rand;
extern crate bit_vec;

mod thread_pool;
mod url;

pub use url::*;
pub mod http;
pub mod server;

#[cfg(test)]
mod tests {
    use crate::http::WebService;
    use crate::server::Server;

    #[test]
    fn test_server() {
        let mut server = Server::new(WebService::new());
        server.run();
    }
}
