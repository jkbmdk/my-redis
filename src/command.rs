use std::vec::IntoIter;

use bytes::Bytes;

use crate::database::Database;
use crate::frame::Frame;
use crate::Result;

pub(crate) mod get;
pub(crate) mod set;
pub(crate) mod unknown;

pub trait Command {
    fn execute(&self, db: Database) -> Frame;
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