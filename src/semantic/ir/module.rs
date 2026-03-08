//! IR module representation.
//!
//! Defines the top-level module structure for the IR.

use crate::parser::ast::Span;
use crate::semantic::ir::Function;
use crate::semantic::scope::ScopeTable;
use crate::semantic::symbol::SymbolTable;
use crate::semantic::types::TypeInterner;

/// Represents a top-level module in the IR.
#[derive(Debug)]
pub struct SemanticModule {
    /// Name of the module
    pub name: String,
    /// Functions defined in this module
    pub functions: Vec<Function>,
    /// Type interner for all types used in this module
    pub types: TypeInterner,
    /// Symbol table for all symbols defined in this module
    pub symbols: SymbolTable,
    /// Scope table for all scopes in this module
    pub scopes: ScopeTable,
}

impl SemanticModule {
    /// Create a new empty module with the given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: Vec::new(),
            types: TypeInterner::new(),
            symbols: SymbolTable::new(),
            scopes: ScopeTable::new(Span::empty()),
        }
    }

    /// Add a function to this module.
    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    /// Get a function by its symbol ID.
    pub fn get_function(&self, id: crate::semantic::symbol::SymbolId) -> Option<&Function> {
        self.functions.iter().find(|f| f.id == id)
    }

    /// Get a mutable reference to a function by its symbol ID.
    pub fn get_function_mut(&mut self, id: crate::semantic::symbol::SymbolId) -> Option<&mut Function> {
        self.functions.iter_mut().find(|f| f.id == id)
    }
}
