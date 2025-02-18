use crossterm::{
    cursor,
    event::{self, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    style, terminal, QueueableCommand,
};
use pyo3::prelude::*;
use std::io::{self, Write};
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

use crate::history::History;
use crate::mode::Modes;
use crate::shell::Shell;

#[pyclass]
pub enum Action {
    Writeline(String),
    Write(char),
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

        let lines: Vec<_> = output.split("\n").collect();

        // TODO: rewrite as an iterator over output, instead of splitting lines
        match lines[..] {
            [] => {}
            [line] => {
                // Not a full line, so add to the output col

                self.output_col +=
                    UnicodeWidthStr::width(strip_ansi_escapes::strip_str(line).as_str()) as u16;

                // Constrain to window bounds
                if self.output_col >= self.cols {
                    self.output_col %= self.cols;
                    stdout.queue(style::Print("\r\n"))?;
                }

                self.output_col %= self.cols;
                stdout.queue(style::Print(format!("{}\r\n", line)))?;
            }
            [.., last] => {
                // Output col is determined by only the last line
                self.output_col =
                    UnicodeWidthStr::width(strip_ansi_escapes::strip_str(last).as_str()) as u16;

                for line in lines.iter() {
                    stdout.queue(style::Print(format!("{}\r\n", line)))?;
                }
            }
        }

        if self.output_col == 0 {
            stdout.queue(cursor::MoveUp(1))?;
        }

        self.shell.write(&self.modes)?;
        self.shell.flush()?;

        Ok(())
    }
}
