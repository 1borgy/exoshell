use unicode_width::UnicodeWidthStr;

// width=0
// ╭╮
// ││
// ╰╯
// width=1
// ╭(╮
// │ │
// ╰(╯
// width=2
// ╭()╮
// │  │
// ╰()╯
// width=3
// ╭(e)╮
// │   │
// ╰(M)╯
// width=10
// ╭(exoshell)╮
// │          │
// ╰(MODE)(q )╯
// width=24
// ╭(exoshell)────────────╮
// │                      │
// ╰(MODE)(q Quit)(l Line)╯
// width=26
// ╭(exoshell)──────────────╮
// │                        │
// ╰(MODE)─(q Quit)─(l Line)╯
// ╭(exoshell but with long na)╮
// │                           │
// ╰(MODE)────(q Quit)─(l Line)╯

// pad by unicode character width, not bytes
//pub fn pad(value: impl ToString, width: usize, fill: char) -> String {
//    let mut value = value.to_string();
//
//    for _ in 0..width.saturating_sub(UnicodeWidthStr::width(value.as_str())) {
//        value.push(fill)
//    }
//
//    value
//}

pub struct Banner {
    left: Vec<Component>,
    right: Vec<Component>,
    fill: char,
}

impl Banner {
    pub fn new(fill: char) -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
            fill,
        }
    }

    pub fn push_left(mut self, c: Component) -> Self {
        self.left.push(c);
        self
    }

    pub fn push_right(mut self, c: Component) -> Self {
        self.right.push(c);
        self
    }

    pub fn render(&self, width: usize) -> String {
        let mut width_remaining = width;

        let mut left_components = Vec::new();
        let mut right_components = Vec::new();

        for component in self.left.iter() {
            let rendered = component.render(width_remaining);

            // Count unicode width to subtract instead of byte length of the string
            width_remaining =
                width_remaining.saturating_sub(UnicodeWidthStr::width(rendered.as_str()));

            left_components.push(rendered);
        }

        for component in self.right.iter() {
            let rendered = component.render(width_remaining);

            width_remaining =
                width_remaining.saturating_sub(UnicodeWidthStr::width(rendered.as_str()));

            right_components.push(rendered);
        }

        // If there is enough extra space to join each component by the fill char, do so
        // Otherwise, just fill between the left and right sides

        // e.g. 2 left components and 3 right components: need 1 fill char for left, 2 fill chars
        // for right
        let num_fill_chars =
            left_components.len().saturating_sub(1) + right_components.len().saturating_sub(1);
        let can_join = width_remaining > num_fill_chars;

        if can_join {
            width_remaining = width_remaining.saturating_sub(num_fill_chars)
        }

        let join = match can_join {
            true => &self.fill.to_string(),
            false => "",
        };

        let left_rendered = left_components.join(join);
        let right_rendered = right_components.join(join);

        let mut output = String::new();

        output.push_str(&left_rendered);
        for _ in 0..width_remaining {
            output.push(self.fill)
        }
        output.push_str(&right_rendered);

        output
    }
}

pub struct Component {
    left: char,
    value: String,
    right: char,
}

impl Component {
    pub fn new(left: char, value: impl ToString, right: char) -> Self {
        Self {
            left,
            value: value.to_string(),
            right,
        }
    }
}

impl Default for Component {
    fn default() -> Self {
        Self {
            value: String::new(),
            left: ' ',
            right: ' ',
        }
    }
}

impl Component {
    pub fn render(&self, max_width: usize) -> String {
        if max_width == 0 {
            String::new()
        } else if max_width == 1 {
            self.left.to_string()
        } else if max_width == 2 {
            format!("{}{}", self.left, self.right)
        } else {
            format!(
                "{}{}{}",
                self.left,
                &self.value[..(max_width - self.left.len_utf8() - self.right.len_utf8())
                    .min(self.value.len())],
                self.right
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::Component;

    fn component_len0() -> Component {
        Component {
            left: '<',
            right: '>',
            ..Default::default()
        }
    }

    #[test]
    fn component_len0_width0() {
        assert_eq!("", component_len0().render(0))
    }

    #[test]
    fn component_len0_width1() {
        assert_eq!("<", component_len0().render(1))
    }

    #[test]
    fn component_len0_width2() {
        assert_eq!("<>", component_len0().render(2))
    }

    #[test]
    fn component_len0_width_3() {
        // Does not increase in size
        assert_eq!("<>", component_len0().render(3))
    }

    fn component_len5() -> Component {
        Component {
            left: '<',
            right: '>',
            value: "abcde".to_string(),
        }
    }

    #[test]
    fn component_len5_width_3() {
        assert_eq!("<a>", component_len5().render(3))
    }

    #[test]
    fn component_len5_width7() {
        assert_eq!("<abcde>", component_len5().render(7))
    }

    #[test]
    fn component_len5_width8() {
        // Should not increase in size
        assert_eq!("<abcde>", component_len5().render(8))
    }
}
