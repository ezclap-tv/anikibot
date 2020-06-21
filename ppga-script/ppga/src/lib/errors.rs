use logos::Span;

#[derive(Debug, Clone)]
pub struct LineSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: LineSpan,
    pub message: String,
}

impl ParseError {
    pub fn new<S: Into<String>>(line: usize, span: Span, message: S) -> ParseError {
        Self {
            span: LineSpan {
                line,
                start: span.start,
                end: span.end,
            },
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrCtx<'a> {
    pub errors: Vec<ParseError>,
    pub source: &'a str,
    pub lines: Vec<&'a str>,
}

impl<'a> ErrCtx<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            errors: vec![],
            source,
            lines: source.split("\n").collect::<Vec<_>>(),
        }
    }

    pub fn record(&mut self, e: ParseError) {
        self.errors.push(e)
    }

    #[inline(always)]
    pub fn had_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn report_all(&self) {
        eprintln!("{}", self.report_to_string());
    }

    pub fn report_to_string(&self) -> String {
        let mut result = vec![];

        for e in &self.errors {
            let line = self
                .lines
                .get(e.span.line)
                .or_else(|| self.lines.last())
                .unwrap()
                .trim();
            let prefix = format!("[Line {:03}]", e.span.line + 1);
            result.push(format!(
                "{} ParseError at `{}`: {}",
                prefix,
                &self.source[e.span.start..e.span.end],
                e.message
            ));
            result.push(format!("|\n|{}{}\n|", " ".repeat(4), line));
        }

        result.join("\n")
    }
}
