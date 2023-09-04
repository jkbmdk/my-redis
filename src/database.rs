use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use bytes::Bytes;

pub type Database = Arc<Mutex<HashMap<String, Bytes>>>;

pub fn new_db() -> Database {
    Arc::new(Mutex::new(HashMap::<String, Bytes>::new()))
}