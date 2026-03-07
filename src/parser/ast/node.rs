//! AST node infrastructure with arena allocation
//!
//! This module provides AstNode and NodeBuilder for creating and managing
//! AST nodes using bumpalo arena allocation for efficient memory usage.

use bumpalo::Bump;
use super::types::NodeKind;
use super::Span;

/// AST node with arena lifetime
///
/// An AstNode represents a single node in the TypeScript AST, with a
/// specific kind, optional span, and child nodes. Nodes are
/// allocated in a bumpalo arena and have the arena's lifetime.
#[derive(Debug)]
pub struct AstNode<'a> {
    /// The kind/type of this AST node
    kind: NodeKind,
    /// Source code span for error reporting (optional for release builds)
    #[cfg(feature = "spans")]
    span: Option<Span>,
    /// Child nodes stored as references to the arena
    children: Vec<&'a AstNode<'a>>,
}

impl<'a> AstNode<'a> {
    /// Create a new AST node
    #[inline]
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            #[cfg(feature = "spans")]
            span: None,
            children: Vec::new(),
        }
    }

    /// Get the kind of this node
    #[inline]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Get mutable reference to kind
    #[inline]
    pub fn kind_mut(&mut self) -> &mut NodeKind {
        &mut self.kind
    }

    /// Get the span of this node
    #[cfg(feature = "spans")]
    #[inline]
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    /// Set the span of this node
    #[cfg(feature = "spans")]
    #[inline]
    pub fn set_span(&mut self, span: Span) {
        self.span = Some(span);
    }

    /// Get the children of this node
    #[inline]
    pub fn children(&self) -> &[&'a AstNode<'a>] {
        &self.children
    }

    /// Add a child to this node
    #[inline]
    pub fn add_child(&mut self, child: &'a AstNode<'a>) {
        self.children.push(child);
    }

    /// Add multiple children to this node
    #[inline]
    pub fn extend_children(&mut self, children: Vec<&'a AstNode<'a>>) {
        self.children.extend(children);
    }
}

/// Node builder for arena-allocated AST nodes
///
/// The NodeBuilder provides a convenient interface for constructing
/// AST nodes in a bumpalo arena. It handles allocation and
/// ensures proper lifetime management.
pub struct NodeBuilder<'a> {
    arena: &'a Bump,
}

impl<'a> NodeBuilder<'a> {
    /// Create a new node builder for the given arena
    #[inline]
    pub fn new(arena: &'a Bump) -> Self {
        Self { arena }
    }

    /// Allocate a new AST node in the arena
    pub fn alloc(&self, kind: NodeKind) -> &'a AstNode<'a> {
        self.arena.alloc(AstNode::new(kind))
    }

    /// Allocate a new AST node with a span
    #[cfg(feature = "spans")]
    pub fn alloc_with_span(&self, kind: NodeKind, span: Span) -> &'a AstNode<'a> {
        let node = self.arena.alloc(AstNode::new(kind));
        // This is safe because we have mutable access to the node in arena
        // and it won't be shared until after construction
        unsafe {
            let ptr = node as *const AstNode<'a> as *mut AstNode<'a>;
            (*ptr).set_span(span);
        }
        node
    }

    /// Allocate a new AST node with children
    pub fn alloc_with_children(
        &self,
        kind: NodeKind,
        children: Vec<&'a AstNode<'a>>,
    ) -> &'a AstNode<'a> {
        let node = self.arena.alloc(AstNode::new(kind));
        unsafe {
            let ptr = node as *const AstNode<'a> as *mut AstNode<'a>;
            (*ptr).extend_children(children);
        }
        node
    }

    /// Allocate a new AST node with span and children
    #[cfg(feature = "spans")]
    pub fn alloc_complete(
        &self,
        kind: NodeKind,
        span: Span,
        children: Vec<&'a AstNode<'a>>,
    ) -> &'a AstNode<'a> {
        let node = self.arena.alloc(AstNode::new(kind));
        unsafe {
            let ptr = node as *const AstNode<'a> as *mut AstNode<'a>;
            (*ptr).set_span(span);
            (*ptr).extend_children(children);
        }
        node
    }

    /// Get a reference to the arena
    #[inline]
    pub fn arena(&self) -> &'a Bump {
        self.arena
    }
}

/// AST arena containing all nodes
///
/// The AstArena manages a bumpalo arena and stores all AST nodes
/// for a parsed source file. This provides efficient allocation and
/// deallocation (all nodes are freed when the arena is dropped).
pub struct AstArena<'a> {
    arena: Bump,
    root: Option<&'a AstNode<'a>>,
}

impl<'a> AstArena<'a> {
    /// Create a new AST arena
    pub fn new() -> Self {
        Self {
            arena: Bump::new(),
            root: None,
        }
    }

    /// Create a node builder for this arena
    pub fn builder(&self) -> NodeBuilder<'_> {
        NodeBuilder::new(&self.arena)
    }

    /// Set the root node of this arena
    pub fn set_root(&mut self, root: &'a AstNode<'a>) {
        self.root = Some(root);
    }

    /// Get the root node of this arena
    pub fn root(&self) -> Option<&'a AstNode<'a>> {
        self.root
    }

    /// Get the arena reference
    pub fn arena(&self) -> &Bump {
        &self.arena
    }

    /// Count total nodes allocated in this arena
    ///
    /// This is an approximation based on allocated bytes, as
    /// bumpalo doesn't track object counts directly.
    pub fn allocated_bytes(&self) -> usize {
        self.arena.allocated_bytes()
    }
}

impl Default for AstArena<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::types::{
        NodeKind, BinaryOperator, Literal,
    };

    #[test]
    fn test_ast_node_creation() {
        let kind = NodeKind::Identifier {
            name: "test".to_string(),
        };
        let node = AstNode::new(kind);
        assert!(matches!(node.kind(), NodeKind::Identifier { .. }));
    }

    #[test]
    fn test_ast_node_children() {
        let arena = Bump::new();
        let builder = NodeBuilder::new(&arena);

        let _child1 = builder.alloc(NodeKind::Literal(Literal::String("hello".to_string())));
        let _child2 = builder.alloc(NodeKind::Literal(Literal::Number(42.0)));
        let parent = builder.alloc(NodeKind::Identifier { name: "parent".to_string() });

        assert_eq!(parent.children().len(), 0);
    }

    #[test]
    fn test_node_builder() {
        let arena = Bump::new();
        let builder = NodeBuilder::new(&arena);

        let node = builder.alloc(NodeKind::Identifier {
            name: "test".to_string(),
        });

        assert!(matches!(node.kind(), NodeKind::Identifier { .. }));
    }

    #[test]
    fn test_ast_arena() {
        let arena = AstArena::new();
        let _builder = arena.builder();

        // Verify arena was created successfully
        assert!(arena.root().is_none());
    }
    #[test]
    fn test_node_allocation_in_arena() {
        let arena = Bump::new();
        let builder = NodeBuilder::new(&arena);

        // Allocate multiple nodes
        for i in 0..100 {
            builder.alloc(NodeKind::Literal(Literal::Number(i as f64)));
        }

        // Nodes are allocated in the arena
        assert!(arena.allocated_bytes() > 0);
    }

    #[test]
    fn test_complex_node_structure() {
        let arena = Bump::new();
        let builder = NodeBuilder::new(&arena);

        // Build a binary expression: a + b
        let left = builder.alloc(NodeKind::Identifier { name: "a".to_string() });
        let right = builder.alloc(NodeKind::Identifier { name: "b".to_string() });
        let _add_node = builder.alloc_with_children(
            NodeKind::Binary {
                operator: BinaryOperator::Add,
                left: NodeId::new(0), // Placeholder ID
                right: NodeId::new(1), // Placeholder ID
            },
            vec![left, right],
        );
    }

    #[test]
    fn test_arena_default() {
        let arena = AstArena::default();
        assert!(arena.root().is_none());
    }
}
