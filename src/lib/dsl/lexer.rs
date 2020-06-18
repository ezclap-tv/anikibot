use logos::{Logos, Span};

pub type Number = f64;

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub span: Span,
    pub kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn new(lexeme: &'a str, span: Span, kind: TokenKind) -> Self {
        Self { lexeme, span, kind }
    }
}

#[derive(Debug, PartialEq, Clone, Logos)]
pub enum TokenKind {
    #[token(".")]
    Dot,

    #[token("...")]
    Ellipsis,

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

    #[token("fn")]
    Fn,

    #[token("opt")]
    Opt,

    #[token("let")]
    Let,

    #[token("global")]
    Global,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

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

    #[token("lua")]
    Lua,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("nil")]
    Nil,

    #[regex("[0-9]+(\\.[0-9]+)?", |lex| lex.slice().parse())]
    Number(Number),

    #[regex("\"([^\"\\\\]|\\\\.)*\"")]
    #[regex("'([^'\\\\]|\\\\.)*'")]
    StringLiteral,

    #[regex("f\"([^\"\\\\]|\\\\.)*\"")]
    #[regex("f'([^'\\\\]|\\\\.)*'")]
    InterpolatedString,

    #[token("ether")]
    #[token("!")]
    Not,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

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

    #[token("%=")]
    PercentEqual,

    #[regex(r"//[^\n]*")]
    Comment,

    #[regex(r"/\*", multi_line_comment)]
    MultilineComment,

    #[regex(r"\n")]
    NewLine,

    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,

    EOF,
}

fn multi_line_comment(lex: &mut logos::Lexer<TokenKind>) -> bool {
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
        lex.bump(n);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = r#"
        // We can disable the commands that require an API that is not available.
        /* 
        A multi-line comment
        */
/*
let a =  -1;
let b = 2 + 2;
let c = 3 * 3;
let d = 4 / 4;
let e = 4 // 4;
let f = 5 ** 5;
let g = 6 % 7;
// let h = (7 << 8) & ~(9 >> 10) | 3;
let i = true != false and 3 < 4 or 5 >= 6 or 3 <= 7 and 10 > 2;

let arr = [1, 2, 3];
let dict = {1: 2, 3: 4};

print(len(arr));
print(len(dict));"#;

        let mut lexer = TokenKind::lexer(source);
        while let Some(token) = lexer.next() {
            println!("{:#?}", token);
            println!("{:#?}", lexer.slice());
        }
        assert!(false);
        // assert!(false, "{:#?}", tokens);
    }
}
