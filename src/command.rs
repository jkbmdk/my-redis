use std::vec::IntoIter;

use bytes::Bytes;

use crate::frame::Frame;
use crate::Result;

#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, Bytes),
    Unknown,
}

impl Command {
    pub fn  from_frame(frame: Frame) -> Result<Command> {
        let mut iterator: IntoIter<Frame>;

        match frame {
            Frame::Array(val) => {
                iterator = Vec::into_iter(val);
            }
            _ => return Err(format!("protocol error; expected array, got {:?}", frame).into()),
        }

        let command_name = next_string(&mut iterator)?;

        let command = match &command_name[..] {
            "GET" => Command::Get(next_string(&mut iterator)?),
            "SET" => Command::Set(next_string(&mut iterator)?, next_bytes(&mut iterator)?),
            _ => Command::Unknown
        };
        Ok(command)
    }
}

pub(crate) fn next_string(iterator: &mut IntoIter<Frame>) -> Result<String> {
    match iterator.next().unwrap() {
        Frame::Simple(s) => Ok(s),
        Frame::Bulk(data) => std::str::from_utf8(&data[..])
            .map(|s| s.to_string())
            .map_err(|_| "protocol error; invalid string".into()),
        frame => Err(format!(
            "protocol error; expected simple frame or bulk frame, got {:?}",
            frame
        ).into()),
    }
}

pub(crate) fn next_bytes(iterator: &mut IntoIter<Frame>) -> Result<Bytes> {
    match iterator.next().unwrap() {
        Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
        Frame::Bulk(data) => Ok(data),
        frame => Err(format!(
            "protocol error; expected simple frame or bulk frame, got {:?}",
            frame
        ).into()),
    }
}