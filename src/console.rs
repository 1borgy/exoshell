use crossterm::{cursor, event, style, terminal, QueueableCommand};
use pyo3::prelude::*;
use std::io;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

use crate::mode::Mode;
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
    mode: Mode,
    cols: u16,
    output_col: u16,
}

#[pymethods]
impl Console {
    #[new]
    pub fn new() -> PyResult<Self> {
        let (cols, _) = terminal::size()?;
        Ok(Self {
            shell: Shell::new(cols)?,
            mode: Mode::default(),
            output_col: 0,
            cols,
        })
    }

    pub fn push_title(&mut self, title: String) -> PyResult<()> {
        self.shell.push_title(title);
        Ok(())
    }

    pub fn start(&mut self) -> PyResult<()> {
        terminal::enable_raw_mode()?;
        self.shell.write(&self.mode)?;
        self.shell.flush()?;
        Ok(())
    }

    pub fn stop(&mut self) -> PyResult<()> {
        terminal::disable_raw_mode()?;
        self.shell.clear()?;
        self.shell.flush()?;
        Ok(())
    }

    pub fn update(&mut self, timeout: u64) -> PyResult<Option<Action>> {
        if event::poll(Duration::from_millis(timeout))? {
            let event = event::read()?;
            self.shell.clear()?;

            let message = match event {
                event::Event::Key(key) => {
                    let (new_mode, action) = self.mode.clone().on_key(key);
                    self.mode = new_mode;

                    action
                }
                event::Event::Resize(cols, _) => {
                    self.cols = cols;
                    self.shell.resize(cols)?;
                    None
                }
                _ => None,
            };

            self.shell.write(&self.mode)?;
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

        self.shell.write(&self.mode)?;
        self.shell.flush()?;

        Ok(())
    }
}
