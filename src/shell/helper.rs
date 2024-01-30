use std::borrow::Cow;

use rustyline::{
    completion::{Completer, Pair},
    highlight,
    hint::Hinter,
    history::SearchDirection,
    validate::Validator,
    Changeset, Helper,
};
use yansi::Paint;

mod completer;
use completer::FilenameCompleter;

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
        Cow::Owned(Paint::new(hint).dimmed().to_string())
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, _: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Borrowed(&self.prompt)
    }

    /*fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        if line.is_empty() {
            Cow::Borrowed(line)
        } else {
            let highlighter = Highlighter::new(line);
            Cow::Owned(highlighter.highlight())
        }
    }*/

    fn highlight_char(&self, _: &str, _: usize) -> bool {
        true
    }
}

/*pub struct Highlighter<'a> {
    lexer: Lexer,
    index: usize,
    line: &'a str,
    output: String,
}

impl<'a> Highlighter<'a> {
    fn new(line: &'a str) -> Self {
        Self {
            lexer: Lexer::new(Source::new(String::new(), line.to_string()).into()),
            index: 0,
            line,
            output: String::new(),
        }
    }

    fn highlight(mut self) -> String {
        for token in self.lexer.by_ref() {
            unsafe {
                self.output
                    .as_mut_vec()
                    .extend_from_slice(&self.line.as_bytes()[self.index..token.span.start()])
            }

            match &token.token_type {
                TokenType::Float(..) | TokenType::Int(..) | TokenType::True | TokenType::False => {
                    let s =
                        Paint::yellow(&self.line[token.span.start()..token.span.end()]).to_string();
                    self.output.push_str(&s);
                }
                token_type => {
                    if token_type.is_keyword() {
                        let s = Paint::magenta(&self.line[token.span.start()..token.span.end()])
                            .to_string();
                        self.output.push_str(&s);
                    } else {
                        self.output
                            .push_str(&self.line[token.span.start()..token.span.end()]);
                    }
                }
            }
            self.index = token.span.end();
        }
        let end = &self.lexer.src()[self.index..];
        self.output.push_str(&Paint::new(end).dimmed().to_string());

        self.output
    }
}*/

impl Hinter for EditorHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if ctx.history().is_empty() {
            return None;
        }

        let search_result =
            ctx.history()
                .starts_with(line, ctx.history().len() - 1, SearchDirection::Reverse);

        match search_result {
            Ok(Some(search_result)) => {
                Some(String::from(&search_result.entry[search_result.pos..]))
            }
            _ => None,
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
