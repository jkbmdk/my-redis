use std::io::Cursor;
use bytes::{Buf, BytesMut};
use tokio::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::frame::Frame;
use crate::Result;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        use crate::frame::Error;

        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::try_from(&mut buf) {
            Ok(frame) => {
                self.buffer.advance(buf.position() as usize);

                return Ok(Some(frame));
            }
            Err(Error::Incomplete) => Ok(None),
            Err(Error::Other(v)) => Err(v.into()),
        }
    }

    pub async fn write_frame(&mut self, frame: Frame) -> io::Result<()> {
        let bytes: Vec<u8> = frame.into();
        let slice = bytes.as_slice();
        self.stream.write_all(slice).await?;
        self.stream.flush().await
    }
}
