use crate::Action;
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::{self},
};

#[derive(Clone)]
pub struct Line {
    contents: String,
    cursor: usize,
}

impl Line {
    pub fn empty() -> Self {
        Self {
            contents: String::new(),
            cursor: 0,
        }
    }
}

#[derive(Clone)]
pub struct Raw {}

#[derive(Clone)]
pub struct Prefix {
    previous: Box<Mode>,
}

#[derive(Clone)]
pub enum Mode {
    Line(Line),
    Raw(Raw),
    Prefix(Prefix),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Line(Line::empty())
        //Self::Line(Line { contents: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(), cursor: 0} )
    }
}

impl Line {
    fn color(&self) -> style::Color {
        style::Color::Green
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn contents(&self) -> &str {
        &self.contents
    }

    fn on_key(self, key: KeyEvent) -> (Mode, Option<Action>) {
        match key {
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Home,
                ..
            }
            | KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('a'),
                ..
            } => (
                Mode::Line(Line {
                    contents: self.contents,
                    cursor: 0,
                }),
                None,
            ),

            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::End,
                ..
            }
            | KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('e'),
                ..
            } => {
                let cursor = self.contents.chars().count();
                (
                    Mode::Line(Line {
                        contents: self.contents,
                        cursor,
                    }),
                    None,
                )
            }

            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(c),
                ..
            }
            | KeyEvent {
                modifiers: KeyModifiers::SHIFT,
                code: KeyCode::Char(c),
                ..
            } => {
                let mut chars = self.contents.chars();

                (
                    Mode::Line(Line {
                        contents: format!(
                            "{}{}{}",
                            chars.by_ref().take(self.cursor).collect::<String>(),
                            c,
                            chars.by_ref().collect::<String>()
                        ),
                        cursor: self.cursor + 1,
                    }),
                    None,
                )
            }

            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code,
                ..
            } => match code {
                KeyCode::Enter => (
                    Mode::Line(Line::empty()),
                    Some(Action::Writeline(self.contents)),
                ),

                KeyCode::Left => (
                    Mode::Line(Line {
                        contents: self.contents,
                        // For left, a saturating sub is sufficient since cursor is unsigned
                        cursor: self.cursor.saturating_sub(1),
                    }),
                    None,
                ),
                KeyCode::Right => {
                    // For right, we need to clamp by the length of the current contents
                    let cursor = (self.cursor + 1).min(self.contents.chars().count());
                    (
                        Mode::Line(Line {
                            contents: self.contents,
                            cursor,
                        }),
                        None,
                    )
                }

                KeyCode::Backspace => {
                    if self.cursor > 0 {
                        let cursor = self.cursor - 1;
                        let mut chars = self.contents.chars();

                        let left = chars.by_ref().take(cursor).collect::<String>();
                        chars.by_ref().next();
                        let right = chars.by_ref().collect::<String>();

                        let contents = format!("{}{}", left, right);

                        (Mode::Line(Line { contents, cursor }), None)
                    } else {
                        (Mode::Line(self), None)
                    }
                }
                KeyCode::Delete => {
                    if self.cursor < self.contents.chars().count() {
                        let mut chars = self.contents.chars();

                        let left = chars.by_ref().take(self.cursor).collect::<String>();
                        chars.by_ref().next();
                        let right = chars.by_ref().collect::<String>();

                        let contents = format!("{}{}", left, right);

                        (
                            Mode::Line(Line {
                                contents,
                                cursor: self.cursor,
                            }),
                            None,
                        )
                    } else {
                        (Mode::Line(self), None)
                    }
                }

                // TODO: history with up/down arrows
                _ => (Mode::Line(self), None),
            },

            KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code,
                ..
            } => match code {
                KeyCode::Char('d') => (Mode::Line(self), Some(Action::Quit())),
                KeyCode::Char('c') => (
                    Mode::Line(Line {
                        contents: "".to_string(),
                        cursor: 0,
                    }),
                    None,
                ),
                KeyCode::Char('4') => (
                    Mode::Prefix(Prefix {
                        previous: Box::new(Mode::Line(self)),
                    }),
                    None,
                ),

                // TODO: C-Backspace / C-Left / C-Right
                _ => (Mode::Line(self), None),
            },

            _ => (Mode::Line(self), None),
        }
    }
}

impl Raw {
    fn color(&self) -> style::Color {
        style::Color::Red
    }

    fn cursor(&self) -> usize {
        0
    }

    fn contents(&self) -> &str {
        ""
    }

    fn on_key(self, key: KeyEvent) -> (Mode, Option<Action>) {
        match key {
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(c),
                ..
            }
            | KeyEvent {
                modifiers: KeyModifiers::SHIFT,
                code: KeyCode::Char(c),
                ..
            } => (Mode::Raw(self), Some(Action::Write(c))),

            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code,
                ..
            } => match code {
                KeyCode::Enter => (Mode::Raw(self), Some(Action::Write('\n'))),
                KeyCode::Backspace => (Mode::Raw(self), Some(Action::Write('\u{7f}'))),

                _ => (Mode::Raw(self), None),
            },

            KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code,
                ..
            } => match code {
                KeyCode::Char('4') => (
                    Mode::Prefix(Prefix {
                        previous: Box::new(Mode::Raw(self)),
                    }),
                    None,
                ),

                KeyCode::Char('a') => (Mode::Raw(self), Some(Action::Write('\u{1}'))), // SOH
                KeyCode::Char('b') => (Mode::Raw(self), Some(Action::Write('\u{2}'))), // STX
                KeyCode::Char('c') => (Mode::Raw(self), Some(Action::Write('\u{3}'))), // ETX
                KeyCode::Char('d') => (Mode::Raw(self), Some(Action::Write('\u{4}'))), // EOT
                KeyCode::Char('e') => (Mode::Raw(self), Some(Action::Write('\u{5}'))), // ENQ
                KeyCode::Char('f') => (Mode::Raw(self), Some(Action::Write('\u{6}'))), // EOT
                KeyCode::Char('g') => (Mode::Raw(self), Some(Action::Write('\u{7}'))), // EOT
                KeyCode::Char('h') => (Mode::Raw(self), Some(Action::Write('\u{8}'))), // BS
                KeyCode::Char('i') => (Mode::Raw(self), Some(Action::Write('\u{9}'))), // HT
                KeyCode::Char('j') => (Mode::Raw(self), Some(Action::Write('\u{a}'))), // LF
                KeyCode::Char('k') => (Mode::Raw(self), Some(Action::Write('\u{b}'))), // VT
                KeyCode::Char('l') => (Mode::Raw(self), Some(Action::Write('\u{c}'))), // FF
                KeyCode::Char('m') => (Mode::Raw(self), Some(Action::Write('\u{d}'))), // CR
                KeyCode::Char('n') => (Mode::Raw(self), Some(Action::Write('\u{e}'))), // SO
                KeyCode::Char('o') => (Mode::Raw(self), Some(Action::Write('\u{f}'))), // SI
                KeyCode::Char('p') => (Mode::Raw(self), Some(Action::Write('\u{10}'))), // DLE
                KeyCode::Char('q') => (Mode::Raw(self), Some(Action::Write('\u{11}'))), // DC1
                KeyCode::Char('r') => (Mode::Raw(self), Some(Action::Write('\u{12}'))), // DC2
                KeyCode::Char('s') => (Mode::Raw(self), Some(Action::Write('\u{13}'))), // DC3
                KeyCode::Char('t') => (Mode::Raw(self), Some(Action::Write('\u{14}'))), // DC4
                KeyCode::Char('u') => (Mode::Raw(self), Some(Action::Write('\u{15}'))), // NAK
                KeyCode::Char('v') => (Mode::Raw(self), Some(Action::Write('\u{16}'))), // SYN
                KeyCode::Char('w') => (Mode::Raw(self), Some(Action::Write('\u{17}'))), // ETB
                KeyCode::Char('x') => (Mode::Raw(self), Some(Action::Write('\u{18}'))), // CAN
                KeyCode::Char('y') => (Mode::Raw(self), Some(Action::Write('\u{19}'))), // EM
                KeyCode::Char('z') => (Mode::Raw(self), Some(Action::Write('\u{1a}'))), // SUB

                _ => (Mode::Raw(self), None),
            },

            _ => (Mode::Raw(self), None),
        }
    }
}

impl Prefix {
    fn color(&self) -> style::Color {
        style::Color::Yellow
    }

    fn cursor(&self) -> usize {
        0
    }

    fn contents(&self) -> &str {
        ""
    }

    fn on_key(self, key: KeyEvent) -> (Mode, Option<Action>) {
        match key {
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code,
                ..
            } => match code {
                KeyCode::Char('q') => (Mode::Prefix(self), Some(Action::Quit())),
                KeyCode::Char('l') => (
                    Mode::Line(match *self.previous {
                        Mode::Line(line) => line,
                        _ => Line::empty(),
                    }),
                    None,
                ),
                KeyCode::Char('r') => (Mode::Raw(Raw {}), None),

                _ => (Mode::Prefix(self), None),
            },

            KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code,
                ..
            } => match code {
                KeyCode::Char('4') => (*self.previous, None),

                _ => (Mode::Prefix(self), None),
            },

            _ => (Mode::Prefix(self), None),
        }
    }
}

impl Mode {
    pub fn color(&self) -> style::Color {
        match self {
            Mode::Line(line) => line.color(),
            Mode::Raw(raw) => raw.color(),
            Mode::Prefix(prefix) => prefix.color(),
        }
    }

    pub fn cursor(&self) -> usize {
        match self {
            Mode::Line(line) => line.cursor(),
            Mode::Raw(raw) => raw.cursor(),
            Mode::Prefix(prefix) => prefix.cursor(),
        }
    }

    pub fn contents(&self) -> &str {
        match self {
            Mode::Line(line) => line.contents(),
            Mode::Raw(raw) => raw.contents(),
            Mode::Prefix(prefix) => prefix.contents(),
        }
    }

    pub fn on_key(self, key: KeyEvent) -> (Mode, Option<Action>) {
        match self {
            Mode::Line(line) => line.on_key(key),
            Mode::Raw(raw) => raw.on_key(key),
            Mode::Prefix(prefix) => prefix.on_key(key),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Mode::Line(_) => "LINE",
            Mode::Raw(_) => "RAW",
            Mode::Prefix(_) => "PREFIX",
        }
    }

    pub fn keybinds(&self) -> Vec<&str> {
        match self {
            Mode::Line(_) => vec!["^D Quit", "^\\ Prefix"],
            Mode::Raw(_) => vec!["^\\ Prefix"],
            Mode::Prefix(_) => vec!["q Quit", "r Raw", "l Line", "^\\ Return"],
        }
    }
}
