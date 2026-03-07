//! Parser backend implementations
//!
//! This module contains different parser backend implementations that
//! implement the ParserBackend trait. Currently supported backends:
//! - Deno: Uses Deno subprocess with TypeScript Compiler API

pub mod r#trait;
pub mod deno;

// Re-export key types
pub use r#trait::ParserBackend;
pub use deno::{DenoBackend, DenoBackendConfig};
