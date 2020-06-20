//! This module defines the [`PPGAConfig`] struct used to configure the transpiler.
//!
//! [`PPGAConfig`]: crate::config::PPGAConfig
use crate::codegen::code_builder::DEFAULT_INDENT_SIZE;

/// The config used by the transpiler.
#[derive(Debug, Clone)]
pub struct PPGAConfig {
    /// If `true`, the transpiler will emit the comments
    /// from the original `.ppga` file into the resulting `.lua` file.
    pub emit_comments: bool,
    /// The number of indentation spaces in the resulting lua code.
    pub indent_size: usize,
    /// Specifies whether to include PPGAs standard library symbols.
    /// They are required for some of PPGAs features to work (for example, the `??` and `?` operators).
    pub include_ppga_std: bool,
}

impl Default for PPGAConfig {
    /// Creates a [`PPGAConfig`] with an ident size of [`DEFAULT_INDENT_SIZE`], comments turned off,
    /// and standard symbols included.
    ///
    /// [`PPGAConfig`]: crate::config::PPGAConfig
    /// [`DEFAULT_INDENT_SIZE`]: crate::codegen::code_builder::DEFAULT_INDENT_SIZE
    fn default() -> Self {
        Self {
            emit_comments: false,
            indent_size: DEFAULT_INDENT_SIZE,
            include_ppga_std: true,
        }
    }
}

impl PPGAConfig {
    /// Sets `include_ppga_std` to `false`.
    pub fn disable_std(self) -> Self {
        Self {
            include_ppga_std: false,
            ..self
        }
    }
}
