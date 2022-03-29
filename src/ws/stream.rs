use std::io::{Read, Write};
use crate::http::{Response, Stream as HTTPStream};
use crate::ws::{Error, Message};
use crate::ws::frame::DataFrame;

pub struct Stream<S>
where
    S: Read + Write,
{
    connection: S,
}

impl<S: Read + Write> Stream<S> {
    pub fn await_handshake(mut http: HTTPStream<S>) -> Result<Self, Error> {
        loop {
            let request = http.recv().map_err(|e| Error::HTTPError(e))?;

            match request.header().get_first("Upgrade") {
                Some("websocket") => (),
                _ => continue,
            }

            match request.header().get_first("Connection") {
                Some("Upgrade") => (),
                _ => continue,
            }

            let key = match request.header().get_first("Sec-WebSocket-Key") {
                Some(k) => k,
                _ => continue,
            };

            let accept_key = openssl::base64::encode_block(
                &openssl::sha::sha1(
                    format!("{}{}", key, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").as_bytes(),
                )[..],
            );

            let response = Response::new(101)
                .with_header("Upgrade", "websocket")
                .with_header("Connection", "Upgrade")
                .with_header("Sec-WebSocket-Accept", accept_key.as_str());

            http.send(response).map_err(|e| Error::HTTPError(e))?;

            return Ok(Self {
                connection: http.into_inner(),
            });
        }
    }

    pub fn recv(&mut self) -> Result<Message, Error> {
        DataFrame::read_from(&mut self.connection).map(|f| f.into())
    }

    pub fn send(&mut self, message: Message) -> Result<(), Error> {
        let bytes = DataFrame::from(message).into_bytes();
        self.connection
            .write_all(&bytes[..])
            .map_err(|e| Error::IOError(e))?;
        Ok(())
    }
}
