use std::borrow::Cow;

use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    highlight::Highlighter,
    hint::Hinter,
    history::SearchDirection,
    validate::Validator,
    Helper,
};
use yansi::Paint;

pub struct EditorHelper {
    filename_completer: FilenameCompleter,
    pub prompt: String,
}

impl EditorHelper {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            prompt: String::new(),
        }
    }
}

impl Completer for EditorHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.filename_completer.complete_path(line, pos)
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

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, _: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Borrowed(&self.prompt)
    }
}

impl Hinter for EditorHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if ctx.history().is_empty() {
            return None;
        }

        ctx.history()
            .starts_with(line, ctx.history().len() - 1, SearchDirection::Reverse)
            .map(|search_result| String::from(&search_result.entry[search_result.pos..]))
    }
}

impl Validator for EditorHelper {
    /*fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        let mut parser = Parser::new(ctx.input().to_string());
        match parser.parse() {
            Ok(_) => Ok(ValidationResult::Valid(None)),
            Err(error) => match error.error {
                SyntaxErrorKind::ExpectedToken => Ok(ValidationResult::Incomplete),
                _ => Ok(ValidationResult::Invalid(None)),
            },
        }
    }*/
}

impl Helper for EditorHelper {}
