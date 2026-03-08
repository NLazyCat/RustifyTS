---
phase: 03-semantic
plan: 01b
type: execute
wave: 3
depends_on: ["03-00", "03-01a"]
files_modified:
  - src/semantic/symbol/table.rs
  - src/semantic/scope/analyzer.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "Symbol table supports lexical lookup with parent chain traversal"
    - "Scope analyzer correctly creates nested scopes for blocks, functions, loops, and catch blocks"
    - "Variable hoisting for function declarations and var statements works correctly"
    - "Block-level scoping for let/const declarations is enforced"
    - "Symbol shadowing is allowed per TypeScript semantics"
  artifacts:
    - path: "src/semantic/symbol/table.rs"
      provides: "Symbol table with hash-based storage and parent links"
      exports: "pub struct SymbolTable"
    - path: "src/semantic/scope/analyzer.rs"
      provides: "Scope visitor implementation using existing AST visitor trait"
      exports: "pub struct ScopeAnalyzer"
  key_links:
    - from: "src/semantic/scope/analyzer.rs"
      to: "src/parser/ast/visitor.rs"
      via: "Visitor trait implementation"
      pattern: "impl<'a> Visitor<'a> for ScopeAnalyzer"
    - from: "src/semantic/symbol/table.rs"
      to: "src/semantic/scope/scope.rs"
      via: "ScopeId references"
      pattern: "HashMap<ScopeId, Scope>"
---

<objective>
Implement symbol table and scope analyzer visitor that processes AST nodes according to ES6 semantics.

Purpose: Build the working scope analysis infrastructure that builds scope hierarchy and populates the symbol table.
Output: Fully functional symbol table and scope analyzer that correctly processes TypeScript AST.
</objective>

<execution_context>
@C:/Users/16017/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/16017/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@src/parser/ast/visitor.rs
@.planning/phases/03-semantic/03-01a-PLAN.md

<interfaces>
<!-- Existing AST visitor interface -->
From src/parser/ast/visitor.rs:
```rust
pub trait Visitor<'a> {
    fn visit_node(&mut self, node: &'a AstNode<'a>);
    fn visit_block(&mut self, node: &'a AstNode<'a>);
    fn visit_function_decl(&mut self, node: &'a AstNode<'a>);
    fn visit_variable_statement(&mut self, node: &'a AstNode<'a>);
    // ... other typed visit methods
    fn default_visit_node(&mut self, node: &'a AstNode<'a>);
}
```

<!-- Types from 01a -->
From src/semantic/scope/scope.rs:
```rust
pub struct ScopeId(u32);
pub struct ScopeTable;
```

From src/semantic/symbol/symbol.rs:
```rust
pub struct SymbolId(u32);
pub struct Symbol;
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement SymbolTable data structure</name>
  <files>src/semantic/symbol/table.rs</files>
  <action>
    1. Define SymbolTable struct with symbols: Vec&lt;Symbol&gt;, by_name: FxHashMap&lt;(ScopeId, String), SymbolId&gt;
    2. Implement methods: insert, lookup, lookup_in_scope, lookup_lexical
    3. Ensure insert method accepts type_id parameter to associate symbols with their types
  </action>
  <verify>
    <automated>cargo test semantic::symbol::tests::symbol_table</automated>
  </verify>
  <done>SymbolTable implemented with lookup functionality and type_id support</done>
</task>

<task type="auto">
  <name>Task 2: Implement ScopeAnalyzer visitor</name>
  <files>src/semantic/scope/analyzer.rs</files>
  <action>
    1. Implement ScopeAnalyzer struct with scope_table: ScopeTable, symbol_table: SymbolTable, arena: &'a bumpalo::Bump, type_interner: &'a mut TypeInterner
    2. Implement Visitor trait for ScopeAnalyzer:
       - visit_block: Create new block scope, visit children, pop scope
       - visit_function_decl: Create function scope, add function symbol to current scope, visit children in function scope
       - visit_variable_statement: Handle var hoisting and let/const block scoping
       - visit_for_statement: Create loop scope
       - visit_catch_clause: Create catch scope
       - visit_class_decl: Create class scope, add class symbol
    3. Implement hoisting logic for function declarations and var statements
    4. Ensure each created symbol has its type_id field populated when type information is available
  </action>
  <verify>
    <automated>cargo test semantic::scope::tests::analyzer</automated>
  </verify>
  <done>ScopeAnalyzer correctly builds scope hierarchy and populates symbol table according to ES6 semantics</done>
</task>

</tasks>

<verification>
Run the full test suite for scope and symbol modules:
```bash
cargo test semantic::scope semantic::symbol --no-fail-fast
```
</verification>

<success_criteria>
- Symbol table works correctly with lexical lookup
- Scope hierarchy is correctly built for nested blocks, functions, loops, and catch clauses
- Variable hoisting behaves as expected for function declarations and var statements
- Block-level scoping is enforced for let/const declarations
- Symbol shadowing is allowed and works correctly
- Symbols have type_id field properly associated with their types
</success_criteria>

<output>
After completion, create `.planning/phases/03-semantic/03-01b-SUMMARY.md`
</output>