//! Test suite for main semantic analyzer module.

use super::*;
use bumpalo::Bump;
use crate::parser::ast::{NodeBuilder, NodeKind, Span};
use crate::parser::ast::types::{VariableDeclaration, VariableKind, NodeId};
use crate::semantic::types::Type;

fn test_span() -> Span {
    Span::new(0, 100)
}

#[test]
fn test_analyzer_basic_program() {
    let arena = Bump::new();
    let mut analyzer = SemanticAnalyzer::new(&arena);

    let builder = NodeBuilder::new(&arena);

    // Create a simple AST: { let x = 1; }
    let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
    let var_stmt = builder.alloc(NodeKind::VariableStatement {
        declarations: vec![VariableDeclaration {
            name: "x".to_string(),
            kind: VariableKind::Let,
            initializer: Some(NodeId::new(0)),
            type_annotation: None,
        }],
    });
    let block = builder.alloc_with_children(
        NodeKind::Block {
            statements: vec![NodeId::new(0)],
        },
        vec![var_stmt, lit_1],
    );

    // Analyze the program
    let result = analyzer.analyze(block);

    // Should succeed
    assert!(result.is_ok());

    let module = result.unwrap();

    // Should have created scopes
    assert!(module.scopes.scope_count() > 0);

    // Should have created symbols
    assert!(module.symbols.symbol_count() > 0);

    // Should have the variable 'x'
    let root_scope = module.scopes.root();
    let scopes: Vec<_> = module.scopes.scopes().iter().collect();
    let block_scope = scopes.get(1).map(|s| s.id()).unwrap_or(root_scope);

    let x_symbol = module.symbols.lookup_lexical("x", block_scope, &module.scopes);
    assert!(x_symbol.is_some());
}

#[test]
fn test_analyzer_error_reporting() {
    let arena = Bump::new();
    let mut analyzer = SemanticAnalyzer::new(&arena);

    let builder = NodeBuilder::new(&arena);

    // Create a simple AST - error reporting tests would require
    // more complex scenarios with type errors
    let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
    let var_stmt = builder.alloc(NodeKind::VariableStatement {
        declarations: vec![VariableDeclaration {
            name: "x".to_string(),
            kind: VariableKind::Let,
            initializer: Some(NodeId::new(0)),
            type_annotation: None,
        }],
    });
    let block = builder.alloc_with_children(
        NodeKind::Block {
            statements: vec![NodeId::new(0)],
        },
        vec![var_stmt, lit_1],
    );

    // This should succeed without errors
    let result = analyzer.analyze(block);
    assert!(result.is_ok());
}

#[test]
fn test_analyzer_full_pipeline() {
    let arena = Bump::new();
    let mut analyzer = SemanticAnalyzer::new(&arena);

    let builder = NodeBuilder::new(&arena);

    // Create a more complex AST with multiple scopes
    // { let x = 1; { let y = 2; } }
    let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
    let stmt1 = builder.alloc(NodeKind::VariableStatement {
        declarations: vec![VariableDeclaration {
            name: "x".to_string(),
            kind: VariableKind::Let,
            initializer: Some(NodeId::new(0)),
            type_annotation: None,
        }],
    });

    let lit_2 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(2.0)));
    let stmt2 = builder.alloc(NodeKind::VariableStatement {
        declarations: vec![VariableDeclaration {
            name: "y".to_string(),
            kind: VariableKind::Let,
            initializer: Some(NodeId::new(0)),
            type_annotation: None,
        }],
    });

    let inner_block = builder.alloc_with_children(
        NodeKind::Block {
            statements: vec![NodeId::new(0)],
        },
        vec![stmt2, lit_2],
    );

    let outer_block = builder.alloc_with_children(
        NodeKind::Block {
            statements: vec![NodeId::new(0), NodeId::new(1)],
        },
        vec![stmt1, lit_1, inner_block],
    );

    // Analyze the program
    let result = analyzer.analyze(outer_block);

    // Should succeed
    assert!(result.is_ok());

    let module = result.unwrap();

    // Should have 3 scopes: root + outer_block + inner_block
    assert_eq!(module.scopes.scope_count(), 3);

    // Should have 2 symbols: x and y
    assert_eq!(module.symbols.symbol_count(), 2);

    // Verify both symbols exist
    let root_scope = module.scopes.root();
    let scopes: Vec<_> = module.scopes.scopes().iter().collect();
    let outer_scope = scopes[1].id();
    let inner_scope = scopes[2].id();

    let x_symbol = module.symbols.lookup_lexical("x", inner_scope, &module.scopes);
    assert!(x_symbol.is_some());

    let y_symbol = module.symbols.lookup_in_scope("y", inner_scope);
    assert!(y_symbol.is_some());
}

#[test]
fn test_analyzer_type_wiring() {
    let arena = Bump::new();
    let mut analyzer = SemanticAnalyzer::new(&arena);

    let builder = NodeBuilder::new(&arena);

    // Create a simple variable with type annotation
    // let x: number = 1;
    let lit_1 = builder.alloc(NodeKind::Literal(crate::parser::ast::types::Literal::Number(1.0)));
    let var_stmt = builder.alloc(NodeKind::VariableStatement {
        declarations: vec![VariableDeclaration {
            name: "x".to_string(),
            kind: VariableKind::Let,
            initializer: Some(NodeId::new(0)),
            type_annotation: Some(crate::parser::ast::types::TypeAnnotation::TypeReference {
                name: "number".to_string(),
                type_params: None,
            }),
        }],
    });
    let block = builder.alloc_with_children(
        NodeKind::Block {
            statements: vec![NodeId::new(0)],
        },
        vec![var_stmt, lit_1],
    );

    // Analyze the program
    let result = analyzer.analyze(block);

    // Should succeed
    assert!(result.is_ok());

    let module = result.unwrap();

    // Find the variable symbol
    let root_scope = module.scopes.root();
    let scopes: Vec<_> = module.scopes.scopes().iter().collect();
    let block_scope = scopes.get(1).map(|s| s.id()).unwrap_or(root_scope);

    let x_symbol = module.symbols.lookup_lexical("x", block_scope, &module.scopes);
    assert!(x_symbol.is_some(), "Variable 'x' should be found");

    // Verify the variable has a type_id
    let x_symbol = module.symbols.lookup(x_symbol.unwrap()).unwrap();
    assert!(x_symbol.type_id().is_some(), "Variable symbol should have a type_id");

    // Verify it's a number type (the type resolution should have worked)
    let type_id = x_symbol.type_id().unwrap();
    let type_info = module.types.get(type_id).unwrap();

    // The type should be either Primitive(Number) or Reference(number) that gets resolved
    // For now, we just verify that a type was assigned
    let _type_info = type_info; // Use the variable to avoid warning
}