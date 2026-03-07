//! Scope data structure implementation.
//!
//! Defines the core Scope type and ScopeTable for managing nested lexical scopes.

use crate::parser::ast::Span;
use rustc_hash::FxHashMap;
use std::fmt;

/// Unique identifier for a scope.
///
/// Scopes are identified by a unique integer ID that is assigned when
/// the scope is created. This allows for O(1) scope lookups and comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(u32);

impl ScopeId {
    /// Create a new ScopeId from a u32.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the underlying u32 value of the ScopeId.
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scope({})", self.0)
    }
}

/// The kind of scope, determining its behavior and semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    /// Module-level scope (top-level of a file)
    Module,
    /// Function body scope
    Function,
    /// Block scope (curly braces)
    Block,
    /// Loop body scope (for, while, do-while)
    Loop,
    /// Catch clause scope
    Catch,
    /// Class body scope
    Class,
}

impl fmt::Display for ScopeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScopeKind::Module => write!(f, "module"),
            ScopeKind::Function => write!(f, "function"),
            ScopeKind::Block => write!(f, "block"),
            ScopeKind::Loop => write!(f, "loop"),
            ScopeKind::Catch => write!(f, "catch"),
            ScopeKind::Class => write!(f, "class"),
        }
    }
}

/// A lexical scope containing symbol declarations.
///
/// Each scope represents a region of code where identifiers are declared and
/// looked up. Scopes form a hierarchy via parent pointers, allowing for
/// lexical scoping rules.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Unique identifier for this scope
    id: ScopeId,
    /// The kind of scope
    kind: ScopeKind,
    /// Parent scope, if any (None for root module scope)
    parent: Option<ScopeId>,
    /// Source span where this scope starts and ends
    span: Span,
    /// Symbols declared in this scope, mapping name to symbol ID
    symbols: FxHashMap<String, crate::semantic::symbol::SymbolId>,
}

impl Scope {
    /// Create a new Scope with the given ID, kind, parent, and span.
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>, span: Span) -> Self {
        Self {
            id,
            kind,
            parent,
            span,
            symbols: FxHashMap::default(),
        }
    }

    /// Get the unique ID of this scope.
    #[inline]
    pub fn id(&self) -> ScopeId {
        self.id
    }

    /// Get the kind of this scope.
    #[inline]
    pub fn kind(&self) -> ScopeKind {
        self.kind
    }

    /// Get the parent scope ID, if any.
    #[inline]
    pub fn parent(&self) -> Option<ScopeId> {
        self.parent
    }

    /// Get the source span of this scope.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get a reference to the symbols map for this scope.
    #[inline]
    pub fn symbols(&self) -> &FxHashMap<String, crate::semantic::symbol::SymbolId> {
        &self.symbols
    }

    /// Get a mutable reference to the symbols map for this scope.
    #[inline]
    pub fn symbols_mut(&mut self) -> &mut FxHashMap<String, crate::semantic::symbol::SymbolId> {
        &mut self.symbols
    }

    /// Add a symbol to this scope.
    ///
    /// Returns the previous symbol ID if the name was already declared.
    pub fn add_symbol(&mut self, name: String, symbol_id: crate::semantic::symbol::SymbolId) -> Option<crate::semantic::symbol::SymbolId> {
        self.symbols.insert(name, symbol_id)
    }

    /// Look up a symbol in this scope by name.
    pub fn get_symbol(&self, name: &str) -> Option<crate::semantic::symbol::SymbolId> {
        self.symbols.get(name).copied()
    }

    /// Check if this scope contains a symbol with the given name.
    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}

/// A table managing all scopes in a program.
///
/// The ScopeTable maintains the hierarchy of scopes and provides methods
/// for creating, navigating, and looking up symbols in scopes.
#[derive(Debug, Clone)]
pub struct ScopeTable {
    /// All scopes in the program, indexed by ScopeId
    scopes: Vec<Scope>,
    /// The root module scope
    root: ScopeId,
    /// The currently active scope (top of the scope stack)
    current: ScopeId,
}

impl ScopeTable {
    /// Create a new ScopeTable with a root module scope.
    pub fn new(root_span: Span) -> Self {
        let root_id = ScopeId::new(0);
        let root_scope = Scope::new(root_id, ScopeKind::Module, None, root_span);

        Self {
            scopes: vec![root_scope],
            root: root_id,
            current: root_id,
        }
    }

    /// Get the root scope ID.
    #[inline]
    pub fn root(&self) -> ScopeId {
        self.root
    }

    /// Get the current scope ID.
    #[inline]
    pub fn current(&self) -> ScopeId {
        self.current
    }

    /// Get a reference to the current scope.
    #[inline]
    pub fn current_scope(&self) -> &Scope {
        self.get_scope(self.current).expect("Current scope should exist")
    }

    /// Get a mutable reference to the current scope.
    #[inline]
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.get_scope_mut(self.current).expect("Current scope should exist")
    }

    /// Get a reference to a scope by ID.
    #[inline]
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.get() as usize)
    }

    /// Get a mutable reference to a scope by ID.
    #[inline]
    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.get() as usize)
    }

    /// Create a new scope and make it the current scope.
    ///
    /// The new scope will have the current scope as its parent.
    pub fn push_scope(&mut self, kind: ScopeKind, span: Span) -> ScopeId {
        let new_id = ScopeId::new(self.scopes.len() as u32);
        let new_scope = Scope::new(new_id, kind, Some(self.current), span);
        self.scopes.push(new_scope);
        self.current = new_id;
        new_id
    }

    /// Pop the current scope, returning to its parent.
    ///
    /// Returns the ID of the scope that was popped.
    ///
    /// # Panics
    ///
    /// Panics if attempting to pop the root scope.
    pub fn pop_scope(&mut self) -> ScopeId {
        let popped = self.current;
        let parent = self.get_scope(popped)
            .and_then(|s| s.parent())
            .expect("Cannot pop root scope");

        self.current = parent;
        popped
    }

    /// Create a new scope without changing the current scope.
    ///
    /// The new scope will have the specified parent.
    pub fn create_scope(&mut self, kind: ScopeKind, parent: ScopeId, span: Span) -> ScopeId {
        let new_id = ScopeId::new(self.scopes.len() as u32);
        let new_scope = Scope::new(new_id, kind, Some(parent), span);
        self.scopes.push(new_scope);
        new_id
    }

    /// Look up a symbol by name, starting from the current scope and traversing up the parent chain.
    ///
    /// Returns the symbol ID and the scope where it was found, or None if not found.
    pub fn lookup_symbol(&self, name: &str) -> Option<(crate::semantic::symbol::SymbolId, ScopeId)> {
        let mut current_id = self.current;

        loop {
            let scope = self.get_scope(current_id)?;
            if let Some(symbol_id) = scope.get_symbol(name) {
                return Some((symbol_id, current_id));
            }

            match scope.parent() {
                Some(parent_id) => current_id = parent_id,
                None => return None,
            }
        }
    }

    /// Look up a symbol by name in a specific scope and its parent chain.
    ///
    /// Returns the symbol ID and the scope where it was found, or None if not found.
    pub fn lookup_symbol_in(&self, start_scope: ScopeId, name: &str) -> Option<(crate::semantic::symbol::SymbolId, ScopeId)> {
        let mut current_id = start_scope;

        loop {
            let scope = self.get_scope(current_id)?;
            if let Some(symbol_id) = scope.get_symbol(name) {
                return Some((symbol_id, current_id));
            }

            match scope.parent() {
                Some(parent_id) => current_id = parent_id,
                None => return None,
            }
        }
    }

    /// Get all scopes in the table.
    #[inline]
    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }

    /// Get the number of scopes in the table.
    #[inline]
    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }
}