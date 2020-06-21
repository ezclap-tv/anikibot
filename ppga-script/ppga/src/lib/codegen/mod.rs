//! This module provides functions for transpiling `PPGA` ASTs to `Lua`.
//!
//! Usage example:
//!
//! ```
//! extern crate ppga;
//! use ppga::{frontend::{Parser, lexer}, codegen::emit_lua, config::PPGAConfig};
//!
//! let ast = Parser::with_config(
//!     PPGAConfig::default().disable_std(),
//!     lexer("fn f() { return 5; }")
//! ).parse().unwrap();
//! let output = emit_lua(&ast);
//! assert_eq!(output,
//! r#"local function f()
//!     return (5)
//! end"#);
//! ```
pub mod code_builder;
pub mod codegen;
pub mod snippets;

pub use codegen::{emit_lua, expr_to_lua, fn_to_lua, stmt_to_lua};
