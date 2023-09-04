use std::vec::IntoIter;

use bytes::Bytes;

use crate::database::Database;
use crate::frame::Frame;

use super::{Command, next_bytes, next_string};

pub(crate) struct Set {
    key: String,
    value: Bytes,
}

impl Command for Set {
    fn execute(&self, db: Database) -> Frame {
        let mut db = db.lock().unwrap();
        db.insert(self.key.clone(), self.value.clone());
        Frame::Simple("OK".to_string())
    }
}

impl Set {
    pub(crate) fn new(frames: &mut IntoIter<Frame>) -> Self {
        Set {
            key: next_string(frames).unwrap(),
            value: next_bytes(frames).unwrap(),
        }
    }
}