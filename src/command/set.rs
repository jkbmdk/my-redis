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

impl From<&mut IntoIter<Frame>> for Set {
    fn from(frames: &mut IntoIter<Frame>) -> Self {
        Set {
            key: next_string(frames).unwrap(),
            value: next_bytes(frames).unwrap(),
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
        let mut iter: IntoIter<Frame> = vec![
            Frame::Simple("dog".to_string()),
            Frame::Simple("Jasper".to_string()),
        ].into_iter();

        let command: Set = (&mut iter).into();

        assert_eq!("dog".to_string(), command.key);
        assert_eq!("Jasper".to_string(), String::from_utf8(command.value.to_vec()).unwrap());
    }

    #[test]
    fn it_saves_value_to_the_database() {
        let db = new_db();
        let name: Bytes = [b'H', b'i', b'g'].to_vec().into();
        let command = Set { key: "name".to_string(), value: name.clone()};

        let result = command.execute(db.clone());

        assert_eq!(Frame::Simple("OK".to_string()), result);
        let binding = db.lock().unwrap();
        let value = binding.get("name").unwrap();
        assert_eq!(name, *value)
    }
}