use std::fmt::{self, Display};

use unicode_width::UnicodeWidthStr;
use yansi::Paint;

use super::Value;

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

    fmt_top(f, &[longest_key + 2, longest_value + 2])?;

    let bar = Paint::rgb(171, 178, 191, "│");
    for (key, value) in keys.into_iter().zip(values) {
        let key_spacing = longest_key - console::strip_ansi_codes(&key).width_cjk();
        let value_spacing = longest_value - console::strip_ansi_codes(&value).width_cjk();
        writeln!(
            f,
            "{bar} {:key_spacing$}{} {bar} {:value_spacing$}{} {bar}",
            "", key, "", value
        )?;
    }

    fmt_bot(f, &[longest_key + 2, longest_value + 2])?;

    Ok(())
}

fn fmt_top(f: &mut fmt::Formatter<'_>, cols: &[usize]) -> fmt::Result {
    let mut line = String::new();
    line.push('╭');
    let mut peekable = cols.iter().peekable();
    while let Some(col) = peekable.next() {
        for _ in 0..*col {
            line.push('─');
        }
        if peekable.peek().is_some() {
            line.push('┬');
        }
    }
    line.push_str("╮\n");
    write!(f, "{}", Paint::rgb(171, 178, 191, line))?;
    Ok(())
}

fn fmt_bot(f: &mut fmt::Formatter<'_>, cols: &[usize]) -> fmt::Result {
    let mut line = String::new();
    line.push('╰');
    let mut peekable = cols.iter().peekable();
    while let Some(col) = peekable.next() {
        for _ in 0..*col {
            line.push('─');
        }
        if peekable.peek().is_some() {
            line.push('┴');
        }
    }
    line.push_str("╯\n");
    write!(f, "{}", Paint::rgb(171, 178, 191, line))?;
    Ok(())
}
