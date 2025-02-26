use std::vec::IntoIter;

use tokio::net::TcpStream;

use crate::command::Command;
use crate::connection::Connection;
use crate::database::{new_db, Database};
use crate::frame::Frame;
use crate::Error;

#[derive(Clone)]
pub struct Server {
    pub db: Database,
}

impl Server {
    pub async fn process(&self, socket: TcpStream) {
        let mut connection = Connection::new(socket);

        while let Some(frame) = connection.read_frame().await.unwrap() {
            let response: Frame = self.execute(frame).unwrap();

            connection.write_frame(response).await.unwrap();
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

        let command: Box<dyn Command> = (&mut iterator).try_into()?;

        Ok(command.execute(self.db.clone()))
    }
}

impl Default for Server {
    fn default() -> Self {
        Server { db: new_db() }
    }
}
