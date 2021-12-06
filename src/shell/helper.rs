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
}

impl EditorHelper {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
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

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        Cow::Borrowed(candidate)
    }
}

impl Hinter for EditorHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if let Some(search_result) =
            ctx.history()
                .starts_with(line, ctx.history().len() - 1, SearchDirection::Reverse)
        {
            Some(String::from(&search_result.entry[search_result.pos..]))
        } else {
            None
        }
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
