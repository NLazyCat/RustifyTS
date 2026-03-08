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
pub mod ir;
pub mod analyzer;

use crate::parser::ast::{AstNode, Span};
use analyzer::SemanticAnalyzer;
use bumpalo::Bump;

/// Analyze the given AST and produce a complete semantic module.
///
/// This is the main entry point for semantic analysis. It runs all
/// analysis passes in sequence and returns a complete SemanticModule
/// containing scope, symbol, type, and CFG information.
///
/// # Arguments
///
/// * `ast` - The AST to analyze
/// * `arena` - The bump allocator used for the AST
///
/// # Returns
///
/// * `Ok(SemanticModule)` - The analyzed semantic module
/// * `Err(SemanticError)` - An error occurred during analysis
pub fn analyze(
    ast: &AstNode,
    arena: &Bump,
) -> Result<ir::module::SemanticModule, analyzer::SemanticError> {
    let mut analyzer = SemanticAnalyzer::new(arena);
    analyzer.analyze(ast)
}