use rustyline::{Helper, completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator};
use std::borrow::Cow;
use yansi::Paint;

pub struct EditorHelper;

impl Completer for EditorHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let _ = (line, pos, ctx);
        Ok((0, Vec::with_capacity(0)))
    }

    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected)
    }
}

impl Highlighter for EditorHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(Paint::new(hint).dimmed().to_string())
    }
}

impl Hinter for EditorHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _: &rustyline::Context<'_>) -> Option<Self::Hint> {
        /*if pos == line.len() {
            Some(String::from("test"))
        } else {
            None
        }*/
        None
    }
}

impl Validator for EditorHelper {}

impl Helper for EditorHelper {}
