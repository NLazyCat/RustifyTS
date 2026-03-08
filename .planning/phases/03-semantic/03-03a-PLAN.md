---
phase: 03-semantic
plan: 03a
type: execute
wave: 3
depends_on: ["03-00", "03-01a", "03-02a"]
files_modified:
  - src/semantic/flow/mod.rs
  - src/semantic/flow/cfg.rs
  - src/semantic/flow/builder.rs
  - src/semantic/flow/dominance.rs
  - src/semantic/ir/mod.rs
  - src/semantic/ir/module.rs
  - src/semantic/ir/function.rs
  - src/semantic/ir/instruction.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "Control Flow Graph (CFG) is built with basic blocks for each function"
    - "Basic blocks have terminator instructions (br, condbr, ret)"
    - "Dominator tree is calculated for each CFG"
    - "IR represents all TypeScript statements and expressions in a structured format"
  artifacts:
    - path: "src/semantic/flow/cfg.rs"
      provides: "Control Flow Graph and Basic Block data structures"
      contains: "struct BasicBlock, struct ControlFlowGraph"
    - path: "src/semantic/flow/builder.rs"
      provides: "CFG builder visitor implementation"
      exports: "pub struct CFGBuilder"
    - path: "src/semantic/ir/instruction.rs"
      provides: "IR instruction definitions"
      contains: "enum Instruction { Add, Sub, Mul, Div, Load, Store, Call, Br, CondBr, Ret, ... }"
  key_links:
    - from: "src/semantic/flow/builder.rs"
      to: "src/parser/ast/visitor.rs"
      via: "Visitor trait implementation"
      pattern: "impl<'a> Visitor<'a> for CFGBuilder"
---

<objective>
Implement Control Flow Graph (CFG) construction and Intermediate Representation (IR) infrastructure.

Purpose: Create the low-level IR and CFG components that represent program structure and control flow.
Output: Complete IR representation and CFG builder for TypeScript code.
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
@.planning/phases/03-semantic/03-02a-PLAN.md

<interfaces>
<!-- Required types from prior plans -->
From src/semantic/scope/scope.rs:
```rust
pub struct ScopeId(u32);
pub struct ScopeTable;
```

From src/semantic/symbol/symbol.rs:
```rust
pub struct SymbolId(u32);
pub struct SymbolTable;
```

From src/semantic/types/representation.rs:
```rust
pub struct TypeId(u32);
pub struct TypeInterner;
```

<!-- AST Visitor -->
From src/parser/ast/visitor.rs:
```rust
pub trait Visitor<'a> {
    fn visit_node(&mut self, node: &'a AstNode<'a>);
    // ... other visit methods
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create CFG and IR module structures</name>
  <files>src/semantic/flow/mod.rs, src/semantic/ir/mod.rs</files>
  <action>
    1. Create src/semantic/flow/mod.rs with:
       - pub mod cfg;
       - pub mod builder;
       - pub mod dominance;
       - pub use cfg::*;
       - pub use builder::*;
       - pub use dominance::*;
       - #[cfg(test)] mod tests;
    2. Create src/semantic/ir/mod.rs with:
       - pub mod module;
       - pub mod function;
       - pub mod instruction;
       - pub use module::*;
       - pub use function::*;
       - pub use instruction::*;
       - #[cfg(test)] mod tests;
  </action>
  <verify>
    <automated>cargo build</automated>
  </verify>
  <done>CFG and IR module structures created without build errors</done>
</task>

<task type="auto">
  <name>Task 2: Implement IR instruction and function representation</name>
  <files>src/semantic/ir/instruction.rs, src/semantic/ir/function.rs, src/semantic/ir/module.rs</files>
  <action>
    1. Define ValueId as newtype around u32 for SSA values
    2. Define Instruction enum with variants:
       - Binary { op: BinaryOp, left: ValueId, right: ValueId }
       - Unary { op: UnaryOp, operand: ValueId }
       - Load { address: ValueId }
       - Store { address: ValueId, value: ValueId }
       - Call { function: ValueId, args: Vec&lt;ValueId&gt; }
       - Br { target: BasicBlockId }
       - CondBr { condition: ValueId, true_target: BasicBlockId, false_target: BasicBlockId }
       - Ret { value: Option&lt;ValueId&gt; }
       - Phi { incoming: Vec&lt;(ValueId, BasicBlockId)&gt; }
       - Alloca { ty: TypeId }
       - GetElementPtr { base: ValueId, indices: Vec&lt;ValueId&gt; }
    3. Define BasicBlock struct with id: BasicBlockId, instructions: Vec&lt;Instruction&gt;, terminator: Option&lt;Instruction&gt;
    4. Define Function struct with id: SymbolId, name: String, params: Vec&lt;(String, TypeId)&gt;, return_type: TypeId, blocks: Vec&lt;BasicBlock&gt;, entry_block: BasicBlockId
    5. Define SemanticModule struct with name: String, functions: Vec&lt;Function&gt;, types: TypeInterner, symbols: SymbolTable, scopes: ScopeTable
  </action>
  <verify>
    <automated>cargo test semantic::ir::tests::ir_basics</automated>
  </verify>
  <done>IR representation implemented with instructions, basic blocks, functions, and modules</done>
</task>

<task type="auto">
  <name>Task 3: Implement Control Flow Graph builder</name>
  <files>src/semantic/flow/cfg.rs, src/semantic/flow/builder.rs, src/semantic/flow/dominance.rs</files>
  <action>
    1. Define BasicBlockId as newtype around u32
    2. Define ControlFlowGraph struct with blocks: Vec&lt;BasicBlock&gt;, entry: BasicBlockId, exit: BasicBlockId
    3. Implement CFGBuilder visitor that:
       - Creates a new basic block for each code path
       - Inserts Br terminators for unconditional jumps
       - Inserts CondBr terminators for if/while/for conditions
       - Inserts Ret terminators for return statements
       - Handles break/continue statements with appropriate jumps
       - Builds PHI nodes for variables defined in multiple paths
    4. Implement dominator tree calculation in dominance.rs using the standard iterative algorithm
  </action>
  <verify>
    <automated>cargo test semantic::flow::tests::cfg_builder</automated>
  </verify>
  <done>CFG builder correctly constructs control flow graphs with proper terminators and dominator information</done>
</task>

</tasks>

<verification>
Run IR and CFG tests:
```bash
cargo test semantic::ir semantic::flow --no-fail-fast
```
</verification>

<success_criteria>
- IR representation captures all program semantics accurately
- CFG is correctly built for all function bodies with proper basic block structure
- Dominator tree calculation is accurate for control flow analysis
- All IR and CFG tests pass successfully
</success_criteria>

<output>
After completion, create `.planning/phases/03-semantic/03-03a-SUMMARY.md`
</output>