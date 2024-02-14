use std::{borrow::Cow, fmt::Write, sync::Arc};

use crossterm::style::Stylize;
use rustyline::{
    completion::{Completer, Pair},
    highlight,
    hint::Hinter,
    history::History,
    validate::Validator,
    Changeset, Helper,
};

mod completer;
use completer::FilenameCompleter;

mod highlighter;

use self::highlighter::{ColorType, HighlightVisitor};
use super::history::JsonHistory;
use crate::parser::{ast::Ast, source::Source, Parser};

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

    fn update(
        &self,
        line: &mut rustyline::line_buffer::LineBuffer,
        start: usize,
        elected: &str,
        changeset: &mut Changeset,
    ) {
        let end = line.pos();
        line.replace(start..end, elected, changeset)
    }
}

impl highlight::Highlighter for EditorHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(hint.dim().to_string())
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, _: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Borrowed(&self.prompt)
    }

    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        if line.is_empty() {
            Cow::Borrowed(line)
        } else {
            let highlighter = Highlighter::new(line);
            Cow::Owned(highlighter.highlight())
        }
    }

    fn highlight_char(&self, _: &str, _: usize, _: bool) -> bool {
        true
    }
}

pub struct Highlighter<'a> {
    ast: Ast,
    index: usize,
    line: &'a str,
    output: String,
}

impl<'a> Highlighter<'a> {
    fn new(line: &'a str) -> Self {
        Self {
            ast: Parser::new(String::new(), line.to_string())
                .parse()
                .unwrap_or_else(|_| {
                    Ast::new(
                        Vec::new(),
                        Arc::new(Source::new(String::new(), line.to_string())),
                    )
                }),
            index: 0,
            line,
            output: String::new(),
        }
    }

    fn highlight(mut self) -> String {
        let mut visitor = HighlightVisitor::default();
        visitor.visit_ast(&self.ast);
        for span in visitor.spans {
            let _ = write!(
                self.output,
                "{}",
                &self.line[self.index..span.span.start()].with(ColorType::Base.to_color()),
            );
            let _ = write!(
                self.output,
                "{}",
                &self.line[span.span.start()..span.span.end()].with(span.inner.to_color())
            );
            self.index = span.span.end();
        }
        let end = &self.line[self.index..];
        let _ = write!(&mut self.output, "{}", end.with(ColorType::Base.to_color()));

        self.output
    }
}

impl Hinter for EditorHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if ctx.history().is_empty() {
            return None;
        }

        // This ugly mess assumes that History is a JsonHistory and does a maybe safe cast
        let history = ctx.history() as *const dyn History;
        let history_ptr = unsafe { history.byte_offset_from(std::ptr::null::<JsonHistory>()) }
            as *const JsonHistory;
        let history = unsafe { &*history_ptr };

        let search_result = history.get_hint(line)?;
        Some(String::from(&search_result.entry[search_result.pos..]))
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
