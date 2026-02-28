//! Transmute - TypeScript to Rust Semantic Transpiler
//!
//! This library provides a high-performance TypeScript to Rust code transformation
//! tool that automatically derives ownership, lifetimes, and other Rust semantics
//! from TypeScript source code.

#![warn(missing_docs)]
#![warn(clippy::all)]

// Parser module - Wave 1
pub mod parser;

// Re-export key types for convenient access
pub use parser::{error::ParseError, parse_source, parse_file};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
