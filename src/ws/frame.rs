use super::Error;
use crate::ws::frame::OpCode::BINARY;
use std::io::Read;

#[derive(Debug)]
pub struct DataFrame {
    fin: bool,
    opcode: OpCode,
    payload: Vec<u8>,
}

impl DataFrame {
    pub fn text(text: &str) -> Self {
        Self {
            fin: true,
            opcode: OpCode::TEXT,
            payload: text.as_bytes().to_vec(),
        }
    }

    pub fn binary(binary: Vec<u8>) -> Self {
        Self {
            fin: true,
            opcode: BINARY,
            payload: binary,
        }
    }

    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let mut frames: Vec<DataFrame> = Vec::new();

        loop {
            frames.push(Self::read_single(reader)?);
            if frames.last().unwrap().fin {
                break;
            }
        }

        match frames.len() {
            1 => Ok(frames.remove(0)),
            _ => Ok(frames
                .into_iter()
                .reduce(|mut a, mut b| {
                    a.payload.append(&mut b.payload);
                    return DataFrame {
                        fin: b.fin,
                        opcode: a.opcode,
                        payload: a.payload,
                    };
                })
                .unwrap()),
        }
    }

    pub fn read_single(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buffer = [0u8; 2];
        reader
            .read_exact(&mut buffer[..])
            .map_err(|e| Error::IOError(e))?;

        let fin = (buffer[0] & 0b1000_0000) != 0;
        let opcode = OpCode::from_bits(buffer[0] & 0b0000_1111)?;
        let mask_flag = (buffer[1] & 0b1000_0000) != 0;
        let payload_len = match buffer[1] & 0b0111_1111 {
            126 => {
                let mut len_buffer = [0u8; 8];
                reader
                    .read_exact(&mut len_buffer[6..])
                    .map_err(|e| Error::IOError(e))?;
                u64::from_be_bytes(len_buffer)
            }

            127 => {
                let mut len_buffer = [0u8; 8];
                reader
                    .read_exact(&mut len_buffer[..])
                    .map_err(|e| Error::IOError(e))?;
                u64::from_be_bytes(len_buffer)
            }

            _ => (buffer[1] & 0b0111_1111) as u64,
        };

        let mask = match mask_flag {
            true => {
                let mut mask_key_buffer = [0u8; 4];
                reader
                    .read_exact(&mut mask_key_buffer[..])
                    .map_err(|e| Error::IOError(e))?;
                Some(u32::from_be_bytes(mask_key_buffer))
            }

            false => None,
        };

        let mut payload = vec![0; payload_len as usize];
        reader
            .read_exact(&mut payload[..])
            .map_err(|e| Error::IOError(e))?;

        if let Some(mask) = mask {
            payload = payload
                .into_iter()
                .enumerate()
                .map(|b| unmask_byte(b.0, b.1, mask))
                .collect();
        }

        Ok(DataFrame {
            fin,
            opcode,
            payload,
        })
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(0b1000_0000 | self.opcode.to_byte());

        match self.payload.len() {
            x if x <= 125 => bytes.push(x as u8),

            x if x <= u16::MAX as usize => {
                bytes.push(126);
                let val = (x as u16).to_be_bytes();
                for v in val {
                    bytes.push(v);
                }
            }

            x => {
                bytes.push(127);
                let val = (x as u64).to_be_bytes();
                for v in val {
                    bytes.push(v);
                }
            }
        }

        bytes.append(&mut self.payload);

        return bytes;
    }
}

fn unmask_byte(index: usize, byte: u8, mask: u32) -> u8 {
    let mask = mask.to_be_bytes();
    return byte ^ (mask[index % 4]);
}

#[derive(Debug)]
pub enum OpCode {
    CONTINUATION,
    TEXT,
    BINARY,
    CLOSE,
    PING,
    PONG,
}

impl OpCode {
    fn to_byte(&self) -> u8 {
        match self {
            OpCode::CONTINUATION => 0x0,
            OpCode::TEXT => 0x1,
            OpCode::BINARY => 0x2,
            OpCode::CLOSE => 0x8,
            OpCode::PING => 0x9,
            OpCode::PONG => 0xA,
        }
    }

    fn from_bits(byte: u8) -> Result<Self, Error> {
        match byte {
            0x0 => Ok(OpCode::CONTINUATION),
            0x1 => Ok(OpCode::TEXT),
            0x2 => Ok(OpCode::BINARY),
            0x8 => Ok(OpCode::CLOSE),
            0x9 => Ok(OpCode::PING),
            0xA => Ok(OpCode::PONG),
            _ => Err(Error::InvalidOpCode),
        }
    }
}
