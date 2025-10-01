use crossterm::{cursor, event, style, terminal, QueueableCommand};
use pyo3::prelude::*;
use std::io::{self, Stdout, Write};
use std::time::Duration;

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
    last_col: u16,
    stdout: Stdout,
}

#[pymethods]
impl Console {
    #[new]
    pub fn new(name: String, titles: Vec<String>) -> PyResult<Self> {
        let (cols, _) = terminal::size()?;

        let history = match History::load_by_name(&name) {
            Ok(history) => {
                log::debug!("successfully loaded history for {}", name);
                history
            }
            Err(err) => {
                log::debug!("could not load history for {}: {}", name, err);
                match History::create(&name) {
                    Ok(history) => {
                        log::debug!("created history file for {}", name);
                        history
                    }
                    Err(_) => {
                        log::debug!("using local history");
                        History::default()
                    }
                }
            }
        };

        let mut shell = Shell::new(cols)?;

        for title in titles.iter() {
            shell.push_title(title)
        }
        let stdout = io::stdout();

        Ok(Self {
            shell,
            modes: Modes::new(history),
            last_col: 0,
            cols,
            stdout,
        })
    }

    pub fn start(&mut self) -> PyResult<()> {
        terminal::enable_raw_mode()?;
        self.shell.write(&mut self.stdout, &self.modes)?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn stop(&mut self) -> PyResult<()> {
        self.shell.clear(&mut self.stdout)?;
        self.stdout.flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn update(&mut self, timeout_ns: u64) -> PyResult<Option<Action>> {
        if event::poll(Duration::from_nanos(timeout_ns))? {
            let event = event::read()?;
            self.shell.clear(&mut self.stdout)?;

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

            self.shell.write(&mut self.stdout, &self.modes)?;
            self.stdout.flush()?;

            Ok(message)
        } else {
            Ok(None)
        }
    }

    pub fn print(&mut self, output: String) -> PyResult<()> {
        // self.shell.clear(&mut self.stdout)?;
        self.shell.move_top_left(&mut self.stdout)?;

        // If last print ended mid-line, move back to the saved column
        if self.last_col > 0 {
            self.stdout.queue(cursor::MoveUp(1))?;
            self.stdout.queue(cursor::MoveRight(self.last_col))?;
        }

        let lines = output.split('\n').collect::<Vec<_>>();
        let num_lines = lines.len();
        for (index, line) in lines.iter().enumerate() {
            self.stdout.queue(style::Print(line))?;
            self.stdout
                .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
            // \n must be replaced with \r\n to force newline in raw mode
            if index < num_lines - 1 {
                self.stdout.queue(style::Print("\n\r"))?;
            }
        }

        // Replace all \n with \r\n to force newline in raw mode
        // self.stdout
        //     .queue(style::Print(&output.replace("\n", "\r\n")))?;

        // Save column after printing all output
        let (col, _) = cursor::position()?;
        self.last_col = col;

        // If we ended mid-line, print a newline for the prompt
        if self.last_col > 0 {
            self.stdout.queue(style::Print("\n\r"))?;
        }

        // If we were at the last character, set col to 0 to wrap
        if self.last_col >= self.cols - 1 {
            self.last_col = 0;
        }

        self.shell.write(&mut self.stdout, &self.modes)?;
        self.stdout.flush()?;

        Ok(())
    }
}
