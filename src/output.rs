use std::{error::Error, io::Write, result::Result};

use crossterm::{style::Print, QueueableCommand};

pub(crate) struct Controller {
    enabled: bool,
    prefix: String,
    desination: Box<dyn Write + Send + Sync>,
}

impl Controller {
    pub fn new(enabled: bool, prefix: String, desination: Box<dyn Write + Sync + Send>) -> Self {
        Self {
            enabled,
            prefix,
            desination,
        }
    }

    pub fn print(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        if !self.enabled {
            return Ok(());
        }

        self.desination
            .queue(Print(format!("{}{}\n", &self.prefix, s)))
            .unwrap();
        self.desination.flush()?;
        Ok(())
    }
}
