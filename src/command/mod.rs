use std::vec::IntoIter;

use bytes::Bytes;
use crate::command::get::Get;
use crate::command::mget::MGet;
use crate::command::set::Set;
use crate::command::unknown::Unknown;

use crate::database::Database;
use crate::frame::Frame;
use crate::{Error, Result};

pub(crate) mod get;
pub(crate) mod set;
pub(crate) mod unknown;
pub(crate) mod mget;

pub trait Command {
    fn execute(&self, db: Database) -> Frame;
}

impl TryFrom<&mut IntoIter<Frame>> for Box<dyn Command> {
    type Error = Error;

    fn try_from(frames: &mut IntoIter<Frame>) -> Result<Self> {
        let command_name = match next_string(frames) {
            Ok(name) => name,
            Err(_) => return Err("Lack of command name".into())
        };

        let command: Box<dyn Command> = match &command_name[..] {
            "GET" => Box::new(Get::from(frames)),
            "MGET" => Box::new(MGet::from(frames)),
            "SET" => Box::new(Set::from(frames)),
            v => Box::new(Unknown { name: v.to_string() }),
        };

        Ok(command)
    }
}

pub(crate) fn next_string(iterator: &mut IntoIter<Frame>) -> Result<String> {
    match iterator.next() {
        Some(frame) => {
            match frame {
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
        None => {
            Err("end".into())
        }
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

pub(crate) fn next_integer(iterator: &mut IntoIter<Frame>) -> Result<u64> {
    match iterator.next().unwrap() {
        Frame::Integer(u) => Ok(u),
        frame => Err(format!(
            "protocol error; expected integer frame, got {:?}",
            frame
        ).into()),
    }
}