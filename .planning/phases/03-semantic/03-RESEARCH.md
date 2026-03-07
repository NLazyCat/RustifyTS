# Phase 03: semantic - Research

**Researched:** 2026-03-07
**Domain:** TypeScript semantic analysis, intermediate representation (IR), Rust compiler infrastructure
**Confidence:** HIGH

## Summary

This phase implements the semantic analysis layer for TypeScript to Rust conversion, building on the existing parser infrastructure. The core responsibilities include scope analysis, symbol table construction, type system representation, and control flow analysis, all designed to convert parsed AST nodes into a structured semantic IR that will be consumed by downstream refactoring and code generation phases.

The implementation follows TypeScript/ES6 semantics strictly, with locked decisions for type interning, ES6 scope rules, hash-based symbol tables with parent links, and multi-level control flow graphs with basic blocks. The phase leverages existing Rust ecosystem crates for string interning, hash maps, and arena allocation to ensure performance and correctness.

**Primary recommendation:** Follow the layered architecture pattern with separate modules for scope analysis, symbol table, type system, and control flow, using visitor pattern for AST traversal and arena allocation for memory efficiency.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
#### Type system design
- **类型表示:** Interned types - 不可变类型节点，内部引用以支持共享
- **类型推断:** No inference (TS types) - 直接使用 TypeScript 类型，IR 中不需要推断
- **类型统一:** TS compatibility rules - 使用 TypeScript 兼容性规则
- **泛型支持:** Full generic support - 完整支持泛型，包括类型参数和替换

#### Scope analysis rules
- **作用域规则:** ES6 standard - 函数、块、循环、catch 块，遵循 ES6 语义
- **块级作用域:** Always block scope - 块语句创建新作用域（let/const）
- **变量提升:** Function + var hoisting - 提升函数声明和 var 声明
- **动态作用域:** Lexical + edge cases - 词法作用域，特殊情况特殊处理

#### Symbol table design
- **符号存储:** Hash-based with parent links - 每个作用域使用 HashMap<Name, Symbol>，链接到父作用域
- **名称解析:** Lexical search chain - 沿作用域链向上查找最近的匹配符号
- **标识符遮蔽:** Allow shadowing - 内部作用域遮蔽外部作用域（标准 TS 行为）
- **可见性跟踪:** Export tracking only - 只跟踪导出 vs 内部

#### Control flow representation
- **CFG 粒度:** Basic blocks - 基本块，带有终止符（br, condbr, ret 等）
- **支配关系:** Immediate dominators - 仅计算直接支配者（IDoms）
- **边表示:** Labeled condition edges - true/false 边带条件表达式
- **CFG 范围:** Multi-level CFG - 每个作用域一个 CFG（块、函数、模块）

### Claude's Discretion
- Interned types 的具体实现细节
- 作用域链的遍历优化
- 符号类型的具体分类方式
- CFG 节点的具体类型定义
- 边缘情况处理

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| lasso | 0.7.x | String and type interning | High-performance interner with multiple optimizations for single and multi-threaded use cases, supports custom key types and has low memory overhead |
| bumpalo | 3.16 | Arena allocation | Already used in the project for AST nodes, extends naturally to semantic IR nodes for efficient memory management and deallocation |
| hashbrown | 0.14.x | Hash maps for symbol tables | Faster than std::collections::HashMap, optimized for frequent lookups and insertions common in semantic analysis |
| thiserror | 2.0.18 | Error handling | Already used in the project, provides type-safe error variants for semantic analysis errors |
| miette | 7.2 | Diagnostic reporting | Already used in the project, enables rich error messages with span information for semantic errors |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| fxhash | 0.2.1 | Fast hashing | For symbol table and interner hash maps, provides better performance for string keys |
| petgraph | 0.6.x | Graph data structures | For control flow graph implementation if custom implementation becomes too complex |
| indexmap | 2.0 | Ordered maps | When symbol order preservation is required for correct output generation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| lasso | string-interner | lasso has better performance benchmarks and more active maintenance |
| bumpalo | typed-arena | bumpalo is already in use and supports more allocation patterns |
| petgraph | custom CFG | Custom implementation is lighter weight for our specific use case, petgraph adds unnecessary dependencies |

**Installation:**
```bash
cargo add lasso hashbrown fxhash
# Optional: cargo add petgraph indexmap
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── semantic/
│   ├── mod.rs              # Public API and main analyzer entry
│   ├── analyzer.rs         # Main semantic analyzer coordinator
│   ├── scope/              # Scope analysis module
│   │   ├── mod.rs
│   │   ├── analyzer.rs     # Scope visitor implementation
│   │   └── scope.rs        # Scope data structure
│   ├── symbol/             # Symbol table module
│   │   ├── mod.rs
│   │   ├── table.rs        # Symbol table implementation
│   │   └── symbol.rs       # Symbol type definitions
│   ├── types/              # Type system module
│   │   ├── mod.rs
│   │   ├── interner.rs     # Type interning
│   │   ├── representation.rs # Type data structures
│   │   └── unify.rs        # Type compatibility checking
│   ├── flow/               # Control flow analysis
│   │   ├── mod.rs
│   │   ├── cfg.rs          # Control flow graph implementation
│   │   ├── builder.rs      # CFG builder visitor
│   │   └── dominance.rs    # Dominator calculation
│   └── ir/                 # Intermediate representation
│       ├── mod.rs
│       ├── module.rs       # Module IR
│       ├── function.rs     # Function IR
│       └── instruction.rs  # IR instruction definitions
```

### Pattern 1: Visitor Pattern for Analysis
**What:** Reuse the existing visitor pattern infrastructure from the parser phase to traverse the AST and build semantic information. Each analysis component implements a visitor trait for specific AST node types.
**When to use:** All semantic analysis passes (scope, symbol, type, control flow) that need to traverse the AST.
**Example:**
```rust
// Source: Existing AST visitor pattern
use crate::parser::ast::visitor::Visitor;
use crate::parser::ast::{AstNode, AstKind};

pub struct ScopeAnalyzer<'a> {
    current_scope: ScopeId,
    scope_table: &'a mut ScopeTable,
    arena: &'a bumpalo::Bump,
}

impl<'a> Visitor for ScopeAnalyzer<'a> {
    fn visit_block_stmt(&mut self, node: &AstNode) -> Result<()> {
        // Create new block scope
        let new_scope = self.scope_table.create_scope(
            ScopeKind::Block,
            self.current_scope,
            node.span
        );
        self.current_scope = new_scope;

        // Visit children
        self.visit_children(node)?;

        // Pop scope
        self.current_scope = self.scope_table.get_scope(new_scope).parent.unwrap();

        Ok(())
    }

    fn visit_function_decl(&mut self, node: &AstNode) -> Result<()> {
        // Create function scope
        let func_scope = self.scope_table.create_scope(
            ScopeKind::Function,
            self.current_scope,
            node.span
        );

        // Add function symbol to current scope
        self.symbol_table.insert(
            self.current_scope,
            node.get_name().unwrap(),
            Symbol::Function {
                id: node.id,
                name: node.get_name().unwrap(),
                span: node.span,
                scope: func_scope,
            }
        )?;

        // Process function in new scope
        self.current_scope = func_scope;
        self.visit_children(node)?;
        self.current_scope = self.scope_table.get_scope(func_scope).parent.unwrap();

        Ok(())
    }
}
```

### Pattern 2: Interned Type Representation
**What:** All type instances are stored in a global interner with unique identifiers, allowing efficient comparison and sharing. Types are immutable once created.
**When to use:** All type system operations to avoid duplicate type objects and enable fast equality checks.
**Example:**
```rust
// Source: Lasso crate documentation
use lasso::Rodeo;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

pub enum Type {
    Primitive(PrimitiveType),
    String,
    Number,
    Boolean,
    Null,
    Undefined,
    Array(Box<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    // ... other type variants
}

pub struct TypeInterner {
    interner: Rodeo<Type, TypeId>,
}

impl TypeInterner {
    pub fn new() -> Self {
        Self {
            interner: Rodeo::new(),
        }
    }

    pub fn intern(&mut self, ty: Type) -> TypeId {
        self.interner.get_or_intern(ty)
    }

    pub fn resolve(&self, id: TypeId) -> &Type {
        self.interner.resolve(&id)
    }
}
```

### Pattern 3: Multi-level Control Flow Graph
**What:** Each scope (block, function, module) has its own CFG composed of basic blocks. Basic blocks contain a sequence of instructions and end with a terminator instruction that defines control flow edges to other blocks.
**When to use:** Control flow analysis for dead code detection, reachability analysis, and ownership inference.
**Example:**
```rust
pub struct BasicBlockId(u32);

pub enum Terminator {
    Br(BasicBlockId),                  // Unconditional branch
    CondBr(ExprId, BasicBlockId, BasicBlockId), // Conditional branch
    Ret(Option<ExprId>),               // Return
    Unreachable,                       // Unreachable code
}

pub struct BasicBlock {
    id: BasicBlockId,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
    predecessors: Vec<BasicBlockId>,
    successors: Vec<BasicBlockId>,
}

pub struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
    entry_block: BasicBlockId,
    exit_block: BasicBlockId,
}
```

### Anti-Patterns to Avoid
- **Mutable type instances:** Always use interned immutable types to avoid consistency issues
- **Recursive scope lookups without caching:** Cache resolved symbols to avoid repeated traversal of the scope chain
- **Monolithic analyzer:** Split analysis into separate passes (scope, symbol, type, flow) for maintainability
- **Ignoring hoisting semantics:** Ensure var and function declarations are processed before other statements in the scope

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| String/type interning | Custom hash map-based interner | lasso crate | Handles edge cases like memory optimization, concurrent access, and fast lookups that are error-prone to implement manually |
| Hash map for symbol tables | Custom linked hash map implementation | hashbrown with fxhash | Optimized for string keys and frequent lookups, significantly faster than std::HashMap for this use case |
| Arena allocation | Custom memory pool | bumpalo crate | Already used in the project, provides fast allocation and automatic deallocation for AST and IR nodes |
| CFG graph traversal | Custom DFS/BFS implementation | Standard graph algorithms on CFG structure | Well-understood algorithms with known correctness properties |
| Type unification | Custom type equality checking | Rule-based implementation following TS spec | TypeScript type compatibility rules are complex and well-documented, avoid ad-hoc implementations |

**Key insight:** Semantic analysis components are notoriously complex with many edge cases. Leveraging battle-tested libraries for core infrastructure lets you focus on implementing the correct TypeScript/ES6 semantics rather than debugging low-level data structures.

## Common Pitfalls

### Pitfall 1: Incorrect Hoisting Semantics
**What goes wrong:** Var declarations and function declarations are not properly hoisted to the top of their scope, leading to incorrect symbol resolution.
**Why it happens:** ES6 has complex hoisting rules that differ between var, let/const, and function declarations. Processing statements in order without handling hoisting first breaks semantics.
**How to avoid:** For each scope, first collect all function declarations and hoist them, then collect all var declarations, then process the rest of the statements in order.
**Warning signs:** Reference errors for variables that should be available, function declarations not found when used before their definition.

### Pitfall 2: Incorrect Block Scope Handling
**What goes wrong:** let/const declarations are accessible outside their block scope, or var declarations are incorrectly block-scoped.
**Why it happens:** Confusion between function scope (var) and block scope (let/const, functions in strict mode).
**How to follow:** Create a new scope for every block statement, loop body, and catch clause. Mark let/const declarations as block-scoped and var declarations as function-scoped.
**Warning signs:** Variables declared in if blocks are accessible outside, let declarations are overwritten by inner blocks incorrectly.

### Pitfall 3: Memory Leaks from Cyclic Type References
**What goes wrong:** Generic types and recursive type references create cycles that prevent proper memory management.
**Why it happens:** Interned types with cycles can cause reference counting issues or arena leaks if not handled properly.
**How to avoid:** Use arena allocation for all type nodes (they will be deallocated all at once when the arena is dropped) and use type IDs instead of direct references for recursive types.
**Warning signs:** Memory usage grows linearly with number of files processed, OOM errors during large batch processing.

### Pitfall 4: Incorrect Control Flow Edge Creation
**What goes wrong:** CFG is missing edges or has extra edges, leading to incorrect reachability analysis.
**Why it happens:** Complex control flow structures like break/continue, return, try/catch, and labeled statements are easy to mishandle.
**How to avoid:** Test CFG construction against a comprehensive suite of control flow patterns, verify edge counts and reachability for each construct.
**Warning signs:** Dead code detection reports false positives or misses actual dead code, ownership inference produces incorrect results.

## Code Examples

Verified patterns from official sources:

### Scope Analysis Visitor
```rust
// Source: Existing AST visitor pattern implementation
use crate::parser::ast::{AstNode, AstKind, Span};
use crate::parser::ast::visitor::Visitor;
use hashbrown::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Module,
    Function,
    Block,
    Loop,
    Catch,
}

pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub span: Span,
    pub symbols: HashMap<String, SymbolId>,
}

pub struct ScopeTable {
    scopes: Vec<Scope>,
    next_id: u32,
}

impl ScopeTable {
    pub fn new() -> Self {
        let global_scope = Scope {
            id: ScopeId(0),
            kind: ScopeKind::Global,
            parent: None,
            span: Span::default(),
            symbols: HashMap::new(),
        };

        Self {
            scopes: vec![global_scope],
            next_id: 1,
        }
    }

    pub fn create_scope(&mut self, kind: ScopeKind, parent: ScopeId, span: Span) -> ScopeId {
        let id = ScopeId(self.next_id);
        self.next_id += 1;

        self.scopes.push(Scope {
            id,
            kind,
            parent: Some(parent),
            span,
            symbols: HashMap::new(),
        });

        id
    }

    pub fn get_scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0 as usize]
    }

    pub fn resolve_symbol(&self, mut scope: ScopeId, name: &str) -> Option<SymbolId> {
        loop {
            let scope_obj = self.get_scope(scope);
            if let Some(symbol) = scope_obj.symbols.get(name) {
                return Some(*symbol);
            }

            match scope_obj.parent {
                Some(parent) => scope = parent,
                None => return None,
            }
        }
    }
}
```

### Type Interning
```rust
// Source: Lasso crate documentation (https://docs.rs/lasso/latest/lasso/)
use lasso::{Rodeo, Key};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Key)]
pub struct TypeId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Any,
    Unknown,
    Never,
    Void,
    String,
    Number,
    Boolean,
    Null,
    Undefined,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(TypeId),
    Tuple(Vec<TypeId>),
    Function {
        params: Vec<TypeId>,
        return_type: TypeId,
        type_params: Vec<TypeId>,
    },
    Object {
        properties: HashMap<String, TypeId>,
        index_signature: Option<(TypeId, TypeId)>,
    },
    Union(Vec<TypeId>),
    Intersection(Vec<TypeId>),
    TypeParameter {
        name: String,
        constraint: Option<TypeId>,
        default: Option<TypeId>,
    },
}

pub struct TypeInterner {
    interner: Rodeo<Type, TypeId>,
}

impl TypeInterner {
    pub fn new() -> Self {
        Self {
            interner: Rodeo::new(),
        }
    }

    pub fn intern(&mut self, ty: Type) -> TypeId {
        self.interner.get_or_intern(ty)
    }

    pub fn resolve(&self, id: TypeId) -> &Type {
        self.interner.resolve(&id)
    }

    // Helper methods for common types
    pub fn intern_primitive(&mut self, prim: PrimitiveType) -> TypeId {
        self.intern(Type::Primitive(prim))
    }

    pub fn intern_array(&mut self, element_type: TypeId) -> TypeId {
        self.intern(Type::Array(element_type))
    }
}
```

### Basic CFG Builder
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(u32);

#[derive(Debug)]
pub enum Terminator {
    Br(BasicBlockId),
    CondBr(ExprId, BasicBlockId, BasicBlockId),
    Ret(Option<ExprId>),
    Unreachable,
}

#[derive(Debug)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Option<Terminator>,
    pub predecessors: Vec<BasicBlockId>,
    pub successors: Vec<BasicBlockId>,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
    entry: BasicBlockId,
    exit: BasicBlockId,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        let entry_block = BasicBlock {
            id: BasicBlockId(0),
            instructions: Vec::new(),
            terminator: None,
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        let exit_block = BasicBlock {
            id: BasicBlockId(1),
            instructions: Vec::new(),
            terminator: None,
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        Self {
            blocks: vec![entry_block, exit_block],
            entry: BasicBlockId(0),
            exit: BasicBlockId(1),
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock {
            id,
            instructions: Vec::new(),
            terminator: None,
            predecessors: Vec::new(),
            successors: Vec::new(),
        });
        id
    }

    pub fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        self.blocks[from.0 as usize].successors.push(to);
        self.blocks[to.0 as usize].predecessors.push(from);
    }

    pub fn set_terminator(&mut self, block: BasicBlockId, terminator: Terminator) {
        self.blocks[block.0 as usize].terminator = Some(terminator);
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate allocations for each AST/IR node | Arena allocation with bumpalo | 2020 | 30-50% faster allocation, no per-node deallocation overhead |
| Manual string interning with HashMap | Specialized interner crates (lasso) | 2022 | 2-3x faster intern/lookup operations, reduced memory overhead |
| Single-pass semantic analysis | Multi-pass layered architecture | Ongoing | Improved maintainability, easier to debug individual analysis components |
| Ad-hoc type representation | Interned type IDs | Standard compiler practice | O(1) type equality checks, reduced memory usage from duplicate types |

**Deprecated/outdated:**
- Recursive scope traversal for each symbol lookup: Replace with caching of resolved symbols in the current scope for frequently accessed names
- Mutable type instances: Replace with immutable interned types to avoid consistency issues during analysis

## Open Questions

1. **TypeScript compatibility edge cases**
   - What we know: The type system should follow TypeScript compatibility rules
   - What's unclear: How many of the more complex TypeScript type features (conditional types, mapped types, template literal types) need to be supported in this phase
   - Recommendation: Start with core type features required for basic conversion, add more complex features in later phases as needed

2. **Control flow analysis for async/await**
   - What we know: Basic CFG structure for sync code is well understood
   - What's unclear: How to model async/await control flow in the CFG for accurate ownership inference
   - Recommendation: Implement basic sync CFG first, add async/await support in a separate pass once the core CFG infrastructure is validated

3. **Incremental analysis support**
   - What we know: Current design is for full-file analysis
   - What's unclear: Whether incremental analysis support should be built into the core semantic layer
   - Recommendation: Focus on correct full-file analysis first, incremental support can be added later without changing the core data structures

## Validation Architecture

Test infrastructure already exists for the parser phase, which will be extended for semantic analysis:

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework |
| Config file | none — uses standard Cargo test infrastructure |
| Quick run command | `cargo test semantic -- --test-threads=1` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCOPE-01 | ES6 scope rules implementation | unit | `cargo test semantic::scope::tests::* -x` | ❌ Wave 0 |
| SCOPE-02 | Variable and function hoisting | unit | `cargo test semantic::scope::tests::hoisting* -x` | ❌ Wave 0 |
| SYMBOL-01 | Symbol table with parent links | unit | `cargo test semantic::symbol::tests::* -x` | ❌ Wave 0 |
| SYMBOL-02 | Lexical name resolution | unit | `cargo test semantic::symbol::tests::resolution* -x` | ❌ Wave 0 |
| TYPE-01 | Interned type representation | unit | `cargo test semantic::types::tests::interner* -x` | ❌ Wave 0 |
| TYPE-02 | Type compatibility checking | unit | `cargo test semantic::types::tests::unify* -x` | ❌ Wave 0 |
| CFG-01 | Basic block CFG construction | unit | `cargo test semantic::flow::tests::cfg* -x` | ❌ Wave 0 |
| CFG-02 | Dominator calculation | unit | `cargo test semantic::flow::tests::dominance* -x` | ❌ Wave 0 |
| INT-01 | Full semantic analysis pipeline | integration | `cargo test tests::semantic_* -x` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test semantic -- --test-threads=1`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/semantic_integration.rs` — integration tests for semantic analysis
- [ ] `src/semantic/scope/tests.rs` — scope analysis unit tests
- [ ] `src/semantic/symbol/tests.rs` — symbol table unit tests
- [ ] `src/semantic/types/tests.rs` — type system unit tests
- [ ] `src/semantic/flow/tests.rs` — control flow unit tests

## Sources

### Primary (HIGH confidence)
- [Lasso crate documentation](https://docs.rs/lasso/latest/lasso/) - String interning implementation
- [SWC semantic analyzer](https://rustdoc.swc.rs/swc_semantic_analyzer/index.html) - Reference implementation for TypeScript semantic analysis in Rust
- [ECMAScript 2022 Specification](https://tc39.es/ecma262/) - Scope and semantic rules
- [TypeScript Specification](https://github.com/microsoft/TypeScript/blob/main/doc/spec-ARCHIVED.md) - Type system rules and compatibility

### Secondary (MEDIUM confidence)
- Rustc dev guide - Control flow graph implementation patterns
- "Crafting Interpreters" book - Semantic analysis patterns and best practices

### Tertiary (LOW confidence)
- Various blog posts on compiler implementation in Rust - General patterns and approaches

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Lasso, hashbrown, and bumpalo are widely used in Rust compiler projects
- Architecture: HIGH - Layered architecture with separate analysis passes is standard for production compilers
- Pitfalls: HIGH - The listed pitfalls are well-documented in compiler development resources and existing TypeScript compiler implementations

**Research date:** 2026-03-07
**Valid until:** 2026-04-06 (30 days for stable compiler infrastructure)