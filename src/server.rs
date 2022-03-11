use std::ffi::OsStr;
use std::fs::File;
use std::sync::Arc;
use std::net::{TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::thread_pool::ThreadPool;
use crate::http::{Response as HTTPResponse, Request as HTTPRequest, Response, Verb as HTTPVerb};
use crate::Endpoint;

pub struct Server {
    socket: SocketAddr,
    thread_count: usize,
    endpoints: Arc<EndpointTable>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            thread_count: std::thread::available_parallelism().map(|t| t.into()).unwrap_or(4),
            endpoints: Arc::new(EndpointTable::new()),
        }
    }

    pub fn with_socket(self, socket: SocketAddr) -> Self {
        Self { socket, .. self}
    }

    pub fn with_thread_count(self, thread_count: usize) -> Self {
        Self { thread_count, .. self }
    }

    pub fn with_endpoints(self, endpoints: EndpointTable) -> Self {
        Self { endpoints: Arc::new(endpoints), .. self }
    }

    pub fn run(&mut self) {
        let mut threads = ThreadPool::new(self.thread_count);
        let listener = TcpListener::bind(self.socket).unwrap();
        println!("Listenting on {}:{}", self.socket.ip(), self.socket.port());

        loop {
            match listener.accept() {
                Ok((con, addr)) => {
                    let endpoints = self.endpoints.clone();
                    threads.submit(move || { Self::handle_request(con, addr, endpoints).unwrap(); }).unwrap();
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }

    fn handle_request(mut con: TcpStream, _: SocketAddr, endpoints: Arc<EndpointTable>) -> Result<(), ()> {
        let req = HTTPRequest::from_reader(&mut con)?;
        let mut res = BufWriter::new(con.try_clone().unwrap());

        let response = if let Some(res) = (endpoints.default)(&req) {
            res
        } else {
            (endpoints.not_found)(&req).unwrap()
        };

        res.write(&response.as_bytes()).unwrap();
        res.flush().unwrap();
        Ok(())
    }
}

pub type EndpointCallback = Box<dyn Fn(&HTTPRequest) -> Option<HTTPResponse> + Send + Sync + 'static>;

pub struct EndpointTable {
    registry: Vec<(Endpoint, EndpointCallback)>,
    default: EndpointCallback,
    not_found: EndpointCallback,
}

impl EndpointTable {
    pub fn new() -> Self {
        Self {
            registry: Vec::new(),
            default: Box::new(handle_file_request),
            not_found: Box::new(not_found_response)
        }
    }

    pub fn add_endpoint(&mut self, endpoint: Endpoint, callback: EndpointCallback) {
        self.registry.push((endpoint, callback));
    }
}

pub fn handle_file_request(req: &HTTPRequest) -> Option<HTTPResponse> {
    return match &req.endpoint() {
        (HTTPVerb::GET, res) => {
            let path = find_requested_path(res)?;
            let file = File::open(&path).ok()?;

            let mime = String::from_utf8(std::process::Command::new("file")
                .arg("--mime-type").arg("-b").arg(path.to_str().unwrap())
                .output().unwrap().stdout).unwrap().trim().to_owned();

            HTTPResponse::from_file(mime.as_str(), file).ok()
        }

        _ => {
            None
        }
    }
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
            let dir = path.parent().unwrap().read_dir().unwrap();
            let candidates: Vec<_> = dir.filter(|f| f.is_ok()).map(|f| f.unwrap())
                .map(|f| f.path())
                .filter(|f| f.file_stem().is_some())
                .filter(|f| f.file_stem().unwrap() == stem)
                .collect();
            if candidates.is_empty() { return None; }
            path = candidates[0].clone();
        }
    }

    return Some(path);
}

fn not_found_response(_: &HTTPRequest) -> Option<HTTPResponse> {
    Some(HTTPResponse::new().with_code(404))
}