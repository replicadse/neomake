use std::{error::Error, io::Write, result::Result};

use crossterm::{style::Print, QueueableCommand};

pub(crate) struct Controller {
    pending_lines: Vec<String>,
    prefix: String,
}

impl Controller {
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            pending_lines: vec![],
        }
    }

    pub fn append(&mut self, s: String) {
        self.pending_lines.push(s);
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        let mut stdout = std::io::stdout();
        for l in self.pending_lines.drain(..) {
            stdout.queue(Print(format!("{}{}\n", &self.prefix, l))).unwrap();
        }
        stdout.flush()?;
        Ok(())
    }
}
