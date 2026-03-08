---
phase: 03-semantic
plan: 01a
type: execute
wave: 2
depends_on: ["03-00"]
files_modified:
  - src/semantic/mod.rs
  - src/semantic/scope/mod.rs
  - src/semantic/scope/scope.rs
  - src/semantic/symbol/mod.rs
  - Cargo.toml
autonomous: true
requirements: []
must_haves:
  truths:
    - "Scope data structure correctly represents different scope kinds with parent links"
    - "Symbol data structure correctly represents different symbol kinds with scope information"
    - "Module structure is properly set up for scope and symbol modules"
  artifacts:
    - path: "src/semantic/scope/scope.rs"
      provides: "Scope data structure with kind, parent link, and span"
      contains: "enum ScopeKind { Module, Function, Block, Loop, Catch, Class }"
    - path: "src/semantic/symbol/symbol.rs"
      provides: "Symbol type definitions with different symbol kinds"
      contains: "enum Symbol { Function, Variable, Class, Interface, TypeAlias }"
  key_links:
    - from: "src/semantic/mod.rs"
      to: "scope and symbol modules"
      via: "Module exports"
      pattern: "pub mod scope; pub mod symbol;"
---

<objective>
Implement core data structures for scope and symbol representation, and set up module infrastructure.

Purpose: Establish the foundational data structures required for scope analysis and symbol tracking.
Output: Complete module structure and core data types for scope and symbol handling.
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
@src/parser/ast/node.rs
@src/parser/ast/types.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add dependencies and create module structure</name>
  <files>Cargo.toml, src/semantic/mod.rs, src/semantic/scope/mod.rs, src/semantic/symbol/mod.rs</files>
  <action>
    1. Add required dependencies to Cargo.toml: lasso = "0.7.x", hashbrown = "0.14.x", fxhash = "0.2.1"
    2. Create src/semantic/mod.rs with pub mod scope; pub mod symbol; pub mod types; pub mod flow; pub mod analyzer;
    3. Create src/semantic/scope/mod.rs with pub mod scope; pub mod analyzer; pub use scope::*; pub use analyzer::*; #[cfg(test)] mod tests;
    4. Create src/semantic/symbol/mod.rs with pub mod symbol; pub mod table; pub use symbol::*; pub use table::*; #[cfg(test)] mod tests;
  </action>
  <verify>
    <automated>cargo build</automated>
  </verify>
  <done>Dependencies added and module structure created without build errors</done>
</task>

<task type="auto">
  <name>Task 2: Implement Scope and ScopeTable data structures</name>
  <files>src/semantic/scope/scope.rs</files>
  <action>
    1. Define ScopeKind enum: Module, Function, Block, Loop, Catch, Class
    2. Define ScopeId as newtype around u32
    3. Define Scope struct with id: ScopeId, kind: ScopeKind, parent: Option&lt;ScopeId&gt;, span: Span, symbols: FxHashMap&lt;String, SymbolId&gt;
    4. Define ScopeTable struct with scopes: Vec&lt;Scope&gt;, root: ScopeId, current: ScopeId
    5. Implement methods: create_scope, get_scope, get_current_scope, push_scope, pop_scope
  </action>
  <verify>
    <automated>cargo test semantic::scope::tests::scope_basics</automated>
  </verify>
  <done>Scope and ScopeTable types implemented with basic functionality</done>
</task>

<task type="auto">
  <name>Task 3: Implement Symbol data structure</name>
  <files>src/semantic/symbol/symbol.rs</files>
  <action>
    1. Define SymbolId as newtype around u32
    2. Define SymbolKind enum: Function, Variable, Class, Interface, TypeAlias, Enum, Import
    3. Define Symbol struct with id: SymbolId, name: String, kind: SymbolKind, span: Span, scope: ScopeId, is_export: bool, type_id: Option&lt;TypeId&gt;
    4. Implement basic methods for Symbol type
  </action>
  <verify>
    <automated>cargo test semantic::symbol::tests::symbol_basics</automated>
  </verify>
  <done>Symbol type implemented with all required fields including type_id for type information</done>
</task>

</tasks>

<verification>
Run basic tests for scope and symbol data structures:
```bash
cargo test semantic::scope::tests::scope_basics semantic::symbol::tests::symbol_basics --no-fail-fast
```
</verification>

<success_criteria>
- Module structure is properly set up with all required dependencies
- Scope and ScopeTable types are implemented with basic functionality
- Symbol type includes type_id field to store type information
- All basic tests pass successfully
</success_criteria>

<output>
After completion, create `.planning/phases/03-semantic/03-01a-SUMMARY.md`
</output>