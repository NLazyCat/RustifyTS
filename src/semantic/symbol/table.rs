//! Symbol table implementation.
//!
//! Manages the storage and lookup of symbols across the entire program.

use super::{Symbol, SymbolId, SymbolKind};
use crate::semantic::scope::ScopeId;
use crate::semantic::types::TypeId;
use crate::parser::ast::Span;
use rustc_hash::FxHashMap;

/// A symbol table that stores and manages all symbols in a program.
///
/// The symbol table provides efficient lookup of symbols by name across scopes,
/// supporting lexical scoping rules and proper shadowing behavior.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// All symbols in the program, indexed by SymbolId
    symbols: Vec<Symbol>,
    /// Map from (scope ID, name) to symbol ID for fast lookup
    by_name: FxHashMap<(ScopeId, String), SymbolId>,
    /// Next available symbol ID
    next_id: u32,
}

impl SymbolTable {
    /// Create a new empty SymbolTable.
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            by_name: FxHashMap::default(),
            next_id: 0,
        }
    }

    /// Insert a new symbol into the table.
    ///
    /// Creates a new symbol with the given name, kind, span, scope, and optional type ID.
    /// Returns the ID of the newly created symbol.
    ///
    /// If a symbol with the same name already exists in the same scope, it will be
    /// replaced and the old symbol ID will be returned.
    pub fn insert(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: Span,
        scope: ScopeId,
        type_id: Option<TypeId>,
    ) -> SymbolId {
        let id = SymbolId::new(self.next_id);
        self.next_id += 1;

        let mut symbol = Symbol::new(id, name.clone(), kind, span, scope);
        if let Some(ty) = type_id {
            symbol.set_type_id(ty);
        }

        let key = (scope, name);
        let _old_id = self.by_name.insert(key, id);
        self.symbols.push(symbol);

        id
    }

    /// Look up a symbol by name in the current scope and all parent scopes.
    ///
    /// This performs a lexical lookup, starting from the given scope and traversing
    /// up the parent chain until the symbol is found or the root scope is reached.
    ///
    /// Returns the symbol ID if found, None otherwise.
    pub fn lookup_lexical(
        &self,
        name: &str,
        start_scope: ScopeId,
        scope_table: &crate::semantic::scope::ScopeTable,
    ) -> Option<SymbolId> {
        let mut current_scope = start_scope;

        loop {
            if let Some(&id) = self.by_name.get(&(current_scope, name.to_string())) {
                return Some(id);
            }

            match scope_table.get_scope(current_scope)?.parent() {
                Some(parent) => current_scope = parent,
                None => return None,
            }
        }
    }

    /// Look up a symbol by name in a specific scope only.
    ///
    /// Does not traverse parent scopes. Returns the symbol ID if found in the
    /// specified scope, None otherwise.
    pub fn lookup_in_scope(&self, name: &str, scope: ScopeId) -> Option<SymbolId> {
        self.by_name.get(&(scope, name.to_string())).copied()
    }

    /// Look up a symbol by its ID.
    ///
    /// Returns a reference to the symbol if the ID is valid, None otherwise.
    pub fn lookup(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.get() as usize)
    }

    /// Get a mutable reference to a symbol by its ID.
    ///
    /// Returns a mutable reference to the symbol if the ID is valid, None otherwise.
    pub fn lookup_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(id.get() as usize)
    }

    /// Get all symbols in the table.
    #[inline]
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Get the number of symbols in the table.
    #[inline]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::scope::{Scope, ScopeKind, ScopeTable};
    use crate::parser::ast::Span;

    fn test_span() -> Span {
        Span::new(0, 0)
    }

    #[test]
    fn test_symbol_table_insert_and_lookup() {
        let mut table = SymbolTable::new();
        let scope = ScopeId::new(0);

        // Insert a symbol
        let id = table.insert(
            "x".to_string(),
            SymbolKind::Variable,
            test_span(),
            scope,
            None,
        );

        // Lookup by ID
        let symbol = table.lookup(id).unwrap();
        assert_eq!(symbol.name(), "x");
        assert_eq!(symbol.kind(), SymbolKind::Variable);
        assert_eq!(symbol.scope(), scope);

        // Lookup in scope
        let lookup_id = table.lookup_in_scope("x", scope).unwrap();
        assert_eq!(lookup_id, id);
    }

    #[test]
    fn test_symbol_shadowing() {
        let mut table = SymbolTable::new();
        let scope1 = ScopeId::new(0);
        let scope2 = ScopeId::new(1);

        // Insert x in scope 1
        let id1 = table.insert(
            "x".to_string(),
            SymbolKind::Variable,
            test_span(),
            scope1,
            None,
        );

        // Insert x in scope 2 (shadowing)
        let id2 = table.insert(
            "x".to_string(),
            SymbolKind::Variable,
            test_span(),
            scope2,
            None,
        );

        assert_ne!(id1, id2);
        assert_eq!(table.lookup_in_scope("x", scope1), Some(id1));
        assert_eq!(table.lookup_in_scope("x", scope2), Some(id2));
    }

    #[test]
    fn test_lexical_lookup() {
        let mut scope_table = ScopeTable::new(test_span());
        let root_scope = scope_table.root();

        // Create nested scope
        let child_scope = scope_table.push_scope(ScopeKind::Block, test_span());

        let mut symbol_table = SymbolTable::new();

        // Insert x in root scope
        let root_x = symbol_table.insert(
            "x".to_string(),
            SymbolKind::Variable,
            test_span(),
            root_scope,
            None,
        );

        // Insert y in child scope
        let child_y = symbol_table.insert(
            "y".to_string(),
            SymbolKind::Variable,
            test_span(),
            child_scope,
            None,
        );

        // Lookup x from child scope (should find in root)
        let found_x = symbol_table.lookup_lexical("x", child_scope, &scope_table);
        assert_eq!(found_x, Some(root_x));

        // Lookup y from child scope (should find in child)
        let found_y = symbol_table.lookup_lexical("y", child_scope, &scope_table);
        assert_eq!(found_y, Some(child_y));

        // Lookup y from root scope (should not find)
        let not_found = symbol_table.lookup_lexical("y", root_scope, &scope_table);
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_type_id_association() {
        let mut table = SymbolTable::new();
        let scope = ScopeId::new(0);
        let type_id = TypeId::new(1);

        let id = table.insert(
            "x".to_string(),
            SymbolKind::Variable,
            test_span(),
            scope,
            Some(type_id),
        );

        let symbol = table.lookup(id).unwrap();
        assert_eq!(symbol.type_id(), Some(type_id));
    }
}