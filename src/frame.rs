use std::io::Cursor;

use bytes::{Buf, Bytes};

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    SimpleError(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(String),
}

impl TryFrom<&mut Cursor<&[u8]>> for Frame {
    type Error = Error;

    fn try_from(payload: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        if !payload.has_remaining() {
            return Err(Error::Incomplete);
        }

        match payload.get_u8() {
            b'+' => {
                let line = get_line(payload)?.to_vec();
                let string = String::from_utf8(line).unwrap();

                Ok(Frame::Simple(string))
            }
            b'-' => {
                let line = get_line(payload)?.to_vec();
                let string = String::from_utf8(line).unwrap();

                Ok(Frame::SimpleError(string))
            }
            b':' => {
                let len = get_uint(payload)?;

                Ok(Frame::Integer(len))
            }
            b'$' => {
                if b'-' == peek_u8(payload)? {
                    let line = get_line(payload)?;

                    if line != b"-1" {
                        return Err(Error::Other("protocol error; invalid frame format".to_string()));
                    }

                    Ok(Frame::Null)
                } else {
                    let len: usize = get_uint(payload)?.try_into().unwrap();
                    let n = len + 2;

                    if payload.remaining() < n {
                        return Err(Error::Incomplete);
                    }

                    let slice = payload.chunk();
                    let data = Bytes::copy_from_slice(&slice[..len]);
                    payload.advance(n);

                    Ok(Frame::Bulk(data))
                }
            }
            b'*' => {
                let len = get_uint(payload)?.try_into().unwrap();
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::try_from(&mut *payload)?);
                }

                Ok(Frame::Array(out))
            }
            actual => Err(Error::Other(format!("unknown frame leading byte {}", actual))),
        }
    }
}

impl From<Frame> for Vec<u8> {
    fn from(frame: Frame) -> Self {
        let mut bytes: Vec<u8> = vec![];

        match frame {
            Frame::Simple(val) => {
                bytes.push(b'+');
                bytes.extend(val.as_bytes());
            }
            Frame::SimpleError(val) => {
                bytes.push(b'-');
                bytes.extend(val.as_bytes());
            }
            Frame::Integer(val) => {
                bytes.push(b':');
                bytes.extend(val.to_string().as_bytes());
            }
            Frame::Bulk(val) => {
                let len = val.len() as u64;
                bytes.push(b'$');
                bytes.extend(len.to_string().as_bytes());
                bytes.extend(b"\r\n");
                bytes.extend(val);
            }
            Frame::Null => {
                bytes.push(b'$');
                bytes.extend(b"-1");
            }
            Frame::Array(val) => {
                let len = val.len() as u64;
                bytes.push(b'*');
                bytes.extend(len.to_ne_bytes());
                bytes.extend(b"\r\n");
                for frame in val {
                    let sub: Vec<u8> = frame.into();
                    bytes.extend(sub)
                }
            }
        }

        bytes.extend(b"\r\n");
        bytes
    }
}

fn get_line<'a>(payload: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = payload.position() as usize;
    let end = payload.get_ref().len() - 1;

    for i in start..end {
        if payload.get_ref()[i] == b'\r' && payload.get_ref()[i + 1] == b'\n' {
            payload.set_position((i + 2) as u64);

            return Ok(&payload.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
}

fn get_uint(payload: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(payload)?;

    atoi::<u64>(line).ok_or_else(|| Err(Error::Other("unable to parse integer".to_string())).unwrap())
}

fn peek_u8(payload: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !payload.has_remaining() {
        return Err(Error::Incomplete);
    }

    let position = payload.position();
    let byte = payload.get_u8();
    payload.set_position(position);
    Ok(byte)
}
