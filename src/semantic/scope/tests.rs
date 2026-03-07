//! Test suite for scope analysis module.

use super::*;
use crate::parser::ast::Span;

#[test]
fn scope_basics() {
    // Create a root module scope
    let root_span = Span::new(0, 100);
    let mut scope_table = ScopeTable::new(root_span);

    // Verify root scope properties
    let root_id = scope_table.root();
    assert_eq!(root_id.get(), 0);
    assert_eq!(scope_table.current(), root_id);

    let root_scope = scope_table.current_scope();
    assert_eq!(root_scope.kind(), ScopeKind::Module);
    assert_eq!(root_scope.parent(), None);
    assert_eq!(root_scope.span(), root_span);
    assert!(root_scope.symbols().is_empty());

    // Push a function scope
    let func_span = Span::new(10, 90);
    let func_id = scope_table.push_scope(ScopeKind::Function, func_span);

    assert_eq!(scope_table.current(), func_id);
    assert_eq!(func_id.get(), 1);

    let func_scope = scope_table.current_scope();
    assert_eq!(func_scope.kind(), ScopeKind::Function);
    assert_eq!(func_scope.parent(), Some(root_id));
    assert_eq!(func_scope.span(), func_span);

    // Push a block scope inside the function
    let block_span = Span::new(20, 80);
    let block_id = scope_table.push_scope(ScopeKind::Block, block_span);

    assert_eq!(scope_table.current(), block_id);
    assert_eq!(block_id.get(), 2);

    let block_scope = scope_table.current_scope();
    assert_eq!(block_scope.kind(), ScopeKind::Block);
    assert_eq!(block_scope.parent(), Some(func_id));
    assert_eq!(block_scope.span(), block_span);

    // Pop back to function scope
    let popped = scope_table.pop_scope();
    assert_eq!(popped, block_id);
    assert_eq!(scope_table.current(), func_id);

    // Pop back to root scope
    let popped = scope_table.pop_scope();
    assert_eq!(popped, func_id);
    assert_eq!(scope_table.current(), root_id);

    // Verify total scopes
    assert_eq!(scope_table.scope_count(), 3);
}

#[test]
fn test_scope_nested_scopes() {
    // Test nested scope functionality
    todo!("Implement nested scopes test");
}

#[test]
fn test_scope_identifier_lookup() {
    // Test identifier lookup in scope chain
    todo!("Implement identifier lookup test");
}