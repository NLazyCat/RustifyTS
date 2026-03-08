//! Scope analyzer implementation.
//!
//! Implements the scope analysis logic that traverses the AST and builds the scope tree.

use super::{ScopeId, ScopeKind, ScopeTable};
use crate::parser::ast::visitor::Visitor;
use crate::parser::ast::{AstNode, NodeKind, Span};
use crate::parser::ast::types::{VariableKind, TypeAnnotation};
use crate::semantic::symbol::{SymbolId, SymbolKind, SymbolTable};
use crate::semantic::types::{TypeId, TypeInterner, Type, PrimitiveType};
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

/// Information about a function parameter extracted from the AST.
#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    type_annotation: Option<crate::parser::ast::types::TypeAnnotation>,
    span: Span,
    has_default: bool,
    is_rest: bool,
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

    /// Extract parameter information from a function node.
    ///
    /// Handles all function types: FunctionDeclaration, FunctionExpression, ArrowFunction.
    /// Returns a vector of parameter information including name, type, span, and flags.
    fn extract_parameters(&self, node: &'a AstNode<'a>) -> Vec<ParameterInfo> {
        match node.kind() {
            NodeKind::FunctionDeclaration { params, .. } |
            NodeKind::FunctionExpression { params, .. } |
            NodeKind::ArrowFunction { params, .. } => {
                params.iter()
                    .map(|param| ParameterInfo {
                        name: param.name.clone(),
                        type_annotation: param.type_annotation.clone(),
                        span: Span::new(0, 0), // Will be filled in by caller
                        has_default: param.default_value.is_some(),
                        is_rest: param.is_rest,
                    })
                    .collect()
            }
            _ => vec![],
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

    /// Convert a TypeAnnotation to a Type and intern it.
    fn intern_type_annotation(&mut self, annotation: &Option<TypeAnnotation>) -> Option<TypeId> {
        annotation.as_ref().map(|ann| {
            let ty = self.type_annotation_to_type(ann);
            self._type_interner.intern(ty)
        })
    }

    /// Convert a TypeAnnotation to a Type.
    fn type_annotation_to_type(&self, annotation: &TypeAnnotation) -> Type {
        match annotation {
            TypeAnnotation::TypeReference { name, type_params } => {
                // For primitive types, convert directly
                let is_primitive = match name.as_str() {
                    "string" | "number" | "boolean" | "void" | "any" | "unknown" |
                    "null" | "undefined" | "never" => true,
                    _ => false,
                };

                if is_primitive {
                    let primitive_type = match name.as_str() {
                        "string" => PrimitiveType::String,
                        "number" => PrimitiveType::Number,
                        "boolean" => PrimitiveType::Boolean,
                        "void" => PrimitiveType::Void,
                        "any" => PrimitiveType::Any,
                        "unknown" => PrimitiveType::Unknown,
                        "null" => PrimitiveType::Null,
                        "undefined" => PrimitiveType::Undefined,
                        "never" => PrimitiveType::Never,
                        _ => unreachable!(),
                    };
                    Type::Primitive(primitive_type)
                } else {
                    // For non-primitive types, create a reference
                    // This will be resolved by the TypeResolver later
                    if let Some(params) = type_params {
                        let param_types: Vec<_> = params.iter()
                            .map(|p| self.type_annotation_to_type(p))
                            .collect();
                        Type::Reference {
                            name: name.clone(),
                            type_args: param_types,
                        }
                    } else {
                        Type::Reference {
                            name: name.clone(),
                            type_args: vec![],
                        }
                    }
                }
            }
            TypeAnnotation::ArrayType(elem) => {
                Type::Array(Box::new(self.type_annotation_to_type(elem)))
            }
            TypeAnnotation::UnionType(types) => {
                let union_types: Vec<_> = types.iter()
                    .map(|t| self.type_annotation_to_type(t))
                    .collect();
                Type::Union(union_types)
            }
            TypeAnnotation::FunctionType { params, return_type } => {
                let param_types: Vec<_> = params.iter()
                    .map(|p| self.type_annotation_to_type(
                        &p.type_annotation.clone().unwrap_or(TypeAnnotation::Unknown)
                    ))
                    .collect();
                let return_ty = self.type_annotation_to_type(return_type);
                Type::Function {
                    params: param_types,
                    return_type: Box::new(return_ty),
                    type_params: vec![],
                }
            }
            TypeAnnotation::Unknown => Type::Primitive(PrimitiveType::Unknown),
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

    /// Add function parameters to the current scope.
    ///
    /// Takes parameter information from the AST and creates symbols for each parameter
    /// in the function scope. Handles type annotations and parameter flags.
    fn add_parameters_to_scope(&mut self, params: Vec<ParameterInfo>) {
        for param in params {
            // Intern the type annotation if present
            let type_id = self.intern_type_annotation(&param.type_annotation);

            // Create a symbol for the parameter
            self.symbol_table.insert(
                param.name.clone(),
                SymbolKind::Variable, // Parameters are variables in the function scope
                param.span,
                self.current_scope(),
                type_id,
            );
        }
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
        if let NodeKind::FunctionDeclaration { name, params, return_type, body: _ } = node.kind() {
            // Build the function type from annotations
            let param_types: Vec<_> = params.iter()
                .map(|p| {
                    let ann_ty = self.intern_type_annotation(&p.type_annotation);
                    ann_ty.map(|id| self._type_interner.get(id).unwrap().clone())
                        .unwrap_or_else(|| Type::Primitive(PrimitiveType::Unknown))
                })
                .collect();

            let return_ty = self.intern_type_annotation(return_type)
                .map(|id| Box::new(self._type_interner.get(id).unwrap().clone()));

            let function_type = Type::Function {
                params: param_types,
                return_type: return_ty.unwrap_or_else(|| {
                    Box::new(Type::Primitive(PrimitiveType::Unknown))
                }),
                type_params: vec![],
            };

            let type_id = Some(self._type_interner.intern(function_type));

            // Process function declaration (adds symbol to current scope and creates function scope)
            let (_, _function_scope) = self.process_function_declaration(
                name.clone(),
                self.get_span(node),
                type_id,
            );

            // Extract parameters and add them to the function scope
            let param_infos = self.extract_parameters(node);
            self.add_parameters_to_scope(param_infos);

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

                // Extract type information from type annotation
                let type_id = self.intern_type_annotation(&decl.type_annotation);

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
            if let Some(catch) = catch_clause {
                // Create catch scope
                let _catch_scope = self.push_scope(ScopeKind::Catch, self.get_span(node));

                // Add exception parameter to catch scope if present
                if let Some(_var_node_id) = catch.variable {
                    // Look for the catch parameter in children
                    // The catch parameter is typically an Identifier node in the children
                    // before the catch body
                    for child in node.children() {
                        if let NodeKind::Identifier { name } = child.kind() {
                            // Create symbol for exception parameter with 'any' type
                            let type_id = Some(self._type_interner.intern(Type::Primitive(PrimitiveType::Any)));

                            self.symbol_table.insert(
                                name.clone(),
                                SymbolKind::Variable,
                                self.get_span(child),
                                self.current_scope(),
                                type_id,
                            );
                            break;
                        }
                    }
                }

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

        // Extract parameters and add them to the function scope
        let param_infos = self.extract_parameters(node);
        self.add_parameters_to_scope(param_infos);

        // Visit function body
        self.default_visit_node(node);

        // Pop function scope
        self.pop_scope();
    }

    /// Visit a function expression, creating a function scope.
    fn visit_function_expression(&mut self, node: &'a AstNode<'a>) {
        if let NodeKind::FunctionExpression { name, params, return_type: _, body: _ } = node.kind() {
            // Create function scope for function expression
            let _function_scope = self.push_scope(ScopeKind::Function, self.get_span(node));

            // If the function has a name, add it to the function scope
            if let Some(func_name) = name {
                self.symbol_table.insert(
                    func_name.clone(),
                    SymbolKind::Function,
                    self.get_span(node),
                    self.current_scope(),
                    None, // Type will be inferred later
                );
            }

            // Extract parameters and add them to the function scope
            let param_infos = self.extract_parameters(node);
            self.add_parameters_to_scope(param_infos);

            // Visit function body
            self.default_visit_node(node);

            // Pop function scope
            self.pop_scope();
        } else {
            self.default_visit_node(node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bumpalo::Bump;
    use crate::parser::ast::{NodeBuilder, Span};
    use crate::parser::ast::types::{VariableDeclaration, VariableKind, NodeId, CatchClause};
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

    #[test]
    fn test_catch_parameter_in_scope() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: try { throw e; } catch (err) { console.log(err); }

        // Literal: "error message"
        let lit_msg = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::String("error".to_string())));

        // Identifier: err
        let err_ident = builder.alloc(NodeKind::Identifier { name: "err".to_string() });

        // Try block: { throw "error"; }
        let try_block = builder.alloc_with_children(
            NodeKind::Block { statements: vec![] },
            vec![lit_msg],
        );

        // Catch body: { console.log(err); }
        let catch_body = builder.alloc_with_children(
            NodeKind::Block { statements: vec![] },
            vec![err_ident],
        );

        // Try statement with catch clause
        let try_stmt = builder.alloc_with_children(
            NodeKind::Try {
                try_block: NodeId::new(0),
                catch_clause: Some(CatchClause {
                    variable: Some(NodeId::new(1)), // err_ident reference
                    body: NodeId::new(1), // catch_body reference
                }),
                finally_block: None,
            },
            vec![try_block, err_ident, catch_body],
        );

        // Visit the try statement
        analyzer.visit_node(try_stmt);

        // Should have 3 scopes: root + catch
        assert_eq!(analyzer.scope_table.scope_count(), 2);

        // err should be in the catch scope
        let root_scope = analyzer.scope_table.root();
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let catch_scope = scopes[1].id();

        // err should be in catch scope, not root
        assert!(analyzer.symbol_table.lookup_in_scope("err", root_scope).is_none());
        assert!(analyzer.symbol_table.lookup_in_scope("err", catch_scope).is_some());
    }

    #[test]
    fn test_catch_without_parameter() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: try { throw e; } catch { console.log("error"); }

        // Try block: { throw "error"; }
        let try_block = builder.alloc_with_children(
            NodeKind::Block { statements: vec![] },
            vec![],
        );

        // Catch body: { console.log("error"); }
        let catch_body = builder.alloc_with_children(
            NodeKind::Block { statements: vec![] },
            vec![],
        );

        // Try statement with catch clause (no parameter)
        let try_stmt = builder.alloc_with_children(
            NodeKind::Try {
                try_block: NodeId::new(0),
                catch_clause: Some(CatchClause {
                    variable: None, // No parameter
                    body: NodeId::new(1),
                }),
                finally_block: None,
            },
            vec![try_block, catch_body],
        );

        // Visit the try statement
        analyzer.visit_node(try_stmt);

        // Should have 2 scopes: root + catch
        assert_eq!(analyzer.scope_table.scope_count(), 2);

        // No exception parameter should exist
        let root_scope = analyzer.scope_table.root();
        let catch_scope = analyzer.scope_table.scopes().iter().nth(1).unwrap().id();

        // Catch scope exists but has no exception parameter
        assert!(analyzer.symbol_table.lookup_in_scope("err", root_scope).is_none());
        assert!(analyzer.symbol_table.lookup_in_scope("err", catch_scope).is_none());
    }

    #[test]
    fn test_function_declaration_parameters() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: function add(x: number, y: number) { return x + y; }
        let body_block = builder.alloc(NodeKind::Block { statements: vec![] });
        let func_decl = builder.alloc_with_children(
            NodeKind::FunctionDeclaration {
                name: "add".to_string(),
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "x".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                    crate::parser::ast::types::Parameter {
                        name: "y".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                ],
                return_type: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                    name: "number".to_string(),
                    type_params: None,
                }),
                body: NodeId::new(0),
            },
            vec![body_block],
        );

        // Visit the function
        analyzer.visit_node(func_decl);

        // Should have 2 scopes: root + function
        assert_eq!(analyzer.scope_table.scope_count(), 2);

        // Function 'add' should be in root scope
        let root_scope = analyzer.scope_table.root();
        assert!(analyzer.symbol_table.lookup_in_scope("add", root_scope).is_some());

        // Parameters 'x' and 'y' should be in function scope
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        let x_symbol = analyzer.symbol_table.lookup_in_scope("x", function_scope);
        assert!(x_symbol.is_some(), "Parameter 'x' should exist in function scope");

        let y_symbol = analyzer.symbol_table.lookup_in_scope("y", function_scope);
        assert!(y_symbol.is_some(), "Parameter 'y' should exist in function scope");

        // Check that parameters are not in root scope
        assert!(analyzer.symbol_table.lookup_in_scope("x", root_scope).is_none());
        assert!(analyzer.symbol_table.lookup_in_scope("y", root_scope).is_none());
    }

    #[test]
    fn test_arrow_function_parameters() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: const add = (x: number, y: number) => x + y;
        let body_expr = builder.alloc(NodeKind::Identifier { name: "x".to_string() });
        let arrow_func = builder.alloc_with_children(
            NodeKind::ArrowFunction {
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "x".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                    crate::parser::ast::types::Parameter {
                        name: "y".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                ],
                return_type: None,
                body: NodeId::new(0),
            },
            vec![body_expr],
        );

        // Visit the arrow function
        analyzer.visit_node(arrow_func);

        // Should have 2 scopes: root + function
        assert_eq!(analyzer.scope_table.scope_count(), 2);

        // Parameters 'x' and 'y' should be in function scope
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        let x_symbol = analyzer.symbol_table.lookup_in_scope("x", function_scope);
        assert!(x_symbol.is_some(), "Parameter 'x' should exist in arrow function scope");

        let y_symbol = analyzer.symbol_table.lookup_in_scope("y", function_scope);
        assert!(y_symbol.is_some(), "Parameter 'y' should exist in arrow function scope");
    }

    #[test]
    fn test_function_expression_parameters() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: const add = function(x: number, y: number) { return x + y; };
        let body_block = builder.alloc(NodeKind::Block { statements: vec![] });
        let func_expr = builder.alloc_with_children(
            NodeKind::FunctionExpression {
                name: None,
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "x".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                    crate::parser::ast::types::Parameter {
                        name: "y".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                ],
                return_type: None,
                body: NodeId::new(0),
            },
            vec![body_block],
        );

        // Visit the function expression
        analyzer.visit_node(func_expr);

        // Should have 2 scopes: root + function
        assert_eq!(analyzer.scope_table.scope_count(), 2);

        // Parameters 'x' and 'y' should be in function scope
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        let x_symbol = analyzer.symbol_table.lookup_in_scope("x", function_scope);
        assert!(x_symbol.is_some(), "Parameter 'x' should exist in function expression scope");

        let y_symbol = analyzer.symbol_table.lookup_in_scope("y", function_scope);
        assert!(y_symbol.is_some(), "Parameter 'y' should exist in function expression scope");
    }

    #[test]
    fn test_named_function_expression() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: const add = function add(x: number, y: number) { return x + y; };
        let body_block = builder.alloc(NodeKind::Block { statements: vec![] });
        let func_expr = builder.alloc_with_children(
            NodeKind::FunctionExpression {
                name: Some("add".to_string()),
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "x".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                ],
                return_type: None,
                body: NodeId::new(0),
            },
            vec![body_block],
        );

        // Visit the function expression
        analyzer.visit_node(func_expr);

        // Should have 2 scopes: root + function
        assert_eq!(analyzer.scope_table.scope_count(), 2);

        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        // The function name 'add' should be in the function scope
        let add_symbol = analyzer.symbol_table.lookup_in_scope("add", function_scope);
        assert!(add_symbol.is_some(), "Function name 'add' should exist in function scope");
        assert_eq!(add_symbol.unwrap().kind(), SymbolKind::Function);

        // Parameter 'x' should also be in function scope
        let x_symbol = analyzer.symbol_table.lookup_in_scope("x", function_scope);
        assert!(x_symbol.is_some(), "Parameter 'x' should exist in function scope");
    }

    #[test]
    fn test_rest_parameter() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: function sum(...args: number[]) { return args.reduce((a, b) => a + b, 0); }
        let body_block = builder.alloc(NodeKind::Block { statements: vec![] });
        let func_decl = builder.alloc_with_children(
            NodeKind::FunctionDeclaration {
                name: "sum".to_string(),
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "args".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::ArrayType(
                            Box::new(crate::parser::ast::types::TypeAnnotation::TypeReference {
                                name: "number".to_string(),
                                type_params: None,
                            })
                        )),
                        default_value: None,
                        is_rest: true,
                    },
                ],
                return_type: None,
                body: NodeId::new(0),
            },
            vec![body_block],
        );

        // Visit the function
        analyzer.visit_node(func_decl);

        // Rest parameter 'args' should be in function scope
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        let args_symbol = analyzer.symbol_table.lookup_in_scope("args", function_scope);
        assert!(args_symbol.is_some(), "Rest parameter 'args' should exist in function scope");
    }

    #[test]
    fn test_default_parameter() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: function greet(name: string = "World") { return "Hello, " + name; }
        let body_block = builder.alloc(NodeKind::Block { statements: vec![] });
        let default_value = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::String("World".to_string())));
        let func_decl = builder.alloc_with_children(
            NodeKind::FunctionDeclaration {
                name: "greet".to_string(),
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "name".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "string".to_string(),
                            type_params: None,
                        }),
                        default_value: Some(NodeId::new(0)),
                        is_rest: false,
                    },
                ],
                return_type: None,
                body: NodeId::new(1), // body_block is second child (index 1)
            },
            vec![default_value, body_block],
        );

        // Visit the function
        analyzer.visit_node(func_decl);

        // Parameter 'name' with default value should be in function scope
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        let name_symbol = analyzer.symbol_table.lookup_in_scope("name", function_scope);
        assert!(name_symbol.is_some(), "Parameter 'name' with default value should exist in function scope");
    }

    #[test]
    fn test_parameter_type_annotation() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: function foo(x: number, y: string): boolean { return true; }
        let body_block = builder.alloc(NodeKind::Block { statements: vec![] });
        let func_decl = builder.alloc_with_children(
            NodeKind::FunctionDeclaration {
                name: "foo".to_string(),
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "x".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "number".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                    crate::parser::ast::types::Parameter {
                        name: "y".to_string(),
                        type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                            name: "string".to_string(),
                            type_params: None,
                        }),
                        default_value: None,
                        is_rest: false,
                    },
                ],
                return_type: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                    name: "boolean".to_string(),
                    type_params: None,
                }),
                body: NodeId::new(0),
            },
            vec![body_block],
        );

        // Visit the function
        analyzer.visit_node(func_decl);

        // Check that parameters have type annotations
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        let x_symbol = analyzer.symbol_table.lookup_in_scope("x", function_scope);
        assert!(x_symbol.is_some());
        let x_type_id = x_symbol.unwrap().type_id();
        assert!(x_type_id.is_some(), "Parameter 'x' should have a type annotation");

        let y_symbol = analyzer.symbol_table.lookup_in_scope("y", function_scope);
        assert!(y_symbol.is_some());
        let y_type_id = y_symbol.unwrap().type_id();
        assert!(y_type_id.is_some(), "Parameter 'y' should have a type annotation");

        // Verify the types are correct
        let x_type = type_interner.get(x_type_id.unwrap()).unwrap();
        assert!(matches!(x_type, Type::Primitive(PrimitiveType::Number)));

        let y_type = type_interner.get(y_type_id.unwrap()).unwrap();
        assert!(matches!(y_type, Type::Primitive(PrimitiveType::String)));
    }

    #[test]
    fn test_untyped_parameter() {
        let arena = Bump::new();
        let mut type_interner = TypeInterner::new();
        let mut analyzer = ScopeAnalyzer::new(&arena, &mut type_interner, test_span());

        let builder = NodeBuilder::new(&arena);

        // Create AST: function bar(x) { return x; }
        let body_block = builder.alloc(NodeKind::Block { statements: vec![] });
        let func_decl = builder.alloc_with_children(
            NodeKind::FunctionDeclaration {
                name: "bar".to_string(),
                params: vec![
                    crate::parser::ast::types::Parameter {
                        name: "x".to_string(),
                        type_annotation: None,
                        default_value: None,
                        is_rest: false,
                    },
                ],
                return_type: None,
                body: NodeId::new(0),
            },
            vec![body_block],
        );

        // Visit the function
        analyzer.visit_node(func_decl);

        // Parameter 'x' without type annotation should still be in function scope
        let scopes: Vec<_> = analyzer.scope_table.scopes().iter().collect();
        let function_scope = scopes[1].id();

        let x_symbol = analyzer.symbol_table.lookup_in_scope("x", function_scope);
        assert!(x_symbol.is_some(), "Untyped parameter 'x' should exist in function scope");

        // The type should be None (no annotation)
        let x_type_id = x_symbol.unwrap().type_id();
        assert!(x_type_id.is_none(), "Untyped parameter should not have a type annotation");
    }
}
