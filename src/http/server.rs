use std::{path::PathBuf, sync::Arc};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::io::{BufReader, BufWriter, Write};

use crate::thread_pool::ThreadPool;
use crate::http::{Request, Response, Method};
use crate::http::endpoint::{Endpoint, ParseTree as EndpointTree, Bindings as EndpointBindings};
use crate::{Error, URL};
use crate::http::endpoint::Callback as EndpointCallback;

pub struct Server<H: Handler + Send + Sync + 'static> {
    socket: SocketAddr,
    handler: Arc<H>
}

impl<H: Handler + Send + Sync + 'static> Server<H> {
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
                    threads.submit(move|| handler.handle(con, addr) ).unwrap();
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
}

pub trait Handler {
    fn handle(&self, con: TcpStream, client: SocketAddr);
}

pub struct DefaultHandler {
    endpoints: EndpointTree,
}

impl DefaultHandler {
    pub fn new() -> Self {
        Self {
            endpoints: EndpointTree::new()
        }
    }

    pub fn with_endpoint(mut self, endpoint: Endpoint, callback: EndpointCallback) -> Self {
        self.endpoints.add(endpoint, callback).unwrap();
        self
    }

    pub fn handle_file_request(req: &Request, _: &EndpointBindings) -> Option<Response> {
        return match &req.verb() {
            Method::GET => {
                let path = Self::find_requested_path(req.url())?;
                Response::from_file(path).ok()
            }

            _ => {
                None
            }
        }
    }

    fn find_requested_path(url: &URL) -> Option<PathBuf> {
        let mut path = if url.resource() == "/" {
            PathBuf::from("./index")
        } else {
            PathBuf::from(format!(".{}", url.resource()))
        };

        if !path.exists() {
            let stem = path.file_stem()?;
            if path.file_stem().is_some() && path.extension().is_none() {
                let dir = path.parent()?.read_dir().ok()?;
                let candidates: Vec<_> = dir
                    .filter(|f| f.is_ok())
                    .map(|f| unsafe { f.unwrap_unchecked().path() } )
                    .filter(|f| f.file_stem().map(|f| f == stem).unwrap_or(false))
                    .collect();
                if candidates.is_empty() { return None; }
                path = candidates[0].clone();
            }
        }

        return Some(path);
    }

    fn not_found_response(_: &Request, _: &EndpointBindings) -> Option<Response> {
        Some(Response::from_text("text/html", "<html><body><h1>Not Found</h1></body></html>").with_code(404))
    }
}

impl Handler for DefaultHandler {
    fn handle(&self, con: TcpStream, client: SocketAddr) {
        println!("Started serving client: {}", client);
        loop {
            let mut reader =  BufReader::new(con.try_clone().unwrap());
            let req = match Request::from_stream(&mut reader) {
                Ok(x) => x,
                Err(e) => {
                    if let Error::ConnectionClosed = e {
                        println!("Stopped serving client: {}", client);
                    } else {
                        eprintln!("Error receiving request! Error: {:?}", e);
                    }
                    return;
                }
            };

            let mut res = BufWriter::new(con.try_clone().unwrap());

            let callback = self.endpoints.find_match(req.url()).map(|(c, b)| c(&req, &b)).flatten();

            let response = match callback {
                Some(res) => res,
                None => Self::handle_file_request(&req, &HashMap::new()).or(Self::not_found_response(&req, &HashMap::new())).unwrap()
            };

            res.write(&response.as_bytes()).unwrap();
            res.flush().unwrap();
        }
    }
}