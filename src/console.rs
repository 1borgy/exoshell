use crossterm::{cursor, event, style, terminal, QueueableCommand};
use pyo3::prelude::*;
use std::io;
use std::time::Duration;
use unicode_width::UnicodeWidthChar;

use crate::history::History;
use crate::mode::Modes;
use crate::shell::Shell;

#[pyclass]
pub enum Action {
    Writeline(String),
    Write(String),
    Quit(),
}

#[pyclass]
pub struct Console {
    shell: Shell,
    modes: Modes,
    cols: u16,
    output_col: u16,
}

#[pymethods]
impl Console {
    #[new]
    pub fn new(name: String, titles: Vec<String>) -> PyResult<Self> {
        let (cols, _) = terminal::size()?;

        let mut history =
            History::load_by_name(&name).unwrap_or(History::new(&name).unwrap_or_default());

        if let Err(err) = history.sort() {
            log::warn!("could not sort history: {:?}", err)
        }

        let mut shell = Shell::new(cols)?;

        for title in titles.iter() {
            shell.push_title(title)
        }

        Ok(Self {
            shell,
            modes: Modes::new(history),
            output_col: 0,
            cols,
        })
    }

    pub fn start(&mut self) -> PyResult<()> {
        terminal::enable_raw_mode()?;
        self.shell.write(&self.modes)?;
        self.shell.flush()?;
        Ok(())
    }

    pub fn stop(&mut self) -> PyResult<()> {
        self.shell.clear()?;
        self.shell.flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn update(&mut self, timeout_ns: u64) -> PyResult<Option<Action>> {
        if event::poll(Duration::from_nanos(timeout_ns))? {
            let event = event::read()?;
            self.shell.clear()?;

            let message = match event {
                event::Event::Key(key) => {
                    let action = self.modes.on_key(key);
                    action
                }
                event::Event::Resize(cols, _) => {
                    self.cols = cols;
                    self.shell.resize(cols)?;
                    None
                }
                _ => None,
            };

            self.shell.write(&self.modes)?;
            self.shell.flush()?;

            Ok(message)
        } else {
            Ok(None)
        }
    }

    pub fn print(&mut self, output: String) -> PyResult<()> {
        let mut stdout = io::stdout();

        self.shell.clear()?;

        if self.output_col > 0 {
            stdout.queue(cursor::MoveUp(1))?;
            stdout.queue(cursor::MoveRight(self.output_col))?;
        }

        stdout.queue(style::Print(&output))?;

        for c in output.chars() {
            match c {
                '\r' | '\n' => {
                    self.output_col = 0;
                }
                '\u{7f}' => {
                    self.output_col = self.output_col.saturating_sub(1);
                    stdout.queue(cursor::MoveLeft(2))?;
                    stdout.queue(style::Print(" "))?;
                    stdout.queue(cursor::MoveLeft(1))?;
                }
                _ => {
                    if let Some(width) = UnicodeWidthChar::width(c) {
                        self.output_col = (self.output_col + width as u16) % self.cols;
                    }
                }
            }
        }

        if self.output_col > 0 {
            stdout.queue(style::Print("\r\n"))?;
        }

        self.shell.write(&self.modes)?;
        self.shell.flush()?;

        Ok(())
    }
}
