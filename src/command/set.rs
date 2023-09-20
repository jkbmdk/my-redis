use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::vec::IntoIter;

use bytes::Bytes;

use crate::database::Database;
use crate::frame::Frame;

use super::{Command, next_bytes, next_integer, next_string};

pub(crate) struct Set {
    key: String,
    value: Bytes,
    ttl: Option<Duration>,
    replacement: Replacement,
    get: bool,
}

impl Command for Set {
    fn execute(&self, db: Database) -> Frame {
        let mut db = db.lock().unwrap();
        let previous = db.get(&*self.key);
        let result: Frame = match self.get {
            true => {
                match previous {
                    None => { Frame::Null }
                    Some(value) => { Frame::Bulk(value.clone()) }
                }
            }
            false => { Frame::Simple("OK".to_string()) }
        };

        match self.replacement {
            Replacement::Always => {
                db.insert(self.key.clone(), self.value.clone());
            }
            Replacement::Never => {
                if None == previous {
                    db.insert(self.key.clone(), self.value.clone());
                }
            }
            Replacement::OnlyOverride => {
                if None != previous {
                    db.insert(self.key.clone(), self.value.clone());
                }
            }
        }

        result
    }
}

impl From<&mut IntoIter<Frame>> for Set {
    fn from(frames: &mut IntoIter<Frame>) -> Self {
        let key = next_string(frames).unwrap();
        let value = next_bytes(frames).unwrap();
        let mut ttl: Option<Duration> = None;
        let mut replacement = Replacement::default();
        let mut get: bool = false;

        loop {
            match next_string(frames) {
                Ok(key) => match key.as_str() {
                    "EX" => {
                        let seconds = next_integer(frames).unwrap();
                        ttl = Some(Duration::from_secs(seconds));
                    }
                    "PX" => {
                        let milis = next_integer(frames).unwrap();
                        ttl = Some(Duration::from_millis(milis));
                    }
                    "EXAT" => {
                        let timestamp_seconds = next_integer(frames).unwrap();
                        let duration = Duration::from_secs(timestamp_seconds);
                        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        ttl = duration.checked_sub(timestamp);
                    }
                    "PXAT" => {
                        let timestamp_milis = next_integer(frames).unwrap();
                        let duration = Duration::from_millis(timestamp_milis);
                        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        ttl = duration.checked_sub(timestamp);
                    }
                    "NX" => {
                        replacement = Replacement::Never;
                    }
                    "XX" => {
                        replacement = Replacement::OnlyOverride;
                    }
                    "GET" => {
                        get = true;
                    }
                    _ => {}
                },
                Err(..) => break,
            };
        };

        Set {
            key,
            value,
            ttl,
            replacement,
            get,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Replacement {
    Always,
    Never,
    OnlyOverride,
}

impl Default for Replacement {
    fn default() -> Self {
        Replacement::Always
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::database::new_db;

    use super::*;

    #[test]
    fn it_is_initialized_from_simple_frame_iterator() {
        let mut iter: IntoIter<Frame> = vec![
            Frame::Simple("dog".to_string()),
            Frame::Simple("Jasper".to_string()),
        ].into_iter();

        let command: Set = (&mut iter).into();

        assert_eq!("dog".to_string(), command.key);
        assert_eq!("Jasper".to_string(), String::from_utf8(command.value.to_vec()).unwrap());
        assert_eq!(None, command.ttl);
        assert_eq!(Replacement::Always, command.replacement);
        assert_eq!(false, command.get);
    }

    #[test]
    fn it_is_initialized_from_complex_frame_iterator() {
        let mut iter: IntoIter<Frame> = vec![
            Frame::Simple("airplane".to_string()),
            Frame::Simple("The Beast".to_string()),
            Frame::Simple("EX".to_string()),
            Frame::Integer(180),
            Frame::Simple("XX".to_string()),
            Frame::Simple("GET".to_string()),
        ].into_iter();

        let command: Set = (&mut iter).into();

        assert_eq!("airplane".to_string(), command.key);
        assert_eq!("The Beast".to_string(), String::from_utf8(command.value.to_vec()).unwrap());
        assert_eq!(Duration::from_secs(180), command.ttl.unwrap());
        assert_eq!(Replacement::OnlyOverride, command.replacement);
        assert_eq!(true, command.get);
    }

    #[test]
    fn it_saves_value_to_the_database() {
        let db = new_db();
        let name: Bytes = [b'H', b'i', b'g'].to_vec().into();
        let command = Set {
            key: "name".to_string(),
            value: name.clone(),
            ttl: None,
            replacement: Default::default(),
            get: false
        };

        let result = command.execute(db.clone());

        assert_eq!(Frame::Simple("OK".to_string()), result);
        let binding = db.lock().unwrap();
        let value = binding.get("name").unwrap();
        assert_eq!(name, *value)
    }

    #[test]
    fn it_does_not_overwrite_value_with_never_replacement_policy() {
        let db = new_db();
        let old_name = Bytes::from("Hig");
        {
            let db = db.clone();
            db.lock().unwrap().insert("name".to_string(), old_name.clone());
        }
        let new_name: Bytes = [b'B', b'a', b'n', b'g', b'l', b'e', b'y'].to_vec().into();
        let command = Set {
            key: "name".to_string(),
            value: new_name,
            ttl: None,
            replacement: Replacement::Never,
            get: false
        };

        let result = command.execute(db.clone());

        assert_eq!(Frame::Simple("OK".to_string()), result);
        let binding = db.lock().unwrap();
        let value = binding.get("name").unwrap();
        assert_eq!(old_name, *value)
    }
}