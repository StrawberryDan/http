use std::io::{BufReader, BufWriter, Write};
use std::net::TcpStream;
use crate::http::{Error, Request, Response};

pub struct Stream {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Stream {
    pub fn new(connection: TcpStream) -> Self {
        Self {
            reader: BufReader::new(connection.try_clone().unwrap()),
            writer: BufWriter::new(connection),
        }
    }

    pub fn recv(&mut self) -> Result<Request, Error> {
        Request::from_stream(&mut self.reader)
    }

    pub fn send(&mut self, response: Response) -> Result<(), Error> {
        self.writer.write_all(&response.as_bytes()).map_err(|e| Error::IOError(e))?;
        self.writer.flush().map_err(|e| Error::IOError(e))?;
        Ok(())
    }

}