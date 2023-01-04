use std::{
    error::Error,
    io::Write,
    result::Result,
};

use crossterm::{
    style::Print,
    terminal,
    QueueableCommand,
};

pub(crate) struct Controller {
    max_lines: usize,
    lines: Vec<String>,
    drawn_lines: usize,
}
impl Controller {
    pub fn new(max: usize) -> Self {
        Self {
            max_lines: max,
            drawn_lines: 0,
            lines: Vec::<String>::new(),
        }
    }

    pub fn append(&mut self, s: String) {
        self.lines.push(s);
        if self.lines.len() > self.max_lines {
            self.lines.remove(0);
        }
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        let mut stdout = std::io::stdout();
        if self.drawn_lines > 0 {
            stdout
                .queue(crossterm::cursor::MoveToColumn(0u16))
                .unwrap()
                .queue(crossterm::cursor::MoveUp(self.drawn_lines as u16))
                .unwrap();
        }
        stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

        for l in &self.lines {
            stdout.queue(Print(format!("{}\n", l))).unwrap();
        }
        stdout.flush()?;
        self.drawn_lines = self.lines.len();
        Ok(())
    }
}
