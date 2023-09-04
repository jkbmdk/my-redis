use std::vec::IntoIter;

use tokio::net::TcpStream;

use crate::command::{Command, next_string};
use crate::command::get::Get;
use crate::command::set::Set;
use crate::command::unknown::Unknown;
use crate::connection::Connection;
use crate::database::{Database, new_db};
use crate::Error;
use crate::frame::Frame;

#[derive(Clone)]
pub struct Server {
    pub db: Database,
}

impl Server {
    pub fn new() -> Self {
        Server { db: new_db() }
    }

    pub async fn process(&self, socket: TcpStream) {
        let mut connection = Connection::new(socket);

        while let Some(frame) = connection.read_frame().await.unwrap() {
            let response: Frame = self.execute(frame).unwrap();

            connection.write_frame(&response).await.unwrap();
        }
    }

    fn execute(&self, frame: Frame) -> Result<Frame, Error> {
        let mut iterator: IntoIter<Frame>;

        match frame {
            Frame::Array(val) => {
                iterator = Vec::into_iter(val);
            }
            _ => return Err(format!("protocol error; expected array, got {:?}", frame).into()),
        }

        let command_name = next_string(&mut iterator)?;

        let command: Box<dyn Command> = match &command_name[..] {
            "GET" => Box::new(Get::new(&mut iterator)),
            "SET" => Box::new(Set::new(&mut iterator)),
            v => Box::new(Unknown { name: v.to_string() }),
        };
        Ok(command.execute(self.db.clone()))
    }
}