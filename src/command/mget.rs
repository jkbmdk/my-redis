use std::vec::IntoIter;

use crate::command::next_string;
use crate::database::Database;
use crate::frame::Frame;

use super::Command;

pub(crate) struct MGet {
    keys: Vec<String>,
}

impl Command for MGet {
    fn execute(&self, db: Database) -> Frame {
        let db = db.lock().unwrap();
        let mut result: Vec<Frame> = vec![];

        for key in self.keys.clone() {
            let frame: Frame;

            if let Some(value) = db.get(&*key) {
                frame = Frame::Bulk(value.clone());
            } else {
                frame = Frame::Null;
            }

            result.push(frame);
        }

        Frame::Array(result)
    }
}

impl From<&mut IntoIter<Frame>> for MGet {
    fn from(frames: &mut IntoIter<Frame>) -> Self {
        let mut keys: Vec<String> = vec![];

        loop {
            match next_string(frames) {
                Ok(key) => keys.push(key),
                Err(..) => break,
            };
        };

        MGet { keys }
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
            Frame::Simple("where".to_string()),
            Frame::Bulk(vec![b'w', b'h', b'e', b'n'].into()),
            Frame::Simple("what".to_string()),
        ].into_iter();

        let command: MGet = (&mut iter).into();

        assert_eq!("where".to_string(), command.keys[0]);
        assert_eq!("when".to_string(), command.keys[1]);
        assert_eq!("what".to_string(), command.keys[2]);
    }

    #[test]
    fn it_returns_data_from_db() {
        let db = new_db();
        {
            let db = db.clone();
            db.lock().unwrap().insert("when".to_string(), Bytes::from("now"));
            db.lock().unwrap().insert("where".to_string(), Bytes::from("here"));
            db.lock().unwrap().insert("what".to_string(), Bytes::from("code!"));
        }
        let command = MGet {
            keys: vec![
                "when".to_string(),
                "where".to_string(),
                "what".to_string(),
            ]
        };

        let result = command.execute(db);

        assert_eq!(Frame::Array(
            vec![
                Frame::Bulk(Bytes::from("now")),
                Frame::Bulk(Bytes::from("here")),
                Frame::Bulk(Bytes::from("code!")),
            ]
        ), result);
    }

    #[test]
    fn it_returns_null_when_key_does_not_exist() {
        let db = new_db();
        {
            let db = db.clone();
            db.lock().unwrap().insert("first_name".to_string(), Bytes::from("Gunter"));
            db.lock().unwrap().insert("age".to_string(), Bytes::from("2763"));
        }
        let command = MGet {
            keys: vec![
                "first_name".to_string(),
                "last_name".to_string(),
                "age".to_string(),
            ]
        };

        let result = command.execute(db);

        assert_eq!(Frame::Array(
            vec![
                Frame::Bulk(Bytes::from("Gunter")),
                Frame::Null,
                Frame::Bulk(Bytes::from("2763")),
            ]
        ), result);
    }
}