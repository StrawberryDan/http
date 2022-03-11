pub mod thread_pool;
pub mod http;
pub mod server;

pub use http::Verb as HTTPVerb;
pub use server::Server as HTTPServer;

pub type Endpoint = (HTTPVerb, String);

#[test]
fn test_server() {
    let mut server = HTTPServer::new();
    server.run();
}