//! Visitor pattern for AST traversal
//!
//! This module provides the Visitor trait for traversing TypeScript AST nodes.
//! Visitors enable analysis, transformation, and code generation operations
//! by implementing custom logic for each node kind.

use super::types::{NodeKind, BinaryOperator, Literal};
use super::AstNode;

/// Visitor trait for AST traversal
///
/// Implement this trait to perform custom operations on AST nodes.
/// The visitor pattern allows separating traversal logic from the AST structure.
///
/// # Example
///
/// ```rust
/// struct NodeCounter { count: usize }
///
/// impl<'a> Visitor<'a> for NodeCounter {
///     fn visit_node(&mut self, node: &'a AstNode<'a>) {
///         self.count += 1;
///         self.default_visit_node(node);
///     }
/// }
/// ```
pub trait Visitor<'a> {
    /// Visit any AST node
    ///
    /// Default implementation dispatches to the appropriate typed method
    /// based on the node kind.
    fn visit_node(&mut self, node: &'a AstNode<'a>) {
        match node.kind() {
            // Statements
            NodeKind::Block { .. } => self.visit_block(node),
            NodeKind::ExpressionStatement { .. } => self.visit_expression_statement(node),
            NodeKind::If { .. } => self.visit_if(node),
            NodeKind::For { .. } => self.visit_for(node),
            NodeKind::ForOf { .. } => self.visit_for_of(node),
            NodeKind::While { .. } => self.visit_while(node),
            NodeKind::DoWhile { .. } => self.visit_do_while(node),
            NodeKind::Return { .. } => self.visit_return(node),
            NodeKind::Break { .. } => self.visit_break(node),
            NodeKind::Continue { .. } => self.visit_continue(node),
            NodeKind::Switch { .. } => self.visit_switch(node),
            NodeKind::Try { .. } => self.visit_try(node),
            NodeKind::Throw { .. } => self.visit_throw(node),
            NodeKind::VariableStatement { .. } => self.visit_variable_statement(node),

            // Expressions
            NodeKind::Identifier { .. } => self.visit_identifier(node),
            NodeKind::Literal(..) => self.visit_literal(node),
            NodeKind::Array { .. } => self.visit_array(node),
            NodeKind::Object { .. } => self.visit_object(node),
            NodeKind::Binary { .. } => self.visit_binary(node),
            NodeKind::Unary { .. } => self.visit_unary(node),
            NodeKind::Assignment { .. } => self.visit_assignment(node),
            NodeKind::Conditional { .. } => self.visit_conditional(node),
            NodeKind::Call { .. } => self.visit_call(node),
            NodeKind::Member { .. } => self.visit_member(node),
            NodeKind::New { .. } => self.visit_new(node),
            NodeKind::ArrowFunction { .. } => self.visit_arrow_function(node),
            NodeKind::FunctionExpression { .. } => self.visit_function_expression(node),
            NodeKind::This => self.visit_this(node),
            NodeKind::Super => self.visit_super(node),
            NodeKind::Template { .. } => self.visit_template(node),
            NodeKind::Sequence { .. } => self.visit_sequence(node),

            // Declarations
            NodeKind::FunctionDeclaration { .. } => self.visit_function_declaration(node),
            NodeKind::ClassDeclaration { .. } => self.visit_class_declaration(node),
            NodeKind::InterfaceDeclaration { .. } => self.visit_interface_declaration(node),
            NodeKind::TypeAliasDeclaration { .. } => self.visit_type_alias_declaration(node),
            NodeKind::EnumDeclaration { .. } => self.visit_enum_declaration(node),
            NodeKind::ImportDeclaration { .. } => self.visit_import_declaration(node),
            NodeKind::ExportDeclaration { .. } => self.visit_export_declaration(node),

            // Patterns
            NodeKind::ObjectPattern { .. } => self.visit_object_pattern(node),
            NodeKind::ArrayPattern { .. } => self.visit_array_pattern(node),
            NodeKind::RestPattern { .. } => self.visit_rest_pattern(node),

            // Types
            NodeKind::TypeReference { .. } => self.visit_type_reference(node),
            NodeKind::ArrayType { .. } => self.visit_array_type(node),
            NodeKind::UnionType { .. } => self.visit_union_type(node),
            NodeKind::IntersectionType { .. } => self.visit_intersection_type(node),
            NodeKind::TupleType { .. } => self.visit_tuple_type(node),
            NodeKind::FunctionType { .. } => self.visit_function_type(node),
            NodeKind::TypeParameter { .. } => self.visit_type_parameter(node),
            NodeKind::TypeAnnotation { .. } => self.visit_type_annotation(node),

            // Module
            NodeKind::SourceFile { .. } => self.visit_source_file(node),
            NodeKind::ModuleDeclaration { .. } => self.visit_module_declaration(node),
        }
    }

    /// Default node visiting: visit all children
    fn default_visit_node(&mut self, node: &'a AstNode<'a>) {
        for child in node.children() {
            self.visit_node(*child);
        }
    }

    // --- Statement visitors ---

    /// Visit a block statement
    fn visit_block(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an expression statement
    fn visit_expression_statement(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an if statement
    fn visit_if(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a for statement
    fn visit_for(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a for-of statement
    fn visit_for_of(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a while statement
    fn visit_while(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a do-while statement
    fn visit_do_while(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a return statement
    fn visit_return(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a break statement
    fn visit_break(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a continue statement
    fn visit_continue(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a switch statement
    fn visit_switch(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a try statement
    fn visit_try(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a throw statement
    fn visit_throw(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a variable statement
    fn visit_variable_statement(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    // --- Expression visitors ---

    /// Visit an identifier
    fn visit_identifier(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a literal value
    fn visit_literal(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an array literal
    fn visit_array(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an object literal
    fn visit_object(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a binary expression
    fn visit_binary(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a unary expression
    fn visit_unary(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an assignment expression
    fn visit_assignment(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a conditional (ternary) expression
    fn visit_conditional(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a function call
    fn visit_call(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a member access
    fn visit_member(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a new expression
    fn visit_new(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an arrow function
    fn visit_arrow_function(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a function expression
    fn visit_function_expression(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a this expression
    fn visit_this(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a super expression
    fn visit_super(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a template literal
    fn visit_template(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a sequence expression
    fn visit_sequence(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    // --- Declaration visitors ---

    /// Visit a function declaration
    fn visit_function_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a class declaration
    fn visit_class_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an interface declaration
    fn visit_interface_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a type alias declaration
    fn visit_type_alias_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an enum declaration
    fn visit_enum_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an import declaration
    fn visit_import_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an export declaration
    fn visit_export_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    // --- Pattern visitors ---

    /// Visit an object pattern
    fn visit_object_pattern(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an array pattern
    fn visit_array_pattern(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a rest pattern
    fn visit_rest_pattern(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    // --- Type visitors ---

    /// Visit a type reference
    fn visit_type_reference(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an array type
    fn visit_array_type(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a union type
    fn visit_union_type(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit an intersection type
    fn visit_intersection_type(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a tuple type
    fn visit_tuple_type(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a function type
    fn visit_function_type(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a type parameter
    fn visit_type_parameter(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a type annotation
    fn visit_type_annotation(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    // --- Module visitors ---

    /// Visit a source file node
    fn visit_source_file(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }

    /// Visit a module declaration
    fn visit_module_declaration(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }
}

/// Node counter visitor
///
/// Counts the total number of nodes in an AST by traversing
/// all nodes and incrementing a counter on each visit.
pub struct NodeCounter {
    /// Total count of visited nodes
    pub count: usize,
}

impl NodeCounter {
    /// Create a new node counter
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Count nodes in an AST tree
    ///
    /// Returns the total number of nodes in the tree.
    pub fn count<'a>(root: &'a AstNode<'a>) -> usize {
        let mut counter = Self::new();
        counter.visit_node(root);
        counter.count
    }

    /// Reset the counter to zero
    pub fn reset(&mut self) {
        self.count = 0;
    }
}

impl Default for NodeCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Visitor<'a> for NodeCounter {
    fn visit_node(&mut self, node: &'a AstNode<'a>) {
        self.count += 1;
        self.default_visit_node(node);
    }
}

/// Depth calculator visitor
///
/// Calculates the maximum depth of an AST tree by tracking the
/// depth level as it traverses the tree.
pub struct DepthCalculator {
    /// Current depth level
    current_depth: usize,
    /// Maximum depth found
    max_depth: usize,
}

impl DepthCalculator {
    /// Create a new depth calculator
    pub fn new() -> Self {
        Self {
            current_depth: 0,
            max_depth: 0,
        }
    }

    /// Calculate the maximum depth of an AST tree
    ///
    /// Returns the maximum nesting depth of the tree.
    pub fn depth<'a>(root: &'a AstNode<'a>) -> usize {
        let mut calculator = Self::new();
        calculator.visit_node(root);
        calculator.max_depth
    }
}

impl Default for DepthCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Visitor<'a> for DepthCalculator {
    fn visit_node(&mut self, node: &'a AstNode<'a>) {
        self.current_depth += 1;
        self.max_depth = self.max_depth.max(self.current_depth);

        self.default_visit_node(node);

        self.current_depth -= 1;
    }
}

/// Collect identifiers visitor
///
/// Collects all identifier names found in an AST tree.
pub struct CollectIdentifiers {
    /// Collected identifier names
    identifiers: Vec<String>,
}

impl CollectIdentifiers {
    /// Create a new identifier collector
    pub fn new() -> Self {
        Self {
            identifiers: Vec::new(),
        }
    }

    /// Collect all identifiers from an AST tree
    ///
    /// Returns a vector of all identifier names found in the tree.
    pub fn collect<'a>(root: &'a AstNode<'a>) -> Vec<String> {
        let mut collector = Self::new();
        collector.visit_node(root);
        collector.identifiers
    }
}

impl Default for CollectIdentifiers {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Visitor<'a> for CollectIdentifiers {
    fn visit_identifier(&mut self, node: &'a AstNode<'a>) {
        if let NodeKind::Identifier { name } = node.kind() {
            self.identifiers.push(name.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bumpalo::Bump;
    use crate::parser::ast::{NodeBuilder, NodeId};

    fn create_test_ast<'a>(arena: &'a Bump) -> &'a AstNode<'a> {
        let builder = NodeBuilder::new(arena);

        // Build a simple AST:
        // let x = 1;
        // let y = 2;
        // return x + y;

        let lit_1 = builder.alloc(NodeKind::Literal(Literal::Number(1.0)));
        let lit_2 = builder.alloc(NodeKind::Literal(Literal::Number(2.0)));

        let var_x = builder.alloc(NodeKind::Identifier { name: "x".to_string() });
        let var_y = builder.alloc(NodeKind::Identifier { name: "y".to_string() });

        // x + y
        let add_expr = builder.alloc_with_children(
            NodeKind::Binary {
                operator: BinaryOperator::Add,
                left: NodeId::new(0),
                right: NodeId::new(1),
            },
            vec![var_x, var_y],
        );

        // return x + y;
        let return_stmt = builder.alloc_with_children(
            NodeKind::Return { value: Some(NodeId::new(0)) },
            vec![add_expr],
        );

        // Block with return statement
        let block = builder.alloc_with_children(
            NodeKind::Block { statements: vec![NodeId::new(0)] },
            vec![return_stmt],
        );

        block
    }

    #[test]
    fn test_visitor_default_traversal() {
        let arena = Bump::new();
        let root = create_test_ast(&arena);

        let mut counter = NodeCounter::new();
        counter.visit_node(root);

        // Should count all nodes in the tree
        assert!(counter.count > 0);
    }

    #[test]
    fn test_node_counter() {
        let arena = Bump::new();
        let root = create_test_ast(&arena);

        let count = NodeCounter::count(root);
        assert!(count > 0);

        // Count using instance method
        let mut counter = NodeCounter::new();
        counter.visit_node(root);
        assert_eq!(counter.count, count);

        // Reset and count again
        counter.reset();
        assert_eq!(counter.count, 0);
        counter.visit_node(root);
        assert_eq!(counter.count, count);
    }

    #[test]
    fn test_depth_calculator() {
        let arena = Bump::new();
        let root = create_test_ast(&arena);

        let depth = DepthCalculator::depth(root);
        assert!(depth >= 2); // At least root and one child level
    }

    #[test]
    fn test_collect_identifiers() {
        let arena = Bump::new();
        let root = create_test_ast(&arena);

        let identifiers = CollectIdentifiers::collect(root);
        assert!(identifiers.contains(&"x".to_string()));
        assert!(identifiers.contains(&"y".to_string()));
    }

    #[test]
    fn test_empty_ast() {
        let arena = Bump::new();
        let builder = NodeBuilder::new(&arena);

        // Create minimal AST
        let root = builder.alloc(NodeKind::Identifier { name: "test".to_string() });

        let count = NodeCounter::count(root);
        assert_eq!(count, 1);

        let depth = DepthCalculator::depth(root);
        assert_eq!(depth, 1);
    }

    #[test]
    fn test_visitor_default() {
        assert_eq!(NodeCounter::default().count, 0);
        assert_eq!(DepthCalculator::default().max_depth, 0);
        assert_eq!(CollectIdentifiers::default().identifiers.len(), 0);
    }

    #[test]
    fn test_nested_structure() {
        let arena = Bump::new();
        let builder = NodeBuilder::new(&arena);

        // Build deeply nested structure: ((1))
        let lit = builder.alloc(NodeKind::Literal(Literal::Number(1.0)));
        let inner = builder.alloc_with_children(
            NodeKind::Binary {
                operator: BinaryOperator::Add,
                left: NodeId::new(0),
                right: NodeId::new(0),
            },
            vec![lit, lit],
        );
        let outer = builder.alloc_with_children(
            NodeKind::Binary {
                operator: BinaryOperator::Add,
                left: NodeId::new(0),
                right: NodeId::new(0),
            },
            vec![inner, inner],
        );

        let count = NodeCounter::count(outer);
        assert!(count >= 5); // 1 outer + 2 inner + 2 literals
    }
}
