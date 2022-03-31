use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::thread_pool::ThreadPool;

pub struct Server<H: WebService + Send + Sync + 'static> {
    socket: SocketAddr,
    handler: Arc<H>,
    certificate: Option<PathBuf>,
    key: Option<PathBuf>,
}

impl<H: WebService + Send + Sync + 'static> Server<H> {
    pub fn new(handler: H) -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            handler: Arc::new(handler),
            certificate: None,
            key: None,
        }
    }

    pub fn with_socket(self, socket: SocketAddr) -> Self {
        Self { socket, ..self }
    }

    pub fn with_address(self, addr: IpAddr) -> Self {
        let socket = SocketAddr::new(addr, self.socket.port());
        Self { socket, ..self }
    }

    pub fn with_port(self, port: u16) -> Self {
        let socket = SocketAddr::new(self.socket.ip(), port);
        Self { socket, ..self }
    }

    pub fn with_certificate<P: AsRef<Path>>(self, certificate: Option<P>) -> Self {
        Self { certificate: certificate.map(|x| x.as_ref().to_path_buf()), ..self }
    }

    pub fn with_key<P: AsRef<Path>>(self, key: Option<P>) -> Self {
        Self { key: key.map(|x| x.as_ref().to_path_buf()), ..self }
    }

    pub fn run(&mut self) {
        let mut threads = ThreadPool::new();
        let listener = TcpListener::bind(self.socket).unwrap();
        println!("Listening on {}:{}", self.socket.ip(), self.socket.port());

        loop {
            let handler = self.handler.clone();
            match listener.accept() {
                Ok((con, addr)) => {
                    threads
                        .submit(move || handler.handle_connection(con, addr))
                        .unwrap();
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }

    pub fn run_secure(&mut self) {
        let mut threads = ThreadPool::new();
        let listener = TcpListener::bind(self.socket).unwrap();
        println!("Listening on {}:{}", self.socket.ip(), self.socket.port());

        let tls = {
            let mut tls = openssl::ssl::SslAcceptor::mozilla_intermediate_v5(openssl::ssl::SslMethod::tls()).unwrap();
            tls.set_certificate_file(self.certificate.as_ref().unwrap_or(&PathBuf::from("cert.pem")), openssl::ssl::SslFiletype::PEM)
                .expect("Expected certificate file \"cert.pem\" in working directory!");
            tls.set_private_key_file(self.key.as_ref().unwrap_or(&PathBuf::from("key.pem")), openssl::ssl::SslFiletype::PEM)
                .expect("Expected key file \"key.pem\" in working directory!");
            tls.build()
        };

        loop {
            let handler = self.handler.clone();
            match listener.accept() {
                Ok((con, addr)) => {
                    let con = match tls.accept(con) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    threads
                        .submit(move || handler.handle_connection(con, addr))
                        .unwrap();
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
}

pub trait WebService {
    fn handle_connection(&self, con: impl Read + Write, client: SocketAddr);
}
