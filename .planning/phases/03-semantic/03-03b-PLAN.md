---
phase: 03-semantic
plan: 03b
type: execute
wave: 4
depends_on: ["03-00", "03-01b", "03-02b", "03-03a"]
files_modified:
  - src/semantic/analyzer.rs
  - src/semantic/mod.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "Type system is properly wired to scope/symbol analysis"
    - "Type resolution pass correctly resolves types for all symbols"
    - "Main semantic analyzer coordinates all analysis passes"
    - "Semantic module contains complete scope, symbol, type, and CFG information"
  artifacts:
    - path: "src/semantic/analyzer.rs"
      provides: "Main semantic analyzer coordinator"
      exports: "pub struct SemanticAnalyzer"
  key_links:
    - from: "src/semantic/analyzer.rs"
      to: "src/semantic/scope/analyzer.rs"
      via: "Scope analysis pass"
      pattern: "scope_analyzer.analyze(ast)"
    - from: "src/semantic/analyzer.rs"
      to: "src/semantic/types/resolver.rs"
      via: "Type resolution pass"
      pattern: "type_resolver.resolve(ast, &symbol_table)"
    - from: "src/semantic/analyzer.rs"
      to: "src/semantic/flow/builder.rs"
      via: "CFG construction pass"
      pattern: "cfg_builder.build(ast)"
---

<objective>
Implement the main semantic analyzer coordinator that wires all analysis passes together.

Purpose: Integrate all semantic analysis components into a single cohesive pipeline that produces complete semantic information.
Output: Fully functional semantic analysis pipeline that processes AST and produces a semantic module.
</objective>

<execution_context>
@C:/Users/16017/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/16017/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/03-semantic/03-01b-PLAN.md
@.planning/phases/03-semantic/03-02b-PLAN.md
@.planning/phases/03-semantic/03-03a-PLAN.md

<interfaces>
<!-- Components from prior plans -->
From src/semantic/scope/analyzer.rs:
```rust
pub struct ScopeAnalyzer<'a>;
impl<'a> ScopeAnalyzer<'a> {
    pub fn new(arena: &'a bumpalo::Bump, type_interner: &'a mut TypeInterner) -> Self;
    pub fn analyze(&mut self, ast: &'a AstNode<'a>) -> (ScopeTable, SymbolTable);
}
```

From src/semantic/types/resolver.rs:
```rust
pub struct TypeResolver<'a>;
impl<'a> TypeResolver<'a> {
    pub fn new(symbol_table: &'a SymbolTable, type_interner: &'a mut TypeInterner) -> Self;
    pub fn resolve(&mut self, ast: &'a AstNode<'a>) -> Result<(), TypeError>;
}
```

From src/semantic/flow/builder.rs:
```rust
pub struct CFGBuilder<'a>;
impl<'a> CFGBuilder<'a> {
    pub fn new(symbol_table: &'a SymbolTable, type_interner: &'a TypeInterner) -> Self;
    pub fn build(&mut self, ast: &'a AstNode<'a>) -> Vec<Function>;
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement main SemanticAnalyzer coordinator</name>
  <files>src/semantic/analyzer.rs, src/semantic/mod.rs</files>
  <action>
    1. Define SemanticAnalyzer struct with arena: &'a bumpalo::Bump, type_interner: TypeInterner
    2. Implement analyze method that:
       - Creates and runs ScopeAnalyzer to build scope hierarchy and symbol table
       - Creates and runs TypeResolver to resolve types for all expressions and symbols
       - Ensures type information is stored in symbol table entries via type_id field
       - Creates and runs CFGBuilder to build control flow graphs for all functions
       - Constructs the final SemanticModule with all analysis results
    3. Export the main API from src/semantic/mod.rs: pub fn analyze(ast: &AstNode, arena: &bumpalo::Bump) -> Result&lt;SemanticModule, SemanticError&gt;
  </action>
  <verify>
    <automated>cargo test semantic::analyzer::tests::end_to_end</automated>
  </verify>
  <done>Main semantic analyzer coordinates all passes and produces a complete SemanticModule</done>
</task>

<task type="auto">
  <name>Task 2: Wire type information to symbol table</name>
  <files>src/semantic/analyzer.rs, src/semantic/symbol/symbol.rs</files>
  <action>
    1. Ensure TypeResolver updates symbol table entries with resolved type information
    2. Verify that every Symbol in the symbol table has a type_id field populated
    3. Add validation step to ensure all symbols have valid type information after analysis
  </action>
  <verify>
    <automated>cargo test semantic::analyzer::tests::type_wiring</automated>
  </verify>
  <done>All symbols in the symbol table have correctly associated type information</done>
</task>

</tasks>

<verification>
Run the full semantic analysis test suite:
```bash
cargo test semantic --no-fail-fast
```

Verify end-to-end analysis of a sample TypeScript file:
```bash
cargo run -- analyze tests/fixtures/original/simple-function.ts
```
</verification>

<success_criteria>
- Type system is properly integrated with scope analysis
- All symbols have correct type information associated with them
- Main analyzer coordinates all passes successfully
- Complete semantic module is produced with scope, symbol, type, and CFG information
- End-to-end analysis works for sample TypeScript files
</success_criteria>

<output>
After completion, create `.planning/phases/03-semantic/03-03b-SUMMARY.md`
</output>