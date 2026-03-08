---
phase: 03-semantic
plan: 02b
type: execute
wave: 3
depends_on: ["03-00", "03-02a"]
files_modified:
  - src/semantic/types/unify.rs
  - src/semantic/types/resolver.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "Type compatibility checking follows TypeScript rules"
    - "Generic types with type parameters are supported"
    - "Type substitution for generics works correctly"
    - "Type resolution correctly resolves type references to actual types"
  artifacts:
    - path: "src/semantic/types/unify.rs"
      provides: "Type compatibility checking according to TypeScript rules"
      exports: "pub fn is_subtype(a: TypeId, b: TypeId, interner: &TypeInterner) -> bool"
    - path: "src/semantic/types/resolver.rs"
      provides: "Type resolution pass that resolves type references"
      exports: "pub struct TypeResolver"
  key_links:
    - from: "src/semantic/types/unify.rs"
      to: "src/semantic/types/representation.rs"
      via: "Type variant matching"
      pattern: "match interner.get(a)"
---

<objective>
Implement type compatibility checking and type resolution functionality.

Purpose: Provide TypeScript-compatible type checking and resolution capabilities required for semantic analysis.
Output: Fully functional type system with compatibility checking and type resolution.
</objective>

<execution_context>
@C:/Users/16017/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/16017/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/03-semantic/03-02a-PLAN.md

<interfaces>
<!-- Types from 02a -->
From src/semantic/types/representation.rs:
```rust
pub struct TypeId(u32);
pub enum Type;
```

From src/semantic/types/interner.rs:
```rust
pub struct TypeInterner;
impl TypeInterner {
    pub fn get(&self, id: TypeId) -> Option<&Type>;
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement type compatibility checking</name>
  <files>src/semantic/types/unify.rs</files>
  <action>
    1. Implement is_subtype(a: TypeId, b: TypeId, interner: &TypeInterner) -> bool following TypeScript rules:
       - Primitive types: string <: unknown, number <: unknown, etc.
       - Union types: A <: B | C if A <: B or A <: C
       - Intersection types: A & B <: A and A & B <: B
       - Function types: parameter contravariance, return type covariance
       - Object types: structural typing (extra properties allowed)
       - Array types: covariant element types (matching TypeScript behavior)
       - Generic types: type parameter variance according to usage
    2. Implement type substitution for generics: substitute_type_params(ty: TypeId, substitutions: &FxHashMap<TypeId, TypeId>, interner: &mut TypeInterner) -> TypeId
  </action>
  <verify>
    <automated>cargo test semantic::types::tests::unify</automated>
  </verify>
  <done>Type checking correctly implements TypeScript compatibility rules</done>
</task>

<task type="auto">
  <name>Task 2: Implement type resolution pass</name>
  <files>src/semantic/types/resolver.rs</files>
  <action>
    1. Define TypeResolver struct with symbol_table: &SymbolTable, type_interner: &mut TypeInterner
    2. Implement resolve_type_reference method that resolves Type::Reference variants to actual types
    3. Implement resolve_method that traverses AST nodes and resolves types for expressions and declarations
    4. Ensure resolved types are stored in the symbol table entries via the type_id field
    5. Implement visitor pattern for type resolution to process AST nodes
  </action>
  <verify>
    <automated>cargo test semantic::types::tests::resolver</automated>
  </verify>
  <done>Type resolver correctly resolves type references and associates types with symbols</done>
</task>

</tasks>

<verification>
Run the full type system test suite:
```bash
cargo test semantic::types --no-fail-fast
```
</verification>

<success_criteria>
- Type compatibility checking follows TypeScript semantics exactly
- Generic type substitution works correctly for type parameters
- Type resolution correctly resolves named type references
- Types are properly associated with symbol table entries
- All type system tests pass successfully
</success_criteria>

<output>
After completion, create `.planning/phases/03-semantic/03-02b-SUMMARY.md`
</output>