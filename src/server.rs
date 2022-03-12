use std::{fs::File, path::Path, path::PathBuf, sync::Arc};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::io::{BufReader, BufWriter, Write};
use std::sync::Mutex;


use crate::thread_pool::ThreadPool;
use crate::http::{Request as HTTPRequest, Response as HTTPResponse, Verb as HTTPVerb};
use crate::endpoint::{Endpoint, Tree as EndpointTree, URLBindings};
use crate::Error;

pub type Callback = fn(&HTTPRequest, &URLBindings) -> Option<HTTPResponse>;
pub type ConnectionHandler = fn(TcpStream, SocketAddr, Arc<Mutex<EndpointTree>>);

pub struct Server {
    socket: SocketAddr,
    endpoints: Arc<Mutex<EndpointTree>>,
    handler: ConnectionHandler,
}

impl Server {
    pub fn new() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            endpoints: Arc::new(Mutex::new(EndpointTree::new())),
            handler: Server::default_request_handler,
        }
    }

    pub fn with_socket(self, socket: SocketAddr) -> Self {
        Self { socket, .. self}
    }

    pub fn with_endpoint(self, endpoint: Endpoint, callback: Callback) -> Result<Self, Error> {
        self.endpoints.lock().unwrap().add(endpoint, callback)?;
        Ok(self)
    }

    pub fn with_handler(self, handler: ConnectionHandler) -> Self {
        Self { handler, .. self }
    }

    pub fn run(&mut self) {
        let mut threads = ThreadPool::new();
        let listener = TcpListener::bind(self.socket).unwrap();
        println!("Listening on {}:{}", self.socket.ip(), self.socket.port());

        loop {
            match listener.accept() {
                Ok((con, addr)) => {
                    let endpoints = self.endpoints.clone();
                    let handler = self.handler.clone();
                    threads.submit(move || (handler)(con, addr, endpoints) ).unwrap();
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }

    fn default_request_handler(con: TcpStream, _: SocketAddr, endpoints: Arc<Mutex<EndpointTree>>) {
        loop {
            let mut reader =  BufReader::new(con.try_clone().unwrap());
            let req = match HTTPRequest::from_stream(&mut reader) {
                Ok(x) => x,
                Err(_) => break,
            };

            let mut res = BufWriter::new(con.try_clone().unwrap());

            let endpoints = endpoints.lock().unwrap();
            let callback = endpoints.find_match(req.resource()).map(|(c, b)| c(&req, &b)).flatten();

            let response = match callback {
                Some(res) => res,
                None => handle_file_request(&req, &HashMap::new()).or(not_found_response(&req, &HashMap::new())).unwrap()
            };

            res.write(&response.as_bytes()).unwrap();
            res.flush().unwrap();
        }
    }
}

pub fn handle_file_request(req: &HTTPRequest, _: &URLBindings) -> Option<HTTPResponse> {
    return match &req.verb() {
        HTTPVerb::GET => {
            let path = find_requested_path(req.resource())?;
            let file = File::open(&path).ok()?;
            let mime = mime_type(path).unwrap_or(String::from("application/octet-stream")).trim().to_owned();
            HTTPResponse::from_file(mime.as_str(), file).ok()
        }

        _ => {
            None
        }
    }
}

pub fn mime_type<R: AsRef<Path>>(path: R) -> Result<String, ()> {
    let path = path.as_ref();
    String::from_utf8(
        std::process::Command::new("file")
            .arg("--mime-type").arg("-b").arg(path.to_str().unwrap())
            .output().map_err(|_| ())?.stdout
    ).map_err(|_| ())
}

fn find_requested_path(res: &String) -> Option<PathBuf> {
    let mut path = if res == "/" {
        PathBuf::from("./index")
    } else {
        PathBuf::from(format!(".{}", res))
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

fn not_found_response(_: &HTTPRequest, _: &URLBindings) -> Option<HTTPResponse> {
    Some(HTTPResponse::from_html("<html><body><h1>Not Found</h1></body></html>").with_code(404))
}