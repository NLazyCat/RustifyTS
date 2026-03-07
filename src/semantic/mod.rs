//! Semantic analysis module for TypeScript to Rust compilation.
//!
//! This module handles all semantic analysis phases including:
//! - Scope analysis and symbol table management
//! - Type checking and inference
//! - Control flow graph analysis
//! - Intermediate representation (IR) generation
//! - Full program analysis and validation

pub mod scope;
pub mod symbol;
pub mod types;
pub mod flow;
pub mod analyzer;