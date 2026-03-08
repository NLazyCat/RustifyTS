---
phase: 03-semantic
plan: 02a
type: execute
wave: 2
depends_on: ["03-00"]
files_modified:
  - src/semantic/types/mod.rs
  - src/semantic/types/representation.rs
  - src/semantic/types/interner.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "Type system supports all TypeScript primitive and composite types"
    - "Types are interned with unique identifiers for fast comparison"
    - "Type interner correctly deduplicates identical types"
  artifacts:
    - path: "src/semantic/types/representation.rs"
      provides: "Type data structure with all TypeScript type variants"
      contains: "enum Type { Primitive, String, Number, Boolean, Null, Undefined, Array, Object, Function, Union, Intersection, Generic, TypeParameter }"
    - path: "src/semantic/types/interner.rs"
      provides: "Type interner using lasso for deduplication"
      exports: "pub struct TypeInterner"
  key_links:
    - from: "src/semantic/types/interner.rs"
      to: "lasso::Rodeo"
      via: "Type interning implementation"
      pattern: "Rodeo<Type, TypeId>"
---

<objective>
Implement type representation and interning infrastructure for the TypeScript type system.

Purpose: Create the foundational type system components that enable efficient type representation and comparison.
Output: Complete type representation enum and type interner with deduplication support.
</objective>

<execution_context>
@C:/Users/16017/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/16017/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/03-semantic/03-RESEARCH.md

<interfaces>
<!-- Required types from lasso crate -->
From lasso documentation:
```rust
pub struct Rodeo<K = Spur, S = RandomState> { /* fields omitted */ }
impl<K: Key, S: BuildHasher> Rodeo<K, S> {
    pub fn get_or_intern<T>(&mut self, val: T) -> K where T: AsRef<str> + Into<String>;
    pub fn get(&self, key: K) -> Option<&str>;
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create type system module structure</name>
  <files>src/semantic/types/mod.rs</files>
  <action>
    1. Create src/semantic/types/mod.rs with:
       - pub mod representation;
       - pub mod interner;
       - pub mod unify;
       - pub mod resolver;
       - pub use representation::*;
       - pub use interner::*;
       - pub use unify::*;
       - pub use resolver::*;
       - #[cfg(test)] mod tests;
  </action>
  <verify>
    <automated>cargo build</automated>
  </verify>
  <done>Type system module structure created without build errors</done>
</task>

<task type="auto">
  <name>Task 2: Implement type representation</name>
  <files>src/semantic/types/representation.rs</files>
  <action>
    1. Define TypeId as newtype around u32 implementing lasso::Key trait
    2. Define PrimitiveType enum: String, Number, Boolean, Null, Undefined, Void, Never, Unknown, Any
    3. Define Type enum with variants:
       - Primitive(PrimitiveType)
       - Array(Box&lt;Type&gt;)
       - Tuple(Vec&lt;Type&gt;)
       - Object { properties: FxHashMap&lt;String, Type&gt;, index_signature: Option&lt;Box&lt;Type&gt;&gt; }
       - Function { params: Vec&lt;Type&gt;, return_type: Box&lt;Type&gt;, type_params: Vec&lt;TypeParameter&gt; }
       - Union(Vec&lt;Type&gt;)
       - Intersection(Vec&lt;Type&gt;)
       - TypeParameter { name: String, constraint: Option&lt;Box&lt;Type&gt;&gt;, default: Option&lt;Box&lt;Type&gt;&gt; }
       - Generic { base: TypeId, args: Vec&lt;Type&gt; }
       - Reference { name: String, type_args: Vec&lt;Type&gt; }
    4. Implement Eq, PartialEq, Hash for Type (note: types are compared by TypeId, not structural equality)
  </action>
  <verify>
    <automated>cargo test semantic::types::tests::representation</automated>
  </verify>
  <done>Type enum defined with all required TypeScript type variants</done>
</task>

<task type="auto">
  <name>Task 3: Implement type interner</name>
  <files>src/semantic/types/interner.rs</files>
  <action>
    1. Use lasso::Rodeo as the underlying interner storage
    2. Define TypeInterner struct wrapping Rodeo&lt;Type, TypeId&gt;
    3. Implement methods:
       - new() -> Self
       - intern(&mut self, ty: Type) -> TypeId
       - get(&self, id: TypeId) -> Option<&Type>
       - get_or_intern_primitive(&mut self, prim: PrimitiveType) -> TypeId
       - get_or_intern_array(&mut self, element: TypeId) -> TypeId
       - get_or_intern_union(&mut self, types: Vec<TypeId>) -> TypeId (sorted and deduplicated)
    4. Ensure types are immutable once interned
  </action>
  <verify>
    <automated>cargo test semantic::types::tests::interner</automated>
  </verify>
  <done>Type interner implemented with deduplication and fast lookup</done>
</task>

</tasks>

<verification>
Run type representation and interner tests:
```bash
cargo test semantic::types::tests::representation semantic::types::tests::interner --no-fail-fast
```
</verification>

<success_criteria>
- All TypeScript type variants are correctly represented
- Type interning deduplicates identical types
- Type comparisons by TypeId are fast and correct
- All tests for representation and interner pass
</success_criteria>

<output>
After completion, create `.planning/phases/03-semantic/03-02a-SUMMARY.md`
</output>