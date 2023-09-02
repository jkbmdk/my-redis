use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use my_redis::connection::Connection;
use my_redis::frame::Frame;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    let db: Db = Arc::new(Mutex::new(HashMap::<String, Bytes>::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db: Db = db.clone();

        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    use my_redis::command::Command::{self, Get, Set, Unknown};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(key, value) => {
                let mut db = db.lock().unwrap();
                db.insert(key, value.clone());
                Frame::Simple("OK".to_string())
            }
            Get(key) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(&*key) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            Unknown => {
                Frame::Error("Command not implemented".to_string())
            }
        };

        connection.write_frame(&response).await.unwrap();
    }
}