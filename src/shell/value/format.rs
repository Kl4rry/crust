use std::{
    fmt::{self, Display},
    iter,
};

use unicode_width::UnicodeWidthStr;
use yansi::Paint;

use super::Value;

#[inline(always)]
pub fn bar() -> Paint<char> {
    Paint::rgb(171, 178, 191, '│')
}

pub fn format_columns<'a, T: Display, I: Iterator<Item = (T, &'a Value)>>(
    f: &mut fmt::Formatter<'_>,
    list: I,
) -> fmt::Result {
    let mut longest_value = 0;
    let mut longest_key = 0;
    let mut values = Vec::new();
    let mut keys = Vec::new();
    for (key, value) in list {
        values.push(value.to_compact_string());
        longest_value = std::cmp::max(
            longest_value,
            console::strip_ansi_codes(unsafe { values.last().unwrap_unchecked() }).width_cjk(),
        );

        keys.push(Paint::green(key).to_string());
        longest_key = std::cmp::max(
            longest_key,
            console::strip_ansi_codes(unsafe { keys.last().unwrap_unchecked() }).width_cjk(),
        );
    }

    fmt_horizontal(f, &[longest_key + 2, longest_value + 2], ConfigChars::TOP)?;

    let bar = bar();
    for (key, value) in keys.into_iter().zip(values) {
        let key_spacing = longest_key - console::strip_ansi_codes(&key).width_cjk();
        let value_spacing = longest_value - console::strip_ansi_codes(&value).width_cjk();
        writeln!(
            f,
            "{bar} {:key_spacing$}{} {bar} {:value_spacing$}{} {bar}",
            "", key, "", value
        )?;
    }

    fmt_horizontal(f, &[longest_key + 2, longest_value + 2], ConfigChars::TOP)?;

    Ok(())
}

pub struct ConfigChars {
    pub left: char,
    pub middle: char,
    pub right: char,
}

impl ConfigChars {
    pub const fn new(left: char, middle: char, right: char) -> Self {
        Self {
            left,
            middle,
            right,
        }
    }

    pub const TOP: Self = Self::new('╭', '┬', '╮');
    pub const MID: Self = Self::new('├', '┼', '┤');
    pub const BOT: Self = Self::new('╰', '┴', '╯');
}

pub fn fmt_horizontal(
    f: &mut fmt::Formatter<'_>,
    cols: &[usize],
    ConfigChars {
        left,
        middle,
        right,
    }: ConfigChars,
) -> fmt::Result {
    let mut line = String::new();
    line.push(left);
    let mut peekable = cols.iter().peekable();
    while let Some(col) = peekable.next() {
        for _ in 0..*col {
            line.push('─');
        }
        if peekable.peek().is_some() {
            line.push(middle);
        }
    }
    line.push(right);
    line.push('\n');
    write!(f, "{}", Paint::rgb(171, 178, 191, line))?;
    Ok(())
}

pub fn center_pad(content: impl Display, width: usize) -> String {
    let string = content.to_string();
    let content_width = console::strip_ansi_codes(&string).width_cjk();
    debug_assert!(width >= content_width);
    let difference = width - content_width;
    let left = difference / 2;
    let right = difference - left;

    let mut new = String::new();
    new.extend(iter::repeat(' ').take(left));
    new.push_str(&string);
    new.extend(iter::repeat(' ').take(right));
    new
}

pub fn left_pad(content: impl Display, width: usize) -> String {
    let string = content.to_string();
    let content_width = console::strip_ansi_codes(&string).width_cjk();
    debug_assert!(width >= content_width);
    let left = width - content_width;
    let mut new = String::new();
    new.extend(iter::repeat(' ').take(left));
    new.push_str(&string);
    new
}
