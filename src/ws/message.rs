use crate::ws::frame::{DataFrame, OpCode};

pub enum Message {
    String(String),
    Binary(Vec<u8>),
    Close,
    Ping,
    Pong,
}

impl From<DataFrame> for Message {
    fn from(frame: DataFrame) -> Self {
        match frame.op {
            OpCode::CONTINUATION => unreachable!(),
            OpCode::TEXT => Self::String(String::from_utf8(frame.payload).unwrap()),
            OpCode::BINARY => Self::Binary(frame.payload),
            OpCode::CLOSE => Message::Close,
            OpCode::PING => Message::Ping,
            OpCode::PONG => Message::Pong,
        }
    }
}
