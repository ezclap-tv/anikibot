extern crate logos;

#[macro_use]
mod macros;
pub mod ast;
pub mod code_builder;
pub mod codegen;
pub mod errors;
pub mod lexer;
pub mod parser;

use logos::Logos;

use codegen::emit_lua;
use errors::ErrCtx;
use lexer::TokenKind;
use parser::Parser;

pub fn ppga_to_lua<'a>(source: &'a str, emit_comments: bool) -> Result<String, ErrCtx<'a>> {
    Parser::new(TokenKind::lexer(source))
        .emit_comments(emit_comments)
        .parse()
        .map(|ast| emit_lua(&ast))
}
