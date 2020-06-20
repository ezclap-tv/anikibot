//! Contains the implementations of the PPGA Lexer and Parser
//! as well as the AST definitions.
pub mod ast;
pub mod lexer;
pub mod parser;

use logos::Logos;
pub use parser::Parser;

/// Creates a new lexer from the given source code.
#[inline(always)]
pub fn lexer<'a>(source: &'a str) -> parser::Lexer<'a> {
    lexer::TokenKind::lexer(source)
}
