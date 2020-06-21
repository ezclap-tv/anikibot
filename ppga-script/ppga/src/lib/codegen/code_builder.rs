//! Provides a utility struct for source code generation.

/// The number of indentation spaces in the resulting lua code.
pub const DEFAULT_INDENT_SIZE: usize = 4;

/// A mutable buffer of strings.
pub struct CodeBuilder {
    code: Vec<String>,
    indent: usize,
}

impl CodeBuilder {
    /// Creates a new empty [`CodeBuilder`] with the specified ident size.
    pub fn new(indent: usize) -> Self {
        Self {
            code: vec![],
            indent,
        }
    }

    /// Pushes a line of code into the buffer.
    pub fn push<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.code.push(s.into());
        self
    }

    /// Pads and pushes a line of code into the buffer.
    pub fn push_and_pad<S: Into<String>>(&mut self, s: S, depth: usize) -> &mut Self {
        self.code.push(s.into());
        self.pad_last(depth)
    }

    /// Appends a string to the last line in the buffer.
    /// Creates a new line if the buffer is empty.
    pub fn append<S: Into<String>>(&mut self, s: S) -> &mut Self {
        match self.code.last_mut() {
            Some(last) => last.push_str(&s.into()),
            None => self.code.push(s.into()),
        }
        self
    }

    /// Returns a mutable reference to the last line in the buffer.
    #[inline(always)]
    pub fn last_mut(&mut self) -> Option<&mut String> {
        self.code.last_mut()
    }

    /// Pads the last line the buffer. Does nothing the buffer is empty.
    pub fn pad_last(&mut self, depth: usize) -> &mut Self {
        match self.code.last_mut() {
            Some(s) => *s = pad(s.clone(), depth, self.indent),
            None => (),
        }
        self
    }

    /// Joins the lines in the buffer.
    pub fn build(&mut self) -> String {
        self.code.join("\n")
    }
}

/// Pads the given string with the `indent * depth` spaces.
pub(crate) fn pad<S: Into<String>>(s: S, depth: usize, indent: usize) -> String {
    if depth == 0 {
        s.into()
    } else {
        format!("{}{}", " ".repeat(indent * depth), s.into())
    }
}

impl Default for CodeBuilder {
    fn default() -> Self {
        Self::new(DEFAULT_INDENT_SIZE)
    }
}
