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

impl From<&mut IntoIter<Frame>> for Get {
    fn from(frames: &mut IntoIter<Frame>) -> Self {
        Get {
            key: next_string(frames).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use crate::database::new_db;
    use super::*;

    #[test]
    fn it_is_initialized_from_frame_iterator() {
        let key: String = "number".to_string();
        let mut iter: IntoIter<Frame> = vec![Frame::Simple(key.clone())].into_iter();

        let command: Get = (&mut iter).into();

        assert_eq!(key, command.key);
    }

    #[test]
    fn it_returns_data_from_db() {
        let db = new_db();
        {
            let db = db.clone();
            db.lock().unwrap().insert("key".to_string(), Bytes::from("value"));
        }
        let command = Get { key: "key".to_string() };

        let result = command.execute(db);

        assert_eq!(Frame::Bulk(Bytes::from("value")), result);
    }

    #[test]
    fn it_returns_null_when_key_does_not_exist() {
        let db = new_db();
        let command = Get { key: "key".to_string() };

        let result = command.execute(db);

        assert_eq!(Frame::Null, result);
    }
}