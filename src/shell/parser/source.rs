use miette::SourceCode;

#[derive(Debug, Clone)]
pub struct Source {
    pub name: String,
    pub code: String,
}

impl Source {
    pub fn new(name: String, code: String) -> Self {
        Self { name, code }
    }
}

impl SourceCode for Source {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        self.code
            .read_span(span, context_lines_before, context_lines_after)
    }
}
