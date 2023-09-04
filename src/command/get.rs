use std::vec::IntoIter;

use crate::database::Database;
use crate::frame::Frame;

use super::{Command, next_string};

pub(crate) struct Get {
    key: String,
}

impl Command for Get {
    fn execute(&self, db: Database) -> Frame {
        let db = db.lock().unwrap();
        if let Some(value) = db.get(&*self.key) {
            Frame::Bulk(value.clone())
        } else {
            Frame::Null
        }
    }
}

impl Get {
    pub(crate) fn new(frames: &mut IntoIter<Frame>) -> Self {
        Get {
            key: next_string(frames).unwrap()
        }
    }
}