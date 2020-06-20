pub const DEFAULT_INDENT_SIZE: usize = 4;

pub struct CodeBuilder {
    code: Vec<String>,
    indent: usize,
}

impl Default for CodeBuilder {
    fn default() -> Self {
        Self::new(DEFAULT_INDENT_SIZE)
    }
}

impl CodeBuilder {
    pub fn new(indent: usize) -> Self {
        Self {
            code: vec![],
            indent,
        }
    }

    pub fn push<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.code.push(s.into());
        self
    }

    pub fn push_and_pad<S: Into<String>>(&mut self, s: S, depth: usize) -> &mut Self {
        self.code.push(s.into());
        self.pad_last(depth)
    }

    pub fn append<S: Into<String>>(&mut self, s: S) -> &mut Self {
        match self.code.last_mut() {
            Some(last) => last.push_str(&s.into()),
            None => self.code.push(s.into()),
        }
        self
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut String> {
        self.code.last_mut()
    }

    pub fn pad_last(&mut self, depth: usize) -> &mut Self {
        match self.code.last_mut() {
            Some(s) => *s = pad(s.clone(), depth, self.indent),
            None => (),
        }
        self
    }

    pub fn build(&mut self) -> String {
        self.code.join("\n")
    }
}

pub fn pad<S: Into<String>>(s: S, depth: usize, indent: usize) -> String {
    if depth == 0 {
        s.into()
    } else {
        format!("{}{}", " ".repeat(indent * depth), s.into())
    }
}
