# Phase 04 Planning Summary

**Phase:** 04-refactor
**Planning Date:** 2026-03-08
**Status:** Complete
**Total Waves:** 3
**Estimated Duration:** 8 days

## Overview

Phase 04 implements the Semantic Refactoring Core Layer, which provides the foundation for analyzing reference relationships, inferring ownership, and applying semantic transformations to the IR. This phase is critical for enabling accurate TypeScript to Rust conversion, as Rust's ownership and borrowing semantics require deep semantic analysis.

## Wave Breakdown

### Wave 1: AnalysisContext and Analyzer Traits (2 days)

**Objective:** Build foundational infrastructure for all semantic refactoring components.

**Key Deliverables:**
- `AnalysisContext` struct that holds:
  - SemanticModule from Phase 03
  - Results map: FxHashMap<(NodeId, TypeId), Box<dyn AnalysisResult>>
  - IR version tracking for cache invalidation
  - Cache enabled flag for incremental analysis
- `SemanticAnalyzer<R: AnalysisResult>` trait with:
  - `analyze(&self, ctx: &mut AnalysisContext) -> Result<()>`
  - `name(&self) -> &'static str`
- Module structure under `src/semantic/refactor/`
- Comprehensive tests for context and traits

**Key Decisions:**
- Use FxHashMap for performance-critical results map
- Lazy cache invalidation (only when explicitly called)
- IR version tracking via u64 counter
- Object-safe AnalysisResult trait for storing different result types

### Wave 2: RRA and Dataflow Framework (3 days)

**Objective:** Implement Reference Relation Analysis and generic dataflow framework.

**Key Deliverables:**
- `RefGraph` with adjacency list representation:
  - `successors: Vec<Vec<Edge>>` and `predecessors: Vec<Vec<Edge>>`
  - O(1) edge iteration, optimal for sparse graphs
  - `RefType` enum: Ref, RefMut, Move, Read, Write
- `UsageInfo` tracking: use_count, live_blocks, first_use, last_use
- Dataflow framework with:
  - `Analysis` trait (Domain, START_STATE, DIRECTION, apply)
  - `JoinSemiLattice` trait with join operation
  - `DataflowEngine` with fixpoint iteration
  - Forward and backward direction support
- `RRAAnalyzer` implementing SemanticAnalyzer:
  - Builds reference graph from module
  - Computes liveness using dataflow
  - Generates inter-procedural summaries
- Comprehensive tests for RRA and dataflow

**Key Decisions:**
- Adjacency list (Vec<Vec<Edge>>) for sparse graph efficiency
- Inter-procedural analysis with instance-based summaries
- Backward dataflow for liveness (propagate from uses)
- Lattice join is union for BitSet (live(A ∪ B) = live(A) ∪ live(B))
- max_iterations = 100 for fixed-point iteration

### Wave 3: OA, SR, and Derivation Algorithms (3 days)

**Objective:** Implement ownership annotator, semantic rewriter, and derivation algorithms.

**Key Deliverables:**
- `OwnershipAnnotator` implementing SemanticAnalyzer:
  - Infers ownership from RRA reference analysis
  - Applies manual hints for edge cases
  - Marks ambiguous cases for manual intervention
  - Rust-style categories: Owned, Ref, RefMut, Ambiguous
- `SemanticRewriter<R: Rewriter>` with:
  - Visitor pattern for IR traversal
  - Fixed-point iteration for transformation convergence
  - Immutable IR copying (no in-place mutation)
  - Transformation history tracking (diffs only)
- `Rewriter` trait with rewrite methods for Function, Instruction, BasicBlock
- `RewriteRule` trait for extensible transformation rules
- Derivation algorithms:
  - `OwnershipDeriver`: Infer Owned vs Ref vs RefMut
  - `LifetimeDeriver`: Compute lifetime constraints from CFG
  - `MutabilityDeriver`: Determine mutability from usage patterns
- Comprehensive tests for OA, SR, and derivation

**Key Decisions:**
- Ownership annotations inline in IR nodes
- Manual hints override automatic inference
- Ambiguous ownership requires manual intervention
- Transformation history stores diffs (not full IR copies)
- Sequential pipeline: RRA → OA → derivation
- All results stored in AnalysisContext.results map

## Locked Decisions (from 04-CONTEXT.md)

### RRA Design
- Reference graph: Adjacency list with edge list (from_id, to_id, type)
- Reference types: Typed edges (borrow, own, read, write as enum)
- Analysis scope: Inter-procedural with summaries
- Usage tracking: Full usage (counts + live ranges)

### OA Design
- Storage: Inline in IR nodes
- Determination: Mixed (automatic inference + manual hints)
- Conflict handling: Track ambiguity
- Format: Rust-style categories (Owned, Ref, RefMut)

### SR Design
- IR modification: Immutable copy
- Rewrite application: Fixed-point iteration
- Rewrite structure: Visitor-based
- Transformation tracking: Full history

### Derivation Algorithms
- Order: Sequential pipeline (RRA → OA → derivation)
- Storage: Context results map (keyed by (NodeId, TypeId))
- Incremental: Full incremental support
- Cache: Lazy cache, invalidate on IR changes

## Technical Architecture

### Module Structure
```
src/semantic/refactor/
├── mod.rs              # Public API
├── context/            # AnalysisContext and cache management
│   ├── mod.rs
│   └── context.rs
├── traits/             # Analyzer interfaces
│   ├── mod.rs
│   └── analyzer.rs
├── dataflow/           # Dataflow framework
│   ├── mod.rs
│   └── framework.rs
├── rra/                # Reference Relation Analysis
│   ├── mod.rs
│   ├── graph.rs        # RefGraph, RefType, UsageInfo
│   ├── analyzer.rs     # RRAAnalyzer
│   └── summary.rs      # Inter-procedural summaries
├── oa/                 # Ownership Annotator
│   ├── mod.rs
│   ├── annotator.rs    # OwnershipAnnotator
│   └── categories.rs   # OwnershipCategory, OwnershipAnnotation
├── sr/                 # Semantic Rewriter
│   ├── mod.rs
│   ├── rewriter.rs     # Rewriter trait, SemanticRewriter
│   ├── rules.rs        # RewriteRule, RuleRegistry
│   └── history.rs      # TransformRecord, transformation history
└── derive/             # Derivation algorithms
    ├── mod.rs
    ├── ownership.rs    # OwnershipDeriver
    ├── lifetime.rs     # LifetimeDeriver
    └── mutability.rs   # MutabilityDeriver
```

### Data Flow

```
Phase 03 (SemanticModule)
        ↓
Wave 1: AnalysisContext (cache management)
        ↓
Wave 2: RRAAnalyzer (reference graph + liveness)
        ↓
Wave 3: OwnershipAnnotator (ownership inference)
        ↓
Wave 3: SemanticRewriter (IR transformations)
        ↓
Wave 3: Derivation Algorithms (ownership/lifetime/mutability)
        ↓
Phase 05 (Rust AST generation)
```

## Dependencies

### External Dependencies
- None (uses only existing project libraries)

### Internal Dependencies
- Phase 03: SemanticModule, IR, CFG, types, symbols, scopes
- Wave 1: AnalysisContext, SemanticAnalyzer trait
- Wave 2: RRA (ReferenceAnalysis), dataflow framework

## Integration Points

### Phase 03 → Phase 04
- SemanticModule provides input for analysis
- IR is annotated with ownership information
- CFG used for liveness and lifetime analysis
- Types and symbols referenced by analysis results

### Phase 04 → Phase 05
- Ownership annotations used for Rust code generation
- Lifetime constraints for borrow checking
- Mutability information for mut vs let
- Clone requirements for manual intervention

## Testing Strategy

Each wave includes comprehensive tests:

### Wave 1 Tests
- AnalysisContext creation and mutation
- Trait implementation for mock analyzers
- Cache invalidation behavior
- Results map CRUD operations

### Wave 2 Tests
- Graph construction and edge operations
- Reference type enum functionality
- Usage tracking and liveness
- RRA analyzer execution
- Summary generation
- Dataflow convergence

### Wave 3 Tests
- Ownership categories and annotation
- OA inference and manual hints
- Semantic rewriter transformations
- Fixed-point iteration convergence
- Transformation history tracking
- Ownership derivation algorithm
- Lifetime derivation algorithm
- Mutability derivation algorithm

**Target Coverage:** > 80% for all modules

## Risks and Mitigations

### Risk 1: Fixed-Point Iteration Never Converges
**Mitigation:**
- Ensure transfer functions are monotonic
- Implement top value that absorbs all joins
- Always set max_iterations (100) and return error if exceeded

### Risk 2: Reference Graph Memory Explosion
**Mitigation:**
- Only track references between symbols (SymbolId), not intermediate values
- Use typed edges to distinguish direct from indirect references
- Prune edges after scope exits

### Risk 3: Transformation History Memory Leak
**Mitigation:**
- Store only diffs, not full IR copies
- Make history optional (disabled in production)
- Use arena allocation per iteration

### Risk 4: Inter-Procedural Summary Loss of Precision
**Mitigation:**
- Include specific reference types in summaries
- Use type-based summaries when possible
- Fall back to conservative analysis only when necessary

## Success Criteria

Phase 04 is complete when:

1. All 3 waves implemented and tested
2. All tests pass with > 80% coverage
3. RRA correctly builds reference graphs
4. OA infers ownership with > 90% accuracy on test cases
5. SR applies transformations without IR mutation
6. Derivation algorithms produce valid Rust-compatible results
7. Cache invalidation works correctly
8. Transformation history tracks all changes
9. All components integrate through AnalysisContext
10. Documentation complete for all public APIs

## Next Steps

After Phase 04 planning is complete:

1. **Execute Wave 1** (04-01-PLAN.md):
   - Create module structure
   - Implement AnalysisContext
   - Define SemanticAnalyzer trait
   - Write comprehensive tests

2. **Execute Wave 2** (04-02-PLAN.md):
   - Implement reference graph
   - Build dataflow framework
   - Create RRA analyzer
   - Write tests for RRA and dataflow

3. **Execute Wave 3** (04-03-PLAN.md):
   - Extend IR with ownership annotations
   - Implement ownership annotator
   - Build semantic rewriter
   - Create derivation algorithms
   - Write comprehensive tests

4. **Integration Testing**:
   - Run full analysis pipeline on test modules
   - Verify results are correct
   - Validate cache invalidation
   - Confirm transformation history works

5. **Phase 05 Planning**:
   - Plan Rust AST layer
   - Design Rust code generation
   - Plan integration with Phase 04 results

## Notes

- All locked decisions from 04-CONTEXT.md must be honored
- Claude has discretion for implementation details (data structure choices, algorithm specifics)
- Use existing project libraries (hashbrown, fxhash, bumpalo)
- Follow project coding style (immutability, small files, comprehensive error handling)
- All public APIs must have documentation
- Tests must follow TDD approach (write tests first)
