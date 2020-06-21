#![feature(box_patterns)]
extern crate logos;

#[macro_use]
mod macros;
pub mod codegen;
pub mod config;
pub mod errors;
pub mod frontend;

use codegen::emit_lua;
use errors::ErrCtx;
use frontend::{lexer, Parser};

pub use codegen::code_builder::DEFAULT_INDENT_SIZE;
pub use config::PPGAConfig;

pub fn ppga_to_lua<'a>(source: &'a str, config: PPGAConfig) -> Result<String, ErrCtx<'a>> {
    Parser::with_config(config, lexer(source.trim_end()))
        .parse()
        .map(|ast| emit_lua(&ast))
}
