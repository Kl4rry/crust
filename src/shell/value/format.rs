use std::fmt::{self, Display};

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, ContentArrangement,
};

use super::Value;

pub fn format_columns<T: Display, Item, I: Iterator<Item = (T, Item)>>(
    f: &mut fmt::Formatter<'_>,
    iterator: I,
) -> fmt::Result
where
    Item: AsRef<Value>,
{
    let mut table = comfy_table::Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    for (k, v) in iterator {
        let v = v.as_ref();
        table.add_row(vec![
            Cell::new(k).fg(Color::Green),
            Cell::new(v.to_compact_string()).fg(v.compact_string_color()),
        ]);
    }

    writeln!(f, "{table}")
}
