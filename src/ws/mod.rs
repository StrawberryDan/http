pub mod frame;

use std::io::{Read, Write};
use std::net::{SocketAddr};
use crate::http::{Stream as HTTPStream, Error as HTTPError, Response};
use crate::server::Service;
use crate::ws::frame::{DataFrame};

pub struct Stream<S> where S: Read + Write {
    connection: S,
}

impl<S: Read + Write> Stream<S> {
    pub fn await_handshake(mut http: HTTPStream<S>) -> Result<Self, Error> {
        loop {
            let request = http.recv().map_err(|e| Error::HTTPError(e))?;

            match request.header("Upgrade") {
                Some("websocket") => (),
                _ => continue,
            }

            match request.header("Connection") {
                Some("Upgrade") => (),
                _ => continue,
            }

            let key = match request.header("Sec-WebSocket-Key") {
                Some(k) => k,
                _ => continue,
            };

            let accept_key = openssl::base64::encode_block(
                &openssl::sha::sha1(
                    format!("{}{}", key, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").as_bytes()
                )[..]
            );

            let response = Response::new()
                .with_code(101)
                .with_header_line("Upgrade", "websocket")
                .with_header_line("Connection", "Upgrade")
                .with_header_line("Sec-WebSocket-Accept", accept_key.as_str());

            http.send(response).map_err(|e| Error::HTTPError(e))?;

            return Ok(
                Self {
                    connection: http.into_inner(),
                }
            );
        }
    }

    pub fn recv(&mut self) -> Result<DataFrame, Error> {
        DataFrame::read(&mut self.connection)
    }

    pub fn send(&mut self, frame: DataFrame) -> Result<(), Error> {
        let bytes = frame.into_bytes();
        self.connection.write_all(&bytes[..]).map_err(|e| Error::IOError(e))?;
        Ok(())
    }
}

pub struct WebSocketService {}

impl Service for WebSocketService {
    fn handle_connection(&self, con: impl Read + Write, client: SocketAddr) {
        use std::io::ErrorKind::ConnectionAborted;

        let mut stream = Stream::await_handshake(HTTPStream::new(con)).unwrap();

        loop {
            let frame = match stream.recv() {
                Ok(f) => f,
                Err(Error::IOError(e)) if matches!(e.kind(), ConnectionAborted) => break,
                Err(e) => { eprintln!("{:?}", e); break; }
            };

            stream.send(DataFrame::text("Cum!")).unwrap();
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    HTTPError(HTTPError),
    InvalidOpCode,
}