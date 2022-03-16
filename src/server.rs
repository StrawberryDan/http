use std::{sync::Arc};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use crate::thread_pool::ThreadPool;

pub struct Server<H: Service + Send + Sync + 'static> {
    socket: SocketAddr,
    handler: Arc<H>
}

impl<H: Service + Send + Sync + 'static> Server<H> {
    pub fn new(handler: H) -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            handler: Arc::new(handler),
        }
    }

    pub fn with_socket(self, socket: SocketAddr) -> Self {
        Self { socket, .. self}
    }

    pub fn run(&mut self) {
        let mut threads = ThreadPool::new();
        let listener = TcpListener::bind(self.socket).unwrap();
        println!("Listening on {}:{}", self.socket.ip(), self.socket.port());

        loop {
            match listener.accept() {
                Ok((con, addr)) => {
                    let handler = self.handler.clone();
                    threads.submit(move|| handler.handle_connection(con, addr) ).unwrap();
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
}

pub trait Service {
    fn handle_connection(&self, con: TcpStream, client: SocketAddr);
}