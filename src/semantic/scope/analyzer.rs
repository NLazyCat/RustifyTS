//! Scope analyzer implementation.
//!
//! Implements the scope analysis logic that traverses the AST and builds the scope tree.

use super::{ScopeId, ScopeKind, ScopeTable};
use crate::parser::ast::visitor::Visitor;
use crate::parser::ast::{AstNode, NodeKind, Span};
use crate::parser::ast::types::VariableKind;
use crate::semantic::symbol::{SymbolId, SymbolKind, SymbolTable};
use crate::semantic::types::{TypeId, TypeInterner};
use bumpalo::Bump;

/// Scope analyzer visitor that builds the scope hierarchy and populates the symbol table.
///
/// This visitor traverses the AST and creates appropriate scopes for blocks, functions,
/// classes, loops, and catch clauses. It also processes declarations and adds symbols
/// to the symbol table according to JavaScript/TypeScript scoping rules.
#[derive(Debug)]
pub struct ScopeAnalyzer<'a> {
    /// Scope table managing all scopes
    pub scope_table: ScopeTable,
    /// Symbol table storing all declared symbols
    pub symbol_table: SymbolTable,
    /// Bump allocator for AST nodes
    _arena: &'a Bump,
    /// Type interner for type lookups and creation
    _type_interner: &'a mut TypeInterner,
}

impl<'a> ScopeAnalyzer<'a> {
    /// Create a new ScopeAnalyzer with the given root span.
    pub fn new(
        arena: &'a Bump,
        type_interner: &'a mut TypeInterner,
        root_span: Span,
    ) -> Self {
        Self {
            scope_table: ScopeTable::new(root_span),
            symbol_table: SymbolTable::new(),
            _arena: arena,
            _type_interner: type_interner,
        }
    }

    /// Get the span from a node or return a default span.
    #[inline]
    fn get_span(&self, node: &'a AstNode<'a>) -> Span {
        #[cfg(feature = "spans")]
        {
            node.span().unwrap_or_else(|| Span::new(0, 0))
        }
        #[cfg(not(feature = "spans"))]
        {
            Span::new(0, 0)
        }
    }

    /// Get the current scope ID.
    #[inline]
    pub fn current_scope(&self) -> ScopeId {
        self.scope_table.current()
    }

    /// Push a new scope onto the scope stack.
    #[inline]
    pub fn push_scope(&mut self, kind: ScopeKind, span: Span) -> ScopeId {
        self.scope_table.push_scope(kind, span)
    }

    /// Pop the current scope from the scope stack.
    #[inline]
    pub fn pop_scope(&mut self) -> ScopeId {
        self.scope_table.pop_scope()
    }

    /// Declare a variable in the current scope.
    ///
    /// Handles var hoisting and let/const block scoping according to ES6 semantics.
    pub fn declare_variable(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: Span,
        is_var: bool,
        type_id: Option<TypeId>,
    ) -> SymbolId {
        // For var declarations, we need to hoist to the nearest function or module scope
        let declare_scope = if is_var {
            self.find_hoist_scope()
        } else {
            self.current_scope()
        };

        self.symbol_table.insert(
            name,
            kind,
            span,
            declare_scope,
            type_id,
        )
    }

    /// Find the nearest scope where var declarations should be hoisted to.
    ///
    /// Vars are hoisted to the nearest function scope or the root module scope.
    fn find_hoist_scope(&self) -> ScopeId {
        let mut current = self.current_scope();

        loop {
            let scope = self.scope_table.get_scope(current)
                .expect("Scope should exist");

            match scope.kind() {
                ScopeKind::Function | ScopeKind::Module => return current,
                _ => {
                    current = scope.parent()
                        .expect("Scope should have parent");
                }
            }
        }
    }

    /// Process a function declaration, adding it to the current scope and creating a function scope.
    ///
    /// Function declarations are hoisted to the top of their containing scope.
    pub fn process_function_declaration(
        &mut self,
        name: String,
        span: Span,
        type_id: Option<TypeId>,
    ) -> (SymbolId, ScopeId) {
        // Add function symbol to current scope (hoisted)
        let symbol_id = self.symbol_table.insert(
            name,
            SymbolKind::Function,
            span,
            self.current_scope(),
            type_id,
        );

        // Create function scope
        let function_scope = self.push_scope(ScopeKind::Function, span);

        (symbol_id, function_scope)
    }

    /// Process a class declaration, adding it to the current scope and creating a class scope.
    pub fn process_class_declaration(
        &mut self,
        name: String,
        span: Span,
        type_id: Option<TypeId>,
    ) -> (SymbolId, ScopeId) {
        // Add class symbol to current scope
        let symbol_id = self.symbol_table.insert(
            name,
            SymbolKind::Class,
            span,
            self.current_scope(),
            type_id,
        );

        // Create class scope
        let class_scope = self.push_scope(ScopeKind::Class, span);

        (symbol_id, class_scope)
    }
}

impl<'a> Visitor<'a> for ScopeAnalyzer<'a> {
    /// Visit a block statement, creating a new block scope.
    fn visit_block(&mut self, node: &'a AstNode<'a>) {
        // Create block scope
        let _block_scope = self.push_scope(ScopeKind::Block, self.get_span(node));

        // Visit children in the new scope
        self.default_visit_node(node);

        // Pop back to parent scope
        self.pop_scope();
    }

    /// Visit a function declaration, creating a function scope and hoisting the function symbol.
    fn visit_function_declaration(&mut self, node: &'a AstNode<'a>) {
        if let NodeKind::FunctionDeclaration { name, params: _, return_type: _, body: _ } = node.kind() {
            // TODO: Extract type information from function signature
            let type_id = None;

            // Process function declaration (adds symbol to current scope and creates function scope)
            let (_, _function_scope) = self.process_function_declaration(
                name.clone(),
                self.get_span(node),
                type_id,
            );

            // TODO: Add function parameters to function scope

            // Visit function body
            self.default_visit_node(node);

            // Pop function scope
            self.pop_scope();
        } else {
            // Fallback to default behavior if node kind doesn't match
            self.default_visit_node(node);
        }
    }

    /// Visit a variable statement, handling var hoisting and let/const block scoping.
    fn visit_variable_statement(&mut self, node: &'a AstNode<'a>) {
        if let NodeKind::VariableStatement { declarations } = node.kind() {
            for decl in declarations {
                let is_var = decl.kind == VariableKind::Var;
                // All variables use Variable kind for now - constness will be tracked in symbol flags
                let kind = SymbolKind::Variable;

                // TODO: Extract type information from type annotation
                let type_id = None;

                self.declare_variable(
                    decl.name.clone(),
                    kind,
                    self.get_span(node), // Use statement span for now
                    is_var,
                    type_id,
                );
            }
        }

        // Visit children (initializers)
        self.default_visit_node(node);
    }

    /// Visit a for statement, creating a loop scope.
    fn visit_for(&mut self, node: &'a AstNode<'a>) {
        // Create loop scope
        let _loop_scope = self.push_scope(ScopeKind::Loop, self.get_span(node));

        // Visit init, condition, update, and body in loop scope
        self.default_visit_node(node);

        // Pop loop scope
        self.pop_scope();
    }

    /// Visit a for-of statement, creating a loop scope.
    fn visit_for_of(&mut self, node: &'a AstNode<'a>) {
        // Create loop scope
        let _loop_scope = self.push_scope(ScopeKind::Loop, self.get_span(node));

        // Visit left, right, and body in loop scope
        self.default_visit_node(node);

        // Pop loop scope
        self.pop_scope();
    }

    /// Visit a while statement, creating a loop scope.
    fn visit_while(&mut self, node: &'a AstNode<'a>) {
        // Create loop scope
        let _loop_scope = self.push_scope(ScopeKind::Loop, self.get_span(node));

        // Visit condition and body in loop scope
        self.default_visit_node(node);

        // Pop loop scope
        self.pop_scope();
    }

    /// Visit a do-while statement, creating a loop scope.
    fn visit_do_while(&mut self, node: &'a AstNode<'a>) {
        // Create loop scope
        let _loop_scope = self.push_scope(ScopeKind::Loop, self.get_span(node));

        // Visit body and condition in loop scope
        self.default_visit_node(node);

        // Pop loop scope
        self.pop_scope();
    }

    /// Visit a try statement, creating catch scope if needed.
    fn visit_try(&mut self, node: &'a AstNode<'a>) {
        if let NodeKind::Try { try_block: _, catch_clause, finally_block: _ } = node.kind() {
            // Visit try block (first child is try block)
            if let Some(&try_block_node) = node.children().first() {
                self.visit_node(try_block_node);
            }

            // Handle catch clause if present
            if let Some(_catch) = catch_clause {
                // Create catch scope
                let _catch_scope = self.push_scope(ScopeKind::Catch, self.get_span(node));

                // TODO: Add the exception parameter to catch scope if present
                // if let Some(var_id) = catch.variable {
                //     // Get the variable node from children and add to catch scope
                // }

                // Visit catch body (second child is catch body)
                if node.children().len() >= 2 {
                    self.visit_node(node.children()[1]);
                }

                // Pop catch scope
                self.pop_scope();
            }

            // Visit finally block in current scope (not implemented yet)
        } else {
            self.default_visit_node(node);
        }
    }

    /// Visit a class declaration, creating a class scope.
    fn visit_class_declaration(&mut self, node: &'a AstNode<'a>) {
        if let NodeKind::ClassDeclaration { name, extends: _, members: _ } = node.kind() {
            // TODO: Extract type information from class
            let type_id = None;

            // Process class declaration (adds symbol to current scope and creates class scope)
            let (_, _class_scope) = self.process_class_declaration(
                name.clone(),
                self.get_span(node),
                type_id,
            );

            // Visit class body
            self.default_visit_node(node);

            // Pop class scope
            self.pop_scope();
        } else {
            // Fallback to default behavior if node kind doesn't match
            self.default_visit_node(node);
        }
    }

    /// Visit an arrow function, creating a function scope.
    fn visit_arrow_function(&mut self, node: &'a AstNode<'a>) {
        // Create function scope for arrow function
        let _function_scope = self.push_scope(ScopeKind::Function, self.get_span(node));

        // TODO: Add parameters to function scope

        // Visit function body
        self.default_visit_node(node);

        // Pop function scope
        self.pop_scope();
    }

    /// Visit a function expression, creating a function scope.
    fn visit_function_expression(&mut self, node: &'a AstNode<'a>) {
        // Create function scope for function expression
        let _function_scope = self.push_scope(ScopeKind::Function, self.get_span(node));

        // TODO: Add parameters to function scope
        // TODO: Handle named function expressions (add name to function scope)

        // Visit function body
        self.default_visit_node(node);

        // Pop function scope
        self.pop_scope();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bumpalo::Bump;
    use crate::parser::ast::{NodeBuilder, Span};
    use crate::parser::ast::types::{VariableDeclaration, VariableKind, NodeId};
    use crate::semantic::types::TypeInterner;

    fn test_span() -> Span {
        Span::new(0, 100)
    }

    #[test]
    fn test_block_scope() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: { let x = 1; }
        let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
        let var_stmt = builder.alloc(NodeKind::VariableStatement {
            declarations: vec![VariableDeclaration {
                name: "x".to_string(),
                kind: VariableKind::Let,
                initializer: Some(NodeId::new(0)), // lit_1 is first child
                type_annotation: None,
            }],
        });
        let block = builder.alloc_with_children(
            NodeKind::Block {
                statements: vec![NodeId::new(0)] // var_stmt is first child
            },
            vec![var_stmt, lit_1],
        );

        // Visit the block
        analyzer.visit_node(block);

        // Should have 2 scopes: root + block
        assert_eq!(analyzer.scope_table.scope_count(), 2);

        // x should be in the block scope, not root
        let root_scope = analyzer.scope_table.root();
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let block_scope = scopes[1].id();

        assert!(analyzer.symbol_table.lookup_in_scope("x", root_scope).is_none());
        assert!(analyzer.symbol_table.lookup_in_scope("x", block_scope).is_some());
    }

    #[test]
    fn test_var_hoisting() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: { var x = 1; }
        let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
        let var_stmt = builder.alloc(NodeKind::VariableStatement {
            declarations: vec![VariableDeclaration {
                name: "x".to_string(),
                kind: VariableKind::Var,
                initializer: Some(NodeId::new(0)), // lit_1 is first child
                type_annotation: None,
            }],
        });
        let block = builder.alloc_with_children(
            NodeKind::Block {
                statements: vec![NodeId::new(0)] // var_stmt is first child
            },
            vec![var_stmt, lit_1],
        );

        // Visit the block
        analyzer.visit_node(block);

        // var x should be hoisted to root scope
        let root_scope = analyzer.scope_table.root();

        assert!(analyzer.symbol_table.lookup_in_scope("x", root_scope).is_some());
    }

    #[test]
    fn test_function_scope() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: function f() { let x = 1; }
        let f_name = builder.alloc(NodeKind::Identifier { name: "f".to_string() });
        let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
        let var_stmt = builder.alloc(NodeKind::VariableStatement {
            declarations: vec![VariableDeclaration {
                name: "x".to_string(),
                kind: VariableKind::Let,
                initializer: Some(NodeId::new(0)), // lit_1 is first child after var_stmt
                type_annotation: None,
            }],
        });
        let body = builder.alloc_with_children(
            NodeKind::Block {
                statements: vec![NodeId::new(0)] // var_stmt is first child of block
            },
            vec![var_stmt, lit_1],
        );
        let func_decl = builder.alloc_with_children(
            NodeKind::FunctionDeclaration {
                name: "f".to_string(),
                params: vec![],
                return_type: None,
                body: NodeId::new(1), // body is second child (index 1)
            },
            vec![f_name, body],
        );

        // Visit the function
        analyzer.visit_node(func_decl);

        // Should have 3 scopes: root + function + block
        assert_eq!(analyzer.scope_table.scope_count(), 3);

        // Function f should be in root scope
        let root_scope = analyzer.scope_table.root();
        assert!(analyzer.symbol_table.lookup_in_scope("f", root_scope).is_some());

        // x should be in function's block scope, not root
        assert!(analyzer.symbol_table.lookup_in_scope("x", root_scope).is_none());
    }

    #[test]
    fn test_nested_scopes() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST:
        // let x = 1;
        // {
        //   let x = 2;
        //   {
        //     let y = 3;
        //   }
        // }

        // Literals
        let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
        let lit_2 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(2.0)));
        let lit_3 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(3.0)));

        // Innermost block: { let y = 3; }
        let stmt3 = builder.alloc(NodeKind::VariableStatement {
            declarations: vec![VariableDeclaration {
                name: "y".to_string(),
                kind: VariableKind::Let,
                initializer: Some(NodeId::new(0)), // lit_3 is first child after stmt3
                type_annotation: None,
            }],
        });
        let inner_block = builder.alloc_with_children(
            NodeKind::Block { statements: vec![NodeId::new(0)] }, // stmt3 is first child
            vec![stmt3, lit_3],
        );

        // Outer block: { let x = 2; { let y = 3; } }
        let stmt2 = builder.alloc(NodeKind::VariableStatement {
            declarations: vec![VariableDeclaration {
                name: "x".to_string(),
                kind: VariableKind::Let,
                initializer: Some(NodeId::new(0)), // lit_2 is first child after stmt2
                type_annotation: None,
            }],
        });
        let outer_block = builder.alloc_with_children(
            NodeKind::Block { statements: vec![NodeId::new(0), NodeId::new(1)] }, // stmt2, inner_block
            vec![stmt2, lit_2, inner_block],
        );

        // Root block
        let stmt1 = builder.alloc(NodeKind::VariableStatement {
            declarations: vec![VariableDeclaration {
                name: "x".to_string(),
                kind: VariableKind::Let,
                initializer: Some(NodeId::new(0)), // lit_1 is first child after stmt1
                type_annotation: None,
            }],
        });
        let root_block = builder.alloc_with_children(
            NodeKind::Block { statements: vec![NodeId::new(0), NodeId::new(1)] }, // stmt1, outer_block
            vec![stmt1, lit_1, outer_block],
        );

        // Visit the AST
        analyzer.visit_node(root_block);

        // Should have 4 scopes: root + root_block + outer_block + inner_block
        assert_eq!(analyzer.scope_table.scope_count(), 4);

        let root_scope = analyzer.scope_table.root();
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let root_block_scope = scopes[1].id();
        let outer_block_scope = scopes[2].id();
        let inner_block_scope = scopes[3].id();

        // Check x in root block scope
        assert!(analyzer.symbol_table.lookup_in_scope("x", root_block_scope).is_some());

        // Check x shadowing in outer block
        assert!(analyzer.symbol_table.lookup_in_scope("x", outer_block_scope).is_some());

        // Check y in inner block only
        assert!(analyzer.symbol_table.lookup_in_scope("y", inner_block_scope).is_some());
        assert!(analyzer.symbol_table.lookup_in_scope("y", outer_block_scope).is_none());
        assert!(analyzer.symbol_table.lookup_in_scope("y", root_block_scope).is_none());
        assert!(analyzer.symbol_table.lookup_in_scope("y", root_scope).is_none());
    }
}