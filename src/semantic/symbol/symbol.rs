//! Symbol data structure implementation.
//!
//! Defines the core Symbol type representing program identifiers and their metadata.

use crate::parser::ast::Span;
use crate::semantic::scope::ScopeId;
use crate::semantic::types::TypeId;
use std::fmt;

/// Unique identifier for a symbol.
///
/// Symbols are identified by a unique integer ID that is assigned when
/// the symbol is created. This allows for O(1) symbol lookups and comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(u32);

impl SymbolId {
    /// Create a new SymbolId from a u32.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the underlying u32 value of the SymbolId.
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.0)
    }
}

/// The kind of symbol, determining its behavior and semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// Function declaration or expression
    Function,
    /// Variable declaration (let, const, var)
    Variable,
    /// Class declaration
    Class,
    /// Interface declaration
    Interface,
    /// Type alias declaration
    TypeAlias,
    /// Enum declaration
    Enum,
    /// Imported symbol (from import statements)
    Import,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Interface => write!(f, "interface"),
            SymbolKind::TypeAlias => write!(f, "type alias"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::Import => write!(f, "import"),
        }
    }
}

/// A symbol representing a declared identifier in the program.
///
/// Each symbol contains metadata about an identifier including its name,
/// type, scope, location, and other relevant information.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Unique identifier for this symbol
    id: SymbolId,
    /// The name of the symbol (identifier)
    name: String,
    /// The kind of symbol
    kind: SymbolKind,
    /// Source span where this symbol was declared
    span: Span,
    /// The scope where this symbol was declared
    scope: ScopeId,
    /// Whether this symbol is exported from its module
    is_export: bool,
    /// The type of this symbol, if known
    type_id: Option<TypeId>,
}

impl Symbol {
    /// Create a new Symbol with the given ID, name, kind, span, and scope.
    pub fn new(
        id: SymbolId,
        name: String,
        kind: SymbolKind,
        span: Span,
        scope: ScopeId,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            span,
            scope,
            is_export: false,
            type_id: None,
        }
    }

    /// Get the unique ID of this symbol.
    #[inline]
    pub fn id(&self) -> SymbolId {
        self.id
    }

    /// Get the name of this symbol.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the kind of this symbol.
    #[inline]
    pub fn kind(&self) -> SymbolKind {
        self.kind
    }

    /// Get the source span of this symbol's declaration.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get the scope ID where this symbol was declared.
    #[inline]
    pub fn scope(&self) -> ScopeId {
        self.scope
    }

    /// Check if this symbol is exported from its module.
    #[inline]
    pub fn is_export(&self) -> bool {
        self.is_export
    }

    /// Set whether this symbol is exported.
    #[inline]
    pub fn set_export(&mut self, is_export: bool) {
        self.is_export = is_export;
    }

    /// Get the type ID of this symbol, if known.
    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        self.type_id
    }

    /// Set the type ID of this symbol.
    #[inline]
    pub fn set_type_id(&mut self, type_id: TypeId) {
        self.type_id = Some(type_id);
    }

    /// Clear the type ID of this symbol.
    #[inline]
    pub fn clear_type_id(&mut self) {
        self.type_id = None;
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} '{}' [{}] at {}",
            self.kind,
            self.name,
            self.id,
            self.span
        )
    }
}