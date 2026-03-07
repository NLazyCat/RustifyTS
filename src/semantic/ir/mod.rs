//! Intermediate Representation (IR) module.
//!
//! Defines the semantic IR that represents the program after analysis,
//! before code generation. This IR is typed and contains all semantic
//! information needed for Rust code generation.

pub mod module;
pub mod function;
pub mod instruction;

pub use module::*;
pub use function::*;
pub use instruction::*;

#[cfg(test)]
mod tests;