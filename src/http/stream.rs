use std::io::{Read, Write};

use crate::http::{Error, Request, Response};

pub struct Stream<S>
where
    S: Read + Write,
{
    connection: S,
}

impl<S: Read + Write> Stream<S> {
    pub fn new(connection: S) -> Self {
        Self { connection }
    }

    pub fn recv(&mut self) -> Result<Request, Error> {
        Request::read(&mut self.connection)
    }

    pub fn send(&mut self, response: Response) -> Result<(), Error> {
        let bytes = response.as_bytes();
        self.connection
            .write_all(&bytes)
            .map_err(|e| Error::IOError(e))?;
        self.connection.flush().map_err(|e| Error::IOError(e))?;
        Ok(())
    }

    pub fn into_inner(self) -> S {
        self.connection
    }
}
