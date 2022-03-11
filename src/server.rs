use std::{fs::File, sync::Arc, path::PathBuf, path::Path};
use std::ffi::OsStr;
use std::net::{TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr};
use std::io::{BufWriter, Read, Write, BufReader, Error as IOError};
use std::string::FromUtf8Error;

use crate::thread_pool::ThreadPool;
use crate::http::{Response as HTTPResponse, Request as HTTPRequest, Response, Verb as HTTPVerb, Error as HTTPError};
use crate::Endpoint;

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

    pub fn with_entry(mut self, endpoint: Endpoint, callback: EndpointCallback) -> Self {
        self.registry.push((endpoint, callback));
        return self;
    }
}

pub struct Server {
    socket: SocketAddr,
    endpoints: Arc<EndpointTable>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            endpoints: Arc::new(EndpointTable::new()),
        }
    }

    pub fn with_socket(self, socket: SocketAddr) -> Self {
        Self { socket, .. self}
    }

    pub fn with_endpoints(self, endpoints: EndpointTable) -> Self {
        Self { endpoints: Arc::new(endpoints), .. self }
    }

    pub fn run(&mut self) {
        let mut threads = ThreadPool::new();
        let listener = TcpListener::bind(self.socket).unwrap();
        println!("Listening on {}:{}", self.socket.ip(), self.socket.port());

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

    fn handle_request(mut con: TcpStream, _: SocketAddr, endpoints: Arc<EndpointTable>) -> Result<(), HTTPError> {
        loop {
            let mut reader =  BufReader::new(con.try_clone().unwrap());
            let req = match HTTPRequest::from_stream(&mut reader) {
                Ok(x) => x,
                Err(e) => break,
            };

            let mut res = BufWriter::new(con.try_clone().unwrap());

            let response = (endpoints.default)(&req)
                .unwrap_or((endpoints.not_found)(&req).unwrap());

            res.write(&response.as_bytes()).unwrap();
            res.flush().unwrap();
        }

        Ok(())
    }
}

pub fn handle_file_request(req: &HTTPRequest) -> Option<HTTPResponse> {
    return match &req.endpoint() {
        (HTTPVerb::GET, res) => {
            let path = find_requested_path(res)?;
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
            let dir = path.parent().unwrap().read_dir().unwrap();
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

fn not_found_response(_: &HTTPRequest) -> Option<HTTPResponse> {
    Some(HTTPResponse::from_html("<html><body><h1>Not Found</h1></body></html>").with_code(404))
}