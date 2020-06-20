use logos::{Logos, Span};

pub type Number = f64;

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub span: Span,
    pub line: usize,
    pub kind: TokenKind<'a>,
}

impl<'a> Token<'a> {
    pub fn new(lexeme: &'a str, line: usize, span: Span, kind: TokenKind<'a>) -> Self {
        Self {
            lexeme,
            line,
            span,
            kind,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Logos)]
pub enum TokenKind<'a> {
    #[token(".")]
    Dot,

    #[token("..")]
    DoubleDot,

    #[token("...")]
    Ellipsis,

    #[token("@")]
    Variadics,

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("=>")]
    FatArrow,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex("range")]
    Range,

    #[regex("len")]
    Len,

    #[token("fn")]
    Fn,

    #[token("?")]
    Query,

    #[token("let")]
    Let,

    #[token("global")]
    Global,

    #[token("break")]
    Break,

    // XXX: should we implement continue using gotos?
    // #[token("continue")]
    // Continue,
    //
    #[token("and")]
    And,

    #[token("or")]
    Or,

    #[token("in")]
    In,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("fori")]
    ForI,

    #[token("return")]
    Return,

    #[regex(r"lua[\s\f]+\{", scan_lua_block)]
    Lua(&'a str),

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("nil")]
    Nil,

    #[regex("[0-9]+(\\.[0-9]+)?")]
    Number,

    #[regex("f\"([^\"\\\\]|\\\\.)*\"", interpolated_string)]
    #[regex("f'([^'\\\\]|\\\\.)*'", interpolated_string)]
    InterpolatedString(Vec<Frag<'a>>),

    #[regex("\"([^\"\\\\]|\\\\.)*\"")]
    #[regex("'([^'\\\\]|\\\\.)*'")]
    StringLiteral,

    #[token("not")]
    Not,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("\\")]
    BackSlash,

    #[token("%")]
    Percent,

    #[token("**")]
    Pow,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("<")]
    Lt,

    #[token("<=")]
    Le,

    #[token(">")]
    Gt,

    #[token(">=")]
    Ge,

    #[token("==")]
    Eq,

    #[token("!=")]
    Ne,

    #[token("=")]
    Equal,

    #[token("+=")]
    PlusEqual,

    #[token("-=")]
    MinusEqual,

    #[token("*=")]
    StarEqual,

    #[token("**=")]
    PowEqual,

    #[token("/=")]
    SlashEqual,

    #[token("\\=")]
    BackSlashEqual,

    #[token("%=")]
    PercentEqual,

    #[regex(r"//[^\n]*")]
    Comment,

    #[regex(r"/\*", multi_line_comment)]
    MultilineComment,

    #[regex("\n")]
    EOL,

    #[regex("\n[\n]+", |lex| lex.slice().len())]
    EOLSeq(usize),

    #[regex(r"[ \t\f]+", logos::skip)]
    Whitespace,

    #[error]
    Error,

    FragStart,
    FragEnd,
    EOF,
}

fn multi_line_comment<'a>(lex: &mut logos::Lexer<'a, TokenKind<'a>>) -> bool {
    let mut n = 0;
    let mut prev_star = false;
    let mut closed = false;

    for ch in lex.remainder().chars() {
        n += 1;
        if ch == '*' {
            prev_star = true;
        } else if ch == '/' && prev_star {
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

#[derive(Clone)]
pub enum Frag<'a> {
    Str(Token<'a>),
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

    let chars = lex.slice().chars().collect::<Vec<_>>();

    // Skip the `f` and the opening quote.
    let mut i = 2;
    let mut prev_fragment_end = 2;

    while i < chars.len() - 1 {
        if chars[i] == '{' {
            // An unescaped fragment
            if chars.get(i - 1) != Some(&'\\') {
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
                while i < chars.len() && chars[i] != '}' {
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

    if prev_fragment_end < chars.len() - 1 {
        let span = Span {
            start: global_start + prev_fragment_end,
            end: global_start + chars.len() - 1,
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

    for ch in lex.remainder().chars() {
        n += 1;
        if ch == '{' {
            n_opened += 1;
        } else if ch == '}' {
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
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        //         let source = r#"
        // print hello;
        // lua {
        //     for i = 1, 10 then
        //         print(tostring(i) .. "test")
        //     end
        // }
        // let _ = end;"#;
        //         let source = r#""f abc";
        // f"";
        // f" ";
        // f"one";
        // f" two";
        // f"three ";
        // f"\{ escaped }";
        // f"{}";
        // f"{a}";
        // f" {b}";
        // f"{c} ";
        // f"{d}{e}";
        // f"{d * f(35)} {e + 1}";"#;
        let source = r#"
                // We can disable the commands that require an API that is not available.
                /*
                A multi-line comment
                */"#;
        // /*
        // let a =  -1;
        // let b = 2 + 2;
        // let c = 3 * 3;
        // let d = 4 / 4;
        // let e = 4 // 4;
        // let f = 5 ** 5;
        // let g = 6 % 7;
        // // let h = (7 << 8) & ~(9 >> 10) | 3;
        // let i = true != false and 3 < 4 or 5 >= 6 or 3 <= 7 and 10 > 2;

        // let arr = [1, 2, 3];
        // let dict = {1: 2, 3: 4};

        // print(len(arr));
        // print(len(dict));
        //"#;

        let mut lexer = TokenKind::lexer(source);
        while let Some(token) = lexer.next() {
            println!("--- NEW TOKEN ---");
            println!("{:#?}", token);
            println!("`{}`", lexer.slice());
        }
        assert!(false);
        // assert!(false, "{:#?}", tokens);
    }
}
