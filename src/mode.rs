use crate::console::Action;
use crate::history::History;
use crate::shell;
use crossterm::{
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style,
};

#[derive(Debug)]
pub enum Mode {
    Line,
    Raw,
    Prefix,
}

#[derive(Debug)]
pub enum Message {
    ChangeMode(Mode),
    Writeline(String),
    Write(String),
    Quit(),
}

pub trait OnKey {
    fn on_key(&mut self, key: KeyEvent) -> Option<Message>;
}

pub struct Line {
    contents: String,
    cursor: usize,
    history: History,
    history_index: usize,
}

impl Line {
    fn new(history: History) -> Self {
        Self {
            history,
            history_index: 0,
            contents: "".to_string(),
            cursor: 0,
        }
    }

    fn select_history(&mut self, add: usize, sub: usize) {
        let entries = self.history.entries();

        self.history_index = self
            .history_index
            .saturating_add(add)
            .saturating_sub(sub)
            .min(entries.len());

        if self.history_index > 0 {
            if let Some(entry) = entries.get(entries.len() - self.history_index) {
                self.contents = entry.cmd.to_string();
                self.cursor = self.contents.chars().count();
            }
        } else {
            self.contents = "".to_string();
            self.cursor = 0;
        }
    }
}

impl shell::State for Line {
    fn color(&self) -> style::Color {
        style::Color::Green
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn contents(&self) -> &str {
        &self.contents
    }

    fn name(&self) -> &str {
        "LINE"
    }

    fn keybinds(&self) -> Vec<&str> {
        vec!["^D Quit", "^\\ Prefix"]
    }
}

impl OnKey for Line {
    fn on_key(&mut self, key: KeyEvent) -> Option<Message> {
        match key {
            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Home,
                ..
            }
            | KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('a'),
                ..
            } => {
                self.cursor = 0;
                None
            }

            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                code: KeyCode::End,
                ..
            }
            | KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('e'),
                ..
            } => {
                self.cursor = self.contents.chars().count();
                None
            }

            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(c),
                ..
            }
            | KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::SHIFT,
                code: KeyCode::Char(c),
                ..
            } => {
                let mut chars = self.contents.chars().into_iter();
                self.contents = format!(
                    "{}{}{}",
                    chars.by_ref().take(self.cursor).collect::<String>(),
                    c,
                    chars.by_ref().collect::<String>()
                );
                self.cursor += 1;
                self.history_index = 0;

                None
            }

            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                code,
                ..
            } => match code {
                KeyCode::Enter => {
                    let cmd = self.contents.to_string();

                    // Only update history if cmd isn't empty
                    if cmd.len() > 0 {
                        if let Err(err) = self.history.update(&cmd) {
                            log::warn!("could not update history: {:?}", err)
                        }
                    }

                    self.contents = "".to_string();
                    self.cursor = 0;
                    self.history_index = 0;

                    Some(Message::Writeline(cmd))
                }

                KeyCode::Left => {
                    self.cursor = self.cursor.saturating_sub(1);
                    None
                }
                KeyCode::Right => {
                    // For right, we need to clamp by the length of the current contents
                    self.cursor = (self.cursor + 1).min(self.contents.chars().count());
                    None
                }
                KeyCode::Up => {
                    self.select_history(1, 0);
                    None
                }
                KeyCode::Down => {
                    self.select_history(0, 1);
                    None
                }

                KeyCode::Backspace => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        let mut chars = self.contents.chars();

                        let left = chars.by_ref().take(self.cursor).collect::<String>();
                        chars.by_ref().next();
                        let right = chars.by_ref().collect::<String>();

                        self.contents = format!("{}{}", left, right);
                    }
                    None
                }
                KeyCode::Delete => {
                    if self.cursor < self.contents.chars().count() {
                        let mut chars = self.contents.chars();

                        let left = chars.by_ref().take(self.cursor).collect::<String>();
                        chars.by_ref().next();
                        let right = chars.by_ref().collect::<String>();

                        self.contents = format!("{}{}", left, right);
                    }
                    None
                }

                // TODO: history with up/down arrows
                _ => None,
            },

            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                code,
                ..
            } => match code {
                KeyCode::Char('d') => Some(Message::Quit()),
                KeyCode::Char('c') => {
                    self.contents = "".to_string();
                    self.cursor = 0;
                    None
                }
                KeyCode::Char('4') | KeyCode::Char('\\') => Some(Message::ChangeMode(Mode::Prefix)),

                // TODO: C-Backspace / C-Left / C-Right
                _ => None,
            },

            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Raw {}

impl shell::State for Raw {
    fn color(&self) -> style::Color {
        style::Color::Red
    }

    fn cursor(&self) -> usize {
        0
    }

    fn contents(&self) -> &str {
        ""
    }

    fn name(&self) -> &str {
        "RAW"
    }

    fn keybinds(&self) -> Vec<&str> {
        vec!["^\\ Prefix"]
    }
}

impl OnKey for Raw {
    fn on_key(&mut self, key: KeyEvent) -> Option<Message> {
        match key {
            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(c),
                ..
            }
            | KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::SHIFT,
                code: KeyCode::Char(c),
                ..
            } => Some(Message::Write(c.into())),

            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                code,
                ..
            } => match code {
                KeyCode::Enter => Some(Message::Write("\n".into())),
                KeyCode::Backspace => Some(Message::Write("\u{7f}".into())),
                KeyCode::Esc => Some(Message::Write("\u{1b}".into())),
                KeyCode::Tab => Some(Message::Write("\u{09}".into())),
                KeyCode::Up => Some(Message::Write("\u{1b}[A".into())),
                KeyCode::Down => Some(Message::Write("\u{1b}[B".into())),

                _ => None,
            },

            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                code,
                ..
            } => match code {
                KeyCode::Char('4') | KeyCode::Char('\\') => Some(Message::ChangeMode(Mode::Prefix)),

                KeyCode::Char('a') => Some(Message::Write("\u{1}".into())), // SOH
                KeyCode::Char('b') => Some(Message::Write("\u{2}".into())), // STX
                KeyCode::Char('c') => Some(Message::Write("\u{3}".into())), // ETX
                KeyCode::Char('d') => Some(Message::Write("\u{4}".into())), // EOT
                KeyCode::Char('e') => Some(Message::Write("\u{5}".into())), // ENQ
                KeyCode::Char('f') => Some(Message::Write("\u{6}".into())), // EOT
                KeyCode::Char('g') => Some(Message::Write("\u{7}".into())), // EOT
                KeyCode::Char('h') => Some(Message::Write("\u{8}".into())), // BS
                KeyCode::Char('i') => Some(Message::Write("\u{9}".into())), // HT
                KeyCode::Char('j') => Some(Message::Write("\u{a}".into())), // LF
                KeyCode::Char('k') => Some(Message::Write("\u{b}".into())), // VT
                KeyCode::Char('l') => Some(Message::Write("\u{c}".into())), // FF
                KeyCode::Char('m') => Some(Message::Write("\u{d}".into())), // CR
                KeyCode::Char('n') => Some(Message::Write("\u{e}".into())), // SO
                KeyCode::Char('o') => Some(Message::Write("\u{f}".into())), // SI
                KeyCode::Char('p') => Some(Message::Write("\u{10}".into())), // DLE
                KeyCode::Char('q') => Some(Message::Write("\u{11}".into())), // DC1
                KeyCode::Char('r') => Some(Message::Write("\u{12}".into())), // DC2
                KeyCode::Char('s') => Some(Message::Write("\u{13}".into())), // DC3
                KeyCode::Char('t') => Some(Message::Write("\u{14}".into())), // DC4
                KeyCode::Char('u') => Some(Message::Write("\u{15}".into())), // NAK
                KeyCode::Char('v') => Some(Message::Write("\u{16}".into())), // SYN
                KeyCode::Char('w') => Some(Message::Write("\u{17}".into())), // ETB
                KeyCode::Char('x') => Some(Message::Write("\u{18}".into())), // CAN
                KeyCode::Char('y') => Some(Message::Write("\u{19}".into())), // EM
                KeyCode::Char('z') => Some(Message::Write("\u{1a}".into())), // SUB

                _ => None,
            },

            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Prefix {}

impl shell::State for Prefix {
    fn color(&self) -> style::Color {
        style::Color::Yellow
    }

    fn cursor(&self) -> usize {
        0
    }

    fn contents(&self) -> &str {
        ""
    }

    fn name(&self) -> &str {
        "PREFIX"
    }

    fn keybinds(&self) -> Vec<&str> {
        vec!["q Quit", "r Raw", "l Line", "^\\ Return"]
    }
}

impl OnKey for Prefix {
    fn on_key(&mut self, key: KeyEvent) -> Option<Message> {
        match key {
            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                code,
                ..
            } => match code {
                KeyCode::Char('q') => Some(Message::Quit()),
                KeyCode::Char('l') => Some(Message::ChangeMode(Mode::Line)),
                KeyCode::Char('r') => Some(Message::ChangeMode(Mode::Raw)),

                _ => None,
            },

            KeyEvent {
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                code,
                ..
            } => match code {
                KeyCode::Char('4') | KeyCode::Char('\\') => Some(Message::ChangeMode(Mode::Line)),

                _ => None,
            },

            _ => None,
        }
    }
}

pub struct Modes {
    line: Line,
    prefix: Prefix,
    raw: Raw,
    mode: Mode,
}

impl Modes {
    pub fn new(history: History) -> Self {
        Self {
            line: Line::new(history),
            prefix: Prefix::default(),
            raw: Raw::default(),
            mode: Mode::Line,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Option<Action> {
        let message = match self.mode {
            Mode::Line => self.line.on_key(key),
            Mode::Raw => self.raw.on_key(key),
            Mode::Prefix => self.prefix.on_key(key),
        };

        match message {
            Some(message) => match message {
                Message::ChangeMode(mode) => {
                    self.mode = mode;
                    None
                }
                Message::Writeline(line) => Some(Action::Writeline(line)),
                Message::Write(bytes) => Some(Action::Write(bytes)),
                Message::Quit() => Some(Action::Quit()),
            },
            None => None,
        }
    }
}

impl shell::State for Modes {
    fn color(&self) -> style::Color {
        match self.mode {
            Mode::Line => self.line.color(),
            Mode::Raw => self.raw.color(),
            Mode::Prefix => self.prefix.color(),
        }
    }

    fn cursor(&self) -> usize {
        match self.mode {
            Mode::Line => self.line.cursor(),
            Mode::Raw => self.raw.cursor(),
            Mode::Prefix => self.prefix.cursor(),
        }
    }

    fn contents(&self) -> &str {
        match self.mode {
            Mode::Line => self.line.contents(),
            Mode::Raw => self.raw.contents(),
            Mode::Prefix => self.prefix.contents(),
        }
    }

    fn name(&self) -> &str {
        match self.mode {
            Mode::Line => self.line.name(),
            Mode::Raw => self.raw.name(),
            Mode::Prefix => self.prefix.name(),
        }
    }

    fn keybinds(&self) -> Vec<&str> {
        match self.mode {
            Mode::Line => self.line.keybinds(),
            Mode::Raw => self.raw.keybinds(),
            Mode::Prefix => self.prefix.keybinds(),
        }
    }
}
