//! Implements the PPGA Lexer using the [`Logos`] crate.
//!
//! [`Logos`]: https://crates.io/crates/logos
use logos::{Logos, Span};

/// PPGAs numeric type.
pub type Number = f64;

/// A token that is used to add extra information to the [`TokenKind`]s returned by the lexer.
///
/// [`TokenKind`]: crate::frontend::lexer::TokenKind
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub span: Span,
    pub line: usize,
    pub kind: TokenKind<'a>,
}

impl<'a> Token<'a> {
    /// Creates a new token.
    pub fn new(lexeme: &'a str, line: usize, span: Span, kind: TokenKind<'a>) -> Self {
        Self {
            lexeme,
            line,
            span,
            kind,
        }
    }
}

/// The kind of a PPGA token.
#[derive(Debug, PartialEq, Clone, Logos)]
pub enum TokenKind<'a> {
    /// A single dot: `.`. Used in attribute access expressions.
    #[token(".")]
    Dot,

    /// Two dots in a row: `..`. Used for string concatenation.
    #[token("..")]
    DoubleDot,

    /// Three dots in a row: `...`. Used to unpack arrays/tables/function calls.
    #[token("...")]
    Ellipsis,

    /// A single `@`. Used to define variadic parameters in function declarations.
    #[token("@")]
    Variadics,

    /// A single `;`. Used to end statements.
    #[token(";")]
    Semicolon,

    /// A single `:`. Used to call static methods on tables.
    #[token(":")]
    Colon,

    /// A single `,`. Used for value separation in lists, dicts, and function and variable declarations.
    #[token(",")]
    Comma,

    /// A single `(`.
    #[token("(")]
    LeftParen,

    /// A single `)`.
    #[token(")")]
    RightParen,

    /// A single `{`. Used to open blocks and dict literals.
    #[token("{")]
    LeftBrace,

    /// A single `}`. Used to close blocks and dict literals.
    #[token("}")]
    RightBrace,

    /// A single `[`. Used to open attribute access expressions and list literals.
    #[token("[")]
    LeftBracket,

    /// A single `]`. Used to close attribute access expressions and list literals.
    #[token("]")]
    RightBracket,

    /// A fat arrow: `=>`. Used in single-expression functions.
    #[token("=>")]
    FatArrow,

    /// An ASCII identifier. Must start with a letter and may contain letters and numbers afterwards.
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    /// The `range` keyword.
    #[regex("range")]
    Range,

    /// The `len` keyword.
    #[regex("len")]
    Len,

    /// The `fn` keyword.
    #[token("fn")]
    Fn,

    /// A `?`. Used to indicate that an argument is optional or
    /// to automatically unpack let statements initialized with an ok/err pair.
    #[token("?")]
    Query,

    /// The default value operator: `??`. Evaluates the second operand if the first operand is `nil`.
    #[token("??")]
    DoubleQuery,

    /// The `let` keyword. Used for local variable definitions.
    #[token("let")]
    Let,

    /// The `global` keyword. Used for global variable and function definitions.
    #[token("global")]
    Global,

    /// The `break` keyword.
    #[token("break")]
    Break,

    // XXX: should we implement continue using gotos?
    // #[token("continue")]
    // Continue,
    //
    /// The `and` keyword.
    #[token("and")]
    And,
    /// The `or` keyword.
    #[token("or")]
    Or,
    /// The `in` keyword.
    #[token("in")]
    In,
    /// The `if` keyword.
    #[token("if")]
    If,

    /// The `else` keyword.
    #[token("else")]
    Else,
    /// The `while` keyword.
    #[token("while")]
    While,
    /// The `for` keyword.
    #[token("for")]
    For,
    /// The `fori` keyword.
    #[token("fori")]
    ForI,
    /// The `return` keyword.
    #[token("return")]
    Return,
    /// A `lua {}` block. The sequence of bytes inside the block
    /// is de-indented once and directly pasted into the lua file.
    #[regex(r"lua[\s\f]+\{", scan_lua_block)]
    Lua(&'a str),
    /// The `true` literal.
    #[token("true")]
    True,
    /// The `false` literal.
    #[token("false")]
    False,
    /// The `nil` literal.
    #[token("nil")]
    Nil,
    /// A floating point number.
    #[regex("[0-9]+(\\.[0-9]+)?")]
    Number,

    /// An interpolated string literal. Must start with an `f`: `f"a + b = {a + b}"`.
    /// An interpolated fragment may be escaped with a backslash: `f"I am \{ escaped }"`.
    #[regex("f\"([^\"\\\\]|\\\\.)*\"", interpolated_string)]
    #[regex("f'([^'\\\\]|\\\\.)*'", interpolated_string)]
    InterpolatedString(Vec<Frag<'a>>),

    /// A string literal.
    #[regex("\"([^\"\\\\]|\\\\.)*\"")]
    #[regex("'([^'\\\\]|\\\\.)*'")]
    StringLiteral,

    /// The `not` keyword.
    #[token("not")]
    Not,
    /// A single `*`. Used for multiplication.
    #[token("*")]
    Star,
    /// A single `/`. Used for float division.
    #[token("/")]
    Slash,
    /// A single `\`. Used for integer division.
    #[token("\\")]
    BackSlash,
    /// A single `%`. Used to find remainders.
    #[token("%")]
    Percent,
    /// Two asterisks in a row: `**`. Used for exponentiation.
    #[token("**")]
    Pow,
    /// A single `+`. Used for addition.
    #[token("+")]
    Plus,
    /// A single `-`. Used subtraction multiplication.
    #[token("-")]
    Minus,
    /// A single `<`. Used for less than comparisons.
    #[token("<")]
    Lt,
    /// A `<=`. Used for less than or equal to comparisons.
    #[token("<=")]
    Le,
    /// A single `>`. Used for greater than comparisons,
    #[token(">")]
    Gt,
    /// A `>=`. Used for greater than or equal to comparisons.
    #[token(">=")]
    Ge,
    /// A double equals sign: `==`. Used for equality checks.
    #[token("==")]
    Eq,
    /// A `!=`. Used for not-equality checks.
    #[token("!=")]
    Ne,
    /// A single `=`. Used for assignment and dict initialization.
    #[token("=")]
    Equal,
    /// A single `+=`. Expanded to `$1 = $1 + $2`.
    #[token("+=")]
    PlusEqual,
    /// A single `-=`. Expanded to `$1 = $1 - $2`.
    #[token("-=")]
    MinusEqual,
    /// A single `*=`. Expanded to `$1 = $1 * $2`.
    #[token("*=")]
    StarEqual,
    /// A single `**=`. Expanded to `$1 = $1 ** $2`.
    #[token("**=")]
    PowEqual,
    /// A single `/=`. Expanded to `$1 = $1 / $2`.
    #[token("/=")]
    SlashEqual,
    /// A single `\=`. Expanded to `$1 = $1 \ $2`.
    #[token("\\=")]
    BackSlashEqual,
    /// A single `%=`. Expanded to `$1 = $1 % $2`.
    #[token("%=")]
    PercentEqual,
    /// A C-style comment.
    #[regex(r"//[^\n]*")]
    Comment,
    /// A C-style multiline comment.
    #[regex(r"/\*", multi_line_comment)]
    MultilineComment,
    /// A single End of Line character.
    #[regex("\n")]
    EOL,
    /// A sequence of two or more multiline characters.
    #[regex("\n[\n]+", |lex| lex.slice().len())]
    EOLSeq(usize),
    /// A sequence of one or more whitespace characters. Skipped by the lexer.
    #[regex(r"[ \r\t\f]+", logos::skip)]
    Whitespace,
    /// Used to indicate an error.
    #[error]
    Error,
    /// Used to indicate the end of the file.
    EOF,
}

fn multi_line_comment<'a>(lex: &mut logos::Lexer<'a, TokenKind<'a>>) -> bool {
    let mut n = 0;
    let mut prev_star = false;
    let mut closed = false;

    for ch in lex.remainder().bytes() {
        n += 1;
        if ch == b'*' {
            prev_star = true;
        } else if ch == b'/' && prev_star {
            closed = true;
            break;
        } else {
            prev_star = false;
        }
    }

    if closed {
        lex.bump(n - 1);
        true
    } else {
        false
    }
}

/// A fragment of an interpolated string.
#[derive(Clone)]
pub enum Frag<'a> {
    /// A string fragment.
    Str(Token<'a>),
    /// An executable interpolated fragment.
    Sublexer(logos::Lexer<'a, TokenKind<'a>>),
}

impl<'a> std::fmt::Debug for Frag<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Str(tok) =>
                    if f.alternate() {
                        format!("{:#?}", tok)
                    } else {
                        format!("{:?}", tok)
                    },
                Self::Sublexer(_) => "Frag::Lexer(*)".to_owned(),
            }
        )
    }
}

impl<'a> PartialEq for Frag<'a> {
    fn eq(&self, other: &Frag<'a>) -> bool {
        match (self, other) {
            (Self::Str(l), Self::Str(r)) => l.lexeme == r.lexeme,
            _ => false,
        }
    }
}

fn interpolated_string<'a>(
    lex: &mut logos::Lexer<'a, TokenKind<'a>>,
) -> Result<Vec<Frag<'a>>, String> {
    let global_start = lex.span().start;
    let source = lex.source();

    let mut frags = vec![];

    let bytes = lex.slice().bytes().collect::<Vec<_>>();

    // Skip the `f` and the opening quote.
    let mut i = 2;
    let mut prev_fragment_end = 2;

    while i < bytes.len() - 1 {
        if bytes[i] == b'{' {
            // An unescaped fragment
            if bytes.get(i - 1) != Some(&b'\\') {
                let span = Span {
                    start: global_start + prev_fragment_end,
                    end: global_start + i,
                };
                frags.push(Frag::Str(Token::new(
                    &source[span.clone()],
                    0,
                    span,
                    TokenKind::StringLiteral,
                )));

                let frag_start = i + 1;
                while i < bytes.len() && bytes[i] != b'}' {
                    i += 1;
                }
                let span = Span {
                    start: global_start + frag_start,
                    end: global_start + i,
                };

                frags.push(Frag::Sublexer(TokenKind::lexer(&source[span])));

                prev_fragment_end = i + 1;
            } else {
                let span = Span {
                    start: global_start + prev_fragment_end,
                    end: global_start + i - 1, // exclude the escape
                };

                frags.push(Frag::Str(Token::new(
                    &source[span.clone()],
                    0,
                    span,
                    TokenKind::StringLiteral,
                )));
                prev_fragment_end = i;
            }
        }
        i += 1;
    }

    if prev_fragment_end < bytes.len() - 1 {
        let span = Span {
            start: global_start + prev_fragment_end,
            end: global_start + bytes.len() - 1,
        };
        frags.push(Frag::Str(Token::new(
            &source[span.clone()],
            0,
            span,
            TokenKind::StringLiteral,
        )))
    }

    Ok(frags)
}

fn scan_lua_block<'a>(lex: &mut logos::Lexer<'a, TokenKind<'a>>) -> Result<&'a str, String> {
    let start = lex.span().end;
    let mut n = 0;

    let mut n_opened = 1;

    for ch in lex.remainder().bytes() {
        n += 1;
        if ch == b'{' {
            n_opened += 1;
        } else if ch == b'}' {
            n_opened -= 1;
            if n_opened == 0 {
                break;
            }
        }
    }

    if n_opened == 0 {
        lex.bump(n);
        Ok(&lex.source()[start..start + n - 1])
    } else {
        Err(String::from("Unterminated lua block"))
    }
}

#[cfg(test)]
mod tests {}
