pub(crate) mod frame;
pub(crate) mod command;
pub(crate) mod connection;
pub(crate) mod database;
pub mod server;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;