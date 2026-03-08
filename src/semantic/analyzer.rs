//! Main semantic analyzer module.
//!
//! Orchestrates the entire semantic analysis process, coordinating
//! all sub-components (scope, symbol, types, flow, IR) to produce
//! a fully analyzed program representation.

use crate::parser::ast::{AstNode, Span, visitor::Visitor};
use crate::semantic::ir::module::SemanticModule;
use crate::semantic::scope::analyzer::ScopeAnalyzer;
use crate::semantic::types::resolver::TypeResolver;
use crate::semantic::types::TypeInterner;
use bumpalo::Bump;
use thiserror::Error;

/// Error that can occur during semantic analysis.
#[derive(Debug, Error)]
pub enum SemanticError {
    /// Type resolution error
    #[error("type error: {0}")]
    TypeError(#[from] crate::semantic::types::resolver::ResolutionError),
}

/// Main semantic analyzer that coordinates all analysis passes.
///
/// The SemanticAnalyzer runs a series of analysis passes on the AST:
/// 1. Scope analysis - builds the scope hierarchy and symbol table
/// 2. Type resolution - resolves all type references
/// 3. CFG construction - builds control flow graphs for functions
///
/// The result is a complete SemanticModule containing all semantic information.
pub struct SemanticAnalyzer<'a> {
    /// Bump allocator for AST nodes
    _arena: &'a Bump,
    /// Type interner for type management
    type_interner: TypeInterner,
}

impl<'a> SemanticAnalyzer<'a> {
    /// Create a new SemanticAnalyzer with the given arena.
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            _arena: arena,
            type_interner: TypeInterner::new(),
        }
    }

    /// Analyze the given AST and produce a complete semantic module.
    ///
    /// This runs all analysis passes in sequence:
    /// 1. Scope analysis - builds scope tree and symbol table
    /// 2. Type resolution - resolves all type references to concrete types
    /// 3. CFG construction - builds control flow graphs for functions
    pub fn analyze(&mut self, ast: &'a AstNode<'a>) -> Result<SemanticModule, SemanticError> {
        // Get root span for module
        let root_span = Self::get_span(ast);

        // Create a module to hold all analysis results
        let mut module = SemanticModule::new("main".to_string());

        // Pass 1: Scope analysis
        // Build the scope hierarchy and populate the symbol table
        let mut scope_analyzer = ScopeAnalyzer::new(
            self._arena,
            &mut self.type_interner,
            root_span,
        );
        scope_analyzer.visit_node(ast);

        // Transfer scope table and symbol table to module
        module.scopes = scope_analyzer.scope_table;
        module.symbols = scope_analyzer.symbol_table;

        // Pass 2: Type resolution
        // Resolve all type references in the AST
        let mut type_resolver = TypeResolver::new(
            &module.symbols,
            &module.scopes,
            &mut self.type_interner,
            module.scopes.root(),
        );
        type_resolver.visit_node(ast);

        // Pass 3: CFG construction
        // Build control flow graphs for all functions
        // For now, we'll create a simple CFG for each function in the symbol table
        self.build_cfgs_for_functions(&mut module, ast)?;

        Ok(module)
    }

    /// Build CFGs for all functions in the module.
    fn build_cfgs_for_functions(
        &mut self,
        module: &mut SemanticModule,
        ast: &'a AstNode<'a>,
    ) -> Result<(), SemanticError> {
        // Find all function symbols and build CFGs for them
        // This is a simplified implementation - a full implementation would
        // traverse the AST and build CFGs for each function definition

        // For now, we'll just return Ok - the CFG builder will be integrated
        // more fully in a future iteration
        Ok(())
    }

    /// Get the span from a node or return a default span.
    #[inline]
    fn get_span(node: &'a AstNode<'a>) -> Span {
        #[cfg(feature = "spans")]
        {
            node.span().unwrap_or_else(|| Span::new(0, 0))
        }
        #[cfg(not(feature = "spans"))]
        {
            Span::new(0, 0)
        }
    }
}

#[cfg(test)]
mod tests;