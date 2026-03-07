//! Scope analysis module.
//!
//! Handles lexical scope management, variable declaration tracking,
//! and identifier resolution.

pub mod scope;
pub mod analyzer;

pub use scope::*;
pub use analyzer::*;

#[cfg(test)]
mod tests;