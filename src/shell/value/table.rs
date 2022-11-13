use std::{
    cmp::{self, PartialEq},
    fmt::{self},
};

use indexmap::IndexMap;
use textwrap::core::display_width;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use yansi::Paint;

use super::{
    format::{bar, center_pad, fmt_horizontal, left_pad, right_pad, ConfigChars},
    Value,
};
use crate::parser::shell_error::ShellErrorKind;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<Value>>,
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn insert_map(&mut self, map: IndexMap<String, Value>) {
        let mut row = vec![Value::Null; self.headers.len()];
        'outer: for (k, v) in map {
            for (index, header) in self.headers.iter().enumerate() {
                if header == &k {
                    row[index] = v;
                    continue 'outer;
                }
            }
            self.add_column(k);
            row.push(v);
        }
        self.rows.push(row);
    }

    fn add_column(&mut self, name: String) {
        self.headers.push(name);
        for row in &mut self.rows {
            row.push(Value::Null);
        }
    }

    pub fn row(&self, index: usize) -> Result<IndexMap<String, Value>, ShellErrorKind> {
        let mut map = IndexMap::new();

        let row = match self.rows.get(index) {
            Some(row) => row,
            None => {
                return Err(ShellErrorKind::IndexOutOfBounds {
                    len: self.rows.len() as i128,
                    index: index as i128,
                })
            }
        };

        for (k, v) in self.headers.iter().zip(row) {
            map.insert(k.clone(), v.clone());
        }
        Ok(map)
    }

    pub fn column(&self, name: &str) -> Result<Vec<Value>, ShellErrorKind> {
        let index = match self.headers.iter().position(|h| h == name) {
            Some(index) => index,
            None => return Err(ShellErrorKind::ColumnNotFound(name.to_string())),
        };
        let mut values = Vec::new();
        for row in &self.rows {
            values.push(row[index].clone());
        }
        Ok(values)
    }

    pub fn has_column(&self, name: &str) -> bool {
        self.headers.iter().any(|h| h == name)
    }

    pub fn iter(&self) -> impl Iterator<Item = IndexMap<String, Value>> + '_ {
        self.rows.iter().map(|row| {
            self.headers
                .iter()
                .cloned()
                .zip(row.iter().cloned())
                .collect::<IndexMap<String, Value>>()
        })
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        let rows: Vec<Vec<String>> = self
            .rows
            .iter()
            .map(|r| r.iter().map(|c| c.to_compact_string()).collect())
            .collect();

        let mut column_widths = vec![(self.rows.len() - 1).to_string().width_cjk() + 2];
        column_widths.extend(self.headers.iter().map(|c| c.len() + 2));

        for row in &rows {
            for (index, col) in row.iter().enumerate() {
                let len = &mut column_widths[index + 1];
                *len = std::cmp::max(*len, display_width(col) + 2);
            }
        }

        let (term_width, _) = crossterm::terminal::size().unwrap_or((u16::MAX, u16::MAX));
        let max_width = term_width as i32 - column_widths.len() as i32 - 1;
        let mut rest = if max_width <= 0 {
            u16::MAX as usize
        } else {
            max_width as usize
        };

        let number_cols = column_widths.len();
        for (i, width) in column_widths.iter_mut().enumerate() {
            let max_col_width = rest / (number_cols - i);
            let col_width = cmp::min(*width, max_col_width);
            rest -= col_width;
            *width = col_width;
        }

        fmt_horizontal(f, &column_widths, ConfigChars::TOP)?;

        bar().fmt(f)?;
        center_pad(Paint::green('#'), column_widths[0]).fmt(f)?;
        bar().fmt(f)?;
        for (index, header) in self.headers.iter().enumerate() {
            center_pad(Paint::green(header), column_widths[index + 1]).fmt(f)?;
            bar().fmt(f)?;
        }
        writeln!(f)?;

        fmt_horizontal(f, &column_widths, ConfigChars::MID)?;

        for (number, row) in rows.iter().enumerate() {
            bar().fmt(f)?;
            left_pad(Paint::green(number), column_widths[0] - 1).fmt(f)?;
            ' '.fmt(f)?;
            bar().fmt(f)?;
            for (index, col) in row.iter().enumerate() {
                let mut ouput_col = "";
                for (i, grapheme) in col.grapheme_indices(true) {
                    if display_width(&col[..i + grapheme.len()]) < column_widths[index + 1] - 1 {
                        ouput_col = &col[..i + grapheme.len()];
                    } else {
                        break;
                    }
                }

                ' '.fmt(f)?;
                right_pad(ouput_col, column_widths[index + 1] - 2).fmt(f)?;
                ' '.fmt(f)?;
                bar().fmt(f)?;
            }
            writeln!(f)?;
        }

        fmt_horizontal(f, &column_widths, ConfigChars::BOT)?;
        Ok(())
    }
}
