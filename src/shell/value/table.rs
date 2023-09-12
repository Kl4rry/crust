use std::{
    cmp::PartialEq,
    fmt::{self},
    iter, mem,
    ops::Deref,
    rc::Rc,
};

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, ContentArrangement,
};
use indexmap::IndexMap;

use super::{SpannedValue, Value};
use crate::parser::shell_error::ShellErrorKind;

#[repr(transparent)]
pub struct ConstVec<T>(Vec<T>);

impl<T> ConstVec<T>
where
    T: Clone,
{
    pub fn to_vec(&self) -> Vec<T> {
        self.0.clone()
    }
}

impl<T> Deref for ConstVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    headers: Vec<Rc<str>>,
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

    pub fn insert_map(&mut self, map: IndexMap<Rc<str>, Value>) {
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

    fn add_column(&mut self, name: Rc<str>) {
        self.headers.push(name);
        for row in &mut self.rows {
            row.push(Value::Null);
        }
    }

    pub fn row(&self, index: SpannedValue) -> Result<IndexMap<Rc<str>, Value>, ShellErrorKind> {
        let index = index.try_as_index(self.rows.len())?;
        let mut map = IndexMap::new();

        let row = self.rows.get(index).unwrap();

        for (k, v) in self.headers.iter().zip(row) {
            map.insert(k.clone(), v.clone());
        }
        Ok(map)
    }

    pub fn column(&self, name: &str) -> Result<Vec<Value>, ShellErrorKind> {
        let index = match self.headers.iter().position(|h| &**h == name) {
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
        self.headers.iter().any(|h| &**h == name)
    }

    pub fn iter(&self) -> impl Iterator<Item = IndexMap<Rc<str>, Value>> + '_ {
        self.rows.iter().map(|row| {
            self.headers
                .iter()
                .cloned()
                .zip(row.iter().cloned())
                .collect::<IndexMap<Rc<str>, Value>>()
        })
    }

    pub fn rows_mut(&mut self) -> &mut [ConstVec<Value>] {
        unsafe { mem::transmute::<_, &mut [ConstVec<Value>]>(&mut *self.rows) }
    }

    pub fn rows(&self) -> &[Vec<Value>] {
        &self.rows
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        let mut table = comfy_table::Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        let headers = iter::once(Cell::new("#").fg(Color::Green))
            .chain(self.headers.iter().map(|s| Cell::new(s).fg(Color::Green)));
        table.set_header(headers);

        for (index, row) in self.rows.iter().enumerate() {
            let row = iter::once(Cell::new(index + 1).fg(Color::Green)).chain(
                row.iter()
                    .map(|v| Cell::new(v.to_compact_string()).fg(v.compact_string_color())),
            );
            table.add_row(row);
        }

        writeln!(f, "{table}")
    }
}
