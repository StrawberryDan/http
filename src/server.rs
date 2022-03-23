use std::{sync::Arc};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

use crate::thread_pool::ThreadPool;

pub struct Server<H: Service + Send + Sync + 'static> {
    socket: SocketAddr,
    handler: Arc<H>,
    tls: bool,
}

impl<H: Service + Send + Sync + 'static> Server<H> {
    pub fn new(handler: H) -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            handler: Arc::new(handler),
            tls: false,
        }
    }

    pub fn with_socket(self, socket: SocketAddr) -> Self {
        Self { socket, .. self}
    }

    pub fn with_tls(self, tls: bool) -> Self {
        Self { tls, .. self }
    }

    pub fn run(&mut self) {
        let mut threads = ThreadPool::new();
        let listener = TcpListener::bind(self.socket).unwrap();
        println!("Listening on {}:{}", self.socket.ip(), self.socket.port());

        let tls = match self.tls {
            true => {
                let mut tls = openssl::ssl::SslAcceptor::mozilla_intermediate_v5(openssl::ssl::SslMethod::tls()).unwrap();
                tls.set_certificate_file("cert.pem", openssl::ssl::SslFiletype::PEM).unwrap();
                tls.set_private_key_file("key.pem", openssl::ssl::SslFiletype::PEM).unwrap();
                Some(tls.build())
            },
            false => None,
        };

        loop {
            let handler = self.handler.clone();
            match listener.accept() {
                Ok((con, addr)) => {
                    match &tls {
                        None => {
                            threads.submit( move || handler.handle_connection(con, addr) ).unwrap();
                        }

                        Some(tls) => {
                            let con = match tls.accept(con) {
                                Ok(c) => c,
                                Err(_) => continue,
                            };

                            threads.submit( move || handler.handle_connection(con, addr) ).unwrap();
                        }
                    }
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
}

pub trait Service {
    fn handle_connection(&self, con: impl Read + Write, client: SocketAddr);
}