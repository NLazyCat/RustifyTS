//! AST (Abstract Syntax Tree) module
//!
//! This module provides types for representing TypeScript source code
//! as an abstract syntax tree, with support for arena allocation,
//! span tracking, and visitor pattern traversal.

pub mod node;
pub mod span;
pub mod types;

// Re-export key types for convenience
pub use node::{AstArena, AstNode, NodeBuilder};
pub use span::{LineMap, Span};
pub use types::{NodeKind, NodeId};
