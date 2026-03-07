//! Symbol table module.
//!
//! Manages symbol information including declarations, types,
//! and metadata for all identifiers in the program.

pub mod symbol;
pub mod table;

pub use symbol::*;
pub use table::*;

#[cfg(test)]
mod tests;