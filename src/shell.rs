use crate::banner::{Banner, Component};
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, QueueableCommand,
};
use std::io;
use unicode_width::UnicodeWidthChar;

struct Border {
    horizontal: char,
    vertical: char,
    upper_left: char,
    upper_right: char,
    lower_left: char,
    lower_right: char,
}

impl Border {
    pub fn create(w: boxy::Weight, s: boxy::Style) -> Border {
        Border {
            horizontal: boxy::Char::horizontal(w).style(s).into(),
            vertical: boxy::Char::vertical(w).style(s).into(),
            upper_left: boxy::Char::upper_left(w).style(s).into(),
            upper_right: boxy::Char::upper_right(w).style(s).into(),
            lower_left: boxy::Char::lower_left(w).style(s).into(),
            lower_right: boxy::Char::lower_right(w).style(s).into(),
        }
    }
}

pub trait State {
    fn color(&self) -> style::Color;
    fn cursor(&self) -> usize;
    fn contents(&self) -> &str;
    fn name(&self) -> &str;
    fn keybinds(&self) -> Vec<&str>;
}

pub struct Shell {
    border: Border,
    titles: Vec<String>,
    cols: usize,
    // last *known* cursor position
    cursor: (usize, usize),
}

impl Shell {
    pub fn new(cols: impl Into<usize>) -> io::Result<Self> {
        let border = Border::create(boxy::Weight::Normal, boxy::Style::Curved);

        Ok(Self {
            border,
            titles: Vec::new(),
            cols: cols.into(),
            cursor: (0, 0),
        })
    }

    pub fn push_title(&mut self, title: impl ToString) {
        self.titles.push(title.to_string());
    }

    pub fn write(
        &mut self,
        stream: &mut impl QueueableCommand,
        state: &impl State,
    ) -> io::Result<()> {
        // All relative to inner content
        let width = self.cols - 2;
        let contents = state.contents().to_string();
        let color = state.color();
        let cursor = state.cursor();

        let left = '(';
        let right = ')';

        let header = self
            .titles
            .iter()
            .fold(Banner::new(self.border.horizontal), |banner, title| {
                banner.push_left(Component::new(left, title, right))
            });

        stream.queue(style::PrintStyledContent(
            format!(
                "\r{}{}{}\r\n",
                self.border.upper_left,
                header.render(width),
                self.border.upper_right
            )
            .with(color),
        ))?;

        stream.queue(style::PrintStyledContent(self.border.vertical.with(color)))?;

        let mut current_row = 0;
        let mut current_col = 0;

        let mut cursor_row = 0;
        let mut cursor_col = 0;

        for (index, c) in contents.chars().chain(vec![' ']).enumerate() {
            let char_cols = UnicodeWidthChar::width(c).unwrap_or(0);

            if index == cursor {
                cursor_row = current_row;
                cursor_col = current_col;
            }

            if current_col + char_cols <= width {
                // if it fits, print it
                current_col += char_cols;
                stream.queue(style::Print(c))?;
            } else {
                // pad with spaces if we can't fit a full character
                for _ in current_col..width {
                    stream.queue(style::Print(" "))?;
                }

                current_col = char_cols;
                current_row += 1;

                // print the border character instead
                stream.queue(style::PrintStyledContent(
                    format!("{0}\r\n{0}", self.border.vertical).with(color),
                ))?;
                stream.queue(style::Print(format!("{}", c)))?;
            }
        }

        self.cursor = (cursor_row, cursor_col); // (row, col)

        for _ in 0..(width - current_col) {
            stream.queue(style::Print(" "))?;
        }

        stream.queue(style::PrintStyledContent(
            format!("{}\r\n", self.border.vertical).with(color),
        ))?;

        let mut footer = Banner::new(self.border.horizontal).push_left(Component::new(
            left,
            state.name(),
            right,
        ));

        for keybind in state.keybinds() {
            footer = footer.push_right(Component::new(left, keybind, right));
        }

        stream
            .queue(style::PrintStyledContent(
                format!(
                    "{}{}{}\r", // important: no \n
                    self.border.lower_left,
                    footer.render(width),
                    self.border.lower_right
                )
                .with(color),
            ))?
            .queue(cursor::MoveUp((current_row - cursor_row + 1) as u16))?
            .queue(cursor::MoveRight((cursor_col + 1) as u16))?;

        Ok(())
    }

    pub fn clear(&self, stream: &mut impl QueueableCommand) -> io::Result<()> {
        let (cursor_row, _) = self.cursor;

        stream.queue(cursor::MoveUp((cursor_row + 1) as u16))?;
        stream.queue(style::Print("\r"))?;
        stream.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

        Ok(())
    }

    pub fn resize(&mut self, cols: impl Into<usize>) -> io::Result<()> {
        let cols = cols.into();

        // If zoom in, then we need to clear a few extra lines due to word wrap
        if cols < self.cols {
            let (cursor_row, _) = self.cursor;

            // How many extra rows are added per previous row, due to the resize
            let scale_factor = self.cols / cols;
            // If the cursor overflowed to a new row, we need to add 1
            let extra_rows = (cursor_row) * scale_factor;

            let mut stdout = io::stdout();
            stdout.queue(cursor::MoveUp((extra_rows) as u16))?;
            stdout.queue(style::Print("\r"))?;
            stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
        }

        self.cols = cols;

        Ok(())
    }
}
