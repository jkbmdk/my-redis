use crate::database::Database;
use crate::frame::Frame;

use super::Command;

pub(crate) struct Unknown {
    pub name: String
}

impl Command for Unknown {
    fn execute(&self, _db: Database) -> Frame {
        Frame::Error(format!("Command \"{:?}\"not implemented", self.name))
    }
}