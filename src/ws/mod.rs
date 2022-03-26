mod frame;
mod message;
mod stream;

pub use message::*;
pub use stream::*;

use crate::http::{Error as HTTPError};

#[derive(Debug)]
pub enum Error {
    ConnectionClosed,
    IOError(std::io::Error),
    HTTPError(HTTPError),
    InvalidOpCode,
}
