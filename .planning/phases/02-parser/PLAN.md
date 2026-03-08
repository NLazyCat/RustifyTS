# Phase 02: Parser Layer - Implementation Plan

**Phase:** 02
**Status:** Ready for execution
**Generated:** 2026-03-01

## Overview

Phase 02 implements the TypeScript parser layer that converts TypeScript source code into a unified AST-IR representation. The parser uses Deno as a subprocess backend (communicating via MessagePack), implements an arena allocator for efficient AST node storage, provides a visitor pattern for AST traversal, and uses a three-layer error strategy (thiserror + anyhow + miette).

## Phase Goal

Build a complete TypeScript parsing infrastructure that can:
1. Parse TypeScript source code via Deno subprocess backend
2. Store AST nodes efficiently using arena allocation
3. Support AST traversal via visitor pattern
4. Track source locations with span and line map
5. Provide comprehensive error handling with diagnostics

## Deliverables

- `src/parser/mod.rs` - Module interface
- `src/parser/backend/mod.rs` - Backend module
- `src/parser/backend/trait.rs` - ParserBackend trait
- `src/parser/backend/deno.rs` - Deno subprocess implementation
- `src/parser/ast/mod.rs` - AST module
- `src/parser/ast/types.rs` - AST type definitions (Statement, Expression, Declaration, etc.)
- `src/parser/ast/node.rs` - AstNode and NodeKind with arena allocation
- `src/parser/ast/visitor.rs` - Visitor trait and implementations
- `src/parser/ast/span.rs` - Span and LineMap
- `src/parser/error.rs` - Error type definitions with thiserror
- `src/lib.rs` - Updated library entry point
- `deno-bridge/deno_parser.ts` - Deno bridge script for TypeScript Compiler API
- Test fixtures for integration tests

## Dependencies

- Phase 01 (Project基础设施) - should be complete
- External crates: serde, serde_json, rmp-serde, tokio, thiserror, anyhow, miette, bumpalo

## Estimated Time

3 days (6 waves)

---

## Wave 1: Project Configuration and Error Types

**Files modified:**
- `Cargo.toml` - Add dependencies
- `src/lib.rs` - Create library interface

**Files created:**
- `src/parser/error.rs` - Error type definitions

**Depends on:** None

**Autonomous:** true

### Tasks

<tasks>
<task id="w1-t1">
<description>Add required dependencies to Cargo.toml: serde, serde_json, rmp-serde, tokio, thiserror, anyhow, miette, bumpalo, clap</description>
<command>Edit Cargo.toml to add all dependencies with appropriate features</command>
<verification>cargo build succeeds without errors, cargo tree shows all dependencies</verification>
</task>

<task id="w1-t2">
<description>Create src/lib.rs with module declarations and public API stub for parser</description>
<command>Create src/lib.rs with pub mod parser and basic module stub</command>
<verification>cargo build succeeds, lib.rs compiles without errors</verification>
</task>

<task id="w1-t3">
<description>Create src/parser/error.rs with ParseError enum using thiserror, covering file I/O, Deno subprocess, serialization, and syntax errors</description>
<command>Create src/parser/error.rs with thiserror derive macros for all error variants</command>
<verification>ParseError implements std::error::Error, all variants have Display and Debug</verification>
</task>
</tasks>

### must_haves

- [ ] Cargo.toml includes all 8 dependencies with correct features
- [ ] src/lib.rs declares parser module
- [ ] src/parser/error.rs defines ParseError enum with comprehensive error variants
- [ ] Error types are derived with thiserror macro
- [ ] cargo build succeeds

---

## Wave 2: Span and Location Tracking

**Files modified:** None

**Files created:**
- `src/parser/ast/span.rs` - Span and LineMap with zero-based indexing

**Depends on:** Wave 1

**Autonomous:** true

### Tasks

<tasks>
<task id="w2-t1">
<description>Create src/parser/ast/span.rs with Span struct (start, end byte offsets), LineMap struct, and conversion methods between byte offsets and line/column</description>
<command>Create src/parser/ast/span.rs with Span::new(), LineMap::from_source(), and line_col() lookup methods</command>
<verification>Span stores zero-based byte offsets, LineMap::line_col() returns 1-based display values</verification>
</task>

<task id="w2-t2">
<description>Add span unit tests: test span creation, length calculation, empty span detection, line map construction, and line/column lookup</description>
<command>Add #[cfg(test)] module in span.rs with comprehensive unit tests</command>
<verification>All tests pass: test_span_new, test_span_len, test_span_is_empty, test_line_map_creation, test_line_col_lookup</verification>
</task>

<task id="w2-t3">
<description>Create src/parser/ast/mod.rs with module declarations for span, types, node, visitor</description>
<command>Create src/parser/ast/mod.rs with pub mod span and re-arexports</command>
<verification>src/parser/mod.rs can use parser::ast::Span and parser::ast::LineMap</verification>
</task>
</tasks>

### must_haves

- [ ] src/parser/ast/span.rs implements Span with Copy, Clone, Debug, PartialEq, Eq
- [ ] Span uses zero-based byte offsets internally
- [ ] LineMap converts byte offsets to 1-based line/column for display
- [ ] Unit tests verify span calculations and line map correctness
- [ ] All tests pass

---

## Wave 3: AST Types and Node Infrastructure

**Files modified:**
- `src/parser/ast/mod.rs` - Add types and node modules

**Files created:**
- `src/parser/ast/types.rs` - AST node type definitions (Statement, Expression, Declaration, Pattern, Type)
- `src/parser/ast/node.rs` - AstNode, NodeKind, and builder with arena allocation

**Depends on:** Wave 2

**Autonomous:** true

### Tasks

<tasks>
<task id="w3-t1">
<description>Create src/parser/ast/types.rs with categorized enums: Statement, Expression, Declaration, Pattern, Type, Literal, covering core TypeScript constructs</description>
<command>Create types.rs with enum definitions for all node kinds and their associated data structures</command>
<verification>All node types are defined, variants cover TypeScript basics (let/const, function, if/while, arithmetic, literals)</verification>
</task>

<task id="w3-t2">
<description>Create src/parser/ast/node.rs with AstNode struct containing NodeKind, Span, and children vector, plus NodeBuilder for constructing nodes in an arena</description>
<command>Create node.rs with bumpalo::Bump integration, AstNode<'a> struct, and NodeBuilder for arena-allocated nodes</command>
<verification>AstNode uses arena lifetime, NodeBuilder allocates nodes in bumpalo arena, nodes have proper lifetime annotations</verification>
</task>

<task id="w3-t3">
<description>Add AST node unit tests: test arena allocation, node building, span assignment, and structural equality</description>
<command>Add #[cfg(test)] module in node.rs with tests for arena usage and node construction</command>
<verification>Tests verify nodes are allocated in arena, spans are preserved, PartialEq works correctly</verification>
</task>

<task id="w3-t4">
<description>Update src/parser/mod.rs to declare ast module and re-export key types</description>
<command>Edit src/parser/mod.rs with pub mod ast and use re-arexports for AstNode, NodeKind, Span</command>
<verification>Parser module exposes AST types to external consumers</verification>
</task>
</tasks>

### must_haves

- [ ] src/parser/ast/types.rs defines all core TypeScript node types
- [ ] src/parser/ast/node.rs implements AstNode with arena lifetime
- [ ] bumpalo::Bump is integrated for node allocation
- [ ] NodeBuilder provides safe node construction interface
- [ ] Unit tests verify arena allocation correctness
- [ ] All tests pass

---

## Wave 4: Visitor Pattern

**Files modified:**
- `src/parser/ast/mod.rs` - Add visitor module

**Files created:**
- `src/parser/ast/visitor.rs` - Visitor trait and example implementations

**Depends on:** Wave 3

**Autonomous:** true

### Tasks

<tasks>
<task id="w4-t1">
<description>Create src/parser/ast/visitor.rs with Visitor trait defining visit methods for each node kind, and default traversal implementation</description>
<command>Create visitor.rs with trait Visitor<'a> having visit_node, and typed methods for major node kinds</command>
<verification>Visitor trait is generic over arena lifetime, default methods provide recursive traversal</verification>
</task>

<task id="w4-t2">
<description>Implement example visitor: NodeCounter that counts total nodes by traversing the AST</description>
<command>Add NodeCounter struct and Visitor impl that increments count on each node visit</command>
<verification>NodeCounter correctly counts all nodes in test AST</verification>
</task>

<task id="w4-t3">
<description>Add visitor unit tests: test default traversal, node counting, and custom visitor behavior</description>
<command>Add #[cfg(test)] module in visitor.rs with tests for visitor implementation</command>
<verification>Tests verify visitors traverse all nodes, NodeCounter works correctly</verification>
</task>
</tasks>

### must_haves

- [ ] src/parser/ast/visitor.rs defines Visitor trait
- [ ] Visitor has visit methods for all major node kinds
- [ ] Default implementation provides recursive traversal
- [ ] Example visitor (NodeCounter) demonstrates usage
- [ ] Unit tests verify visitor behavior
- [ ] All tests pass

---

## Wave 5: Deno Backend Implementation

**Files modified:**
- `src/parser/mod.rs` - Add backend module

**Files created:**
- `src/parser/backend/mod.rs` - Backend module
- `src/parser/backend/trait.rs` - ParserBackend trait
- `src/parser/backend/deno.rs` - Deno subprocess implementation with MessagePack
- `deno-bridge/deno_parser.ts` - TypeScript Compiler API bridge script

**Depends on:** Wave 3 (needs AST types)

**Autonomous:** false (requires Deno installation)

### Tasks

<tasks>
<task id="w5-t1">
<description>Create src/parser/backend/trait.rs with ParserBackend trait defining parse() and parse_file() methods</description>
<command>Create trait.rs with async parse methods that return Result<ParsedAst<'a>, ParserError></command>
<verification>ParserBackend trait is defined, methods have appropriate signatures</verification>
</task>

<task id="w5-t2">
<description>Create deno-bridge/deno_parser.ts using TypeScript Compiler API to parse source code and output AST in JSON format</description>
<command>Create TypeScript script that uses ts.createSourceFile() and serializes AST to JSON</command>
<verification>Script runs with 'deno run', outputs structured AST data for valid TypeScript code</verification>
</task>

<task id="w5-t3">
<description>Create src/parser/backend/deno.rs with DenoBackend struct and implementation using tokio::process to spawn Deno subprocess</description>
<command>Create DenoBackend with async parse method that spawns Deno, sends source via stdin, receives AST via stdout</command>
<verification>DenoBackend implements ParserBackend trait, subprocess communication works correctly</verification>
</task>

<task id="w5-t4">
<description>Implement MessagePack serialization/deserialization for parse request/response in DenoBackend</description>
<command>Add rmp-serde integration for serializing ParseRequest and deserializing ParseResponse</command>
<verification>MessagePack roundtrip works, Deno backend can send/receive data correctly</verification>
</task>

<task id="w5-t5">
<description>Create src/parser/backend/mod.rs with module declarations and re-exports</description>
<command>Create mod.rs with pub mod trait and pub mod deno, re-export ParserBackend</command>
<verification>Parser module exposes backend types via parser::backend::ParserBackend</verification>
</task>

<task id="w5-t6">
<description>Add Deno backend integration tests with TypeScript fixture files</description>
<command>Create tests/fixtures/parser/ with sample .ts files, add integration tests that parse them</command>
<verification>Integration tests successfully parse TypeScript fixtures and produce valid ASTs</verification>
</task>
</tasks>

### must_haves

- [ ] src/parser/backend/trait.rs defines ParserBackend trait
- [ ] deno-bridge/deno_parser.ts uses TypeScript Compiler API
- [ ] src/parser/backend/deno.rs implements ParserBackend with Deno subprocess
- [ ] tokio::process manages Deno subprocess lifecycle
- [ ] rmp-serde handles MessagePack serialization
- [ ] Integration tests parse TypeScript fixtures successfully
- [ ] All tests pass

---

## Wave 6: Integration and Public API

**Files modified:**
- `src/parser/mod.rs` - Complete module interface
- `src/lib.rs` - Export parser API

**Files created:**
- Tests and documentation

**Depends on:** Wave 5

**Autonomous:** true

### Tasks

<tasks>
<task id="w6-t1">
<description>Complete src/parser/mod.rs with public API: parse_source() and parse_file() convenience functions</description>
<command>Add convenience functions that use DenoBackend internally</command>
<verification>Public API functions work, return Result<AstArena<'a>, ParserError></verification>
</task>

<task id="w6-t2">
<description>Update src/lib.rs to re-export parser module with cleaner API (transmute::parse_source, etc.)</description>
<command>Edit lib.rs to use pub use parser::* and expose high-level parsing functions</command>
<verification>External code can use transmute::parse_source() and transmute::parse_file()</verification>
</task>

<task id="w6-t3">
<description>Add comprehensive integration tests: parse complex TypeScript files, verify AST structure, test error handling</description>
<command>Add tests/parser_integration.rs with tests for various TypeScript constructs</command>
<verification>All integration tests pass, AST structure is correct for various TypeScript inputs</verification>
</task>

<task id="w6-t4">
<documentation>Add module-level documentation to all public API surfaces with examples</documentation>
<command>Add /// comments to parse_source(), parse_file(), ParserBackend, and AST types</command>
<verification>rustdoc generates documentation without warnings, examples compile</verification>
</task>

<task id="w6-t5">
<description>Run full test suite: cargo test --lib</description>
<command>Execute cargo test to verify all tests pass</command>
<verification>All tests pass, no warnings, coverage is acceptable</verification>
</task>
</tasks>

### must_haves

- [ ] src/parser/mod.rs provides public convenience functions
- [ ] src/lib.rs exports parser API at crate level
- [ ] Integration tests verify end-to-end parsing
- [ ] Documentation covers all public APIs with examples
- [ ] All tests pass (unit + integration)
- [ ] cargo test succeeds without warnings

---

## Verification Criteria

### Phase Completion Checklist

- [ ] All 6 waves completed successfully
- [ ] All deliverables created and committed
- [ ] Cargo.toml includes all dependencies
- [ ] All modules compile without errors
- [ ] All unit tests pass
- [ ] All integration tests pass
-- [ ] Deno bridge script parses TypeScript correctly
- [ ] Error handling works across all layers
- [ ] Public API is documented and usable

### Test Coverage

- Unit tests: Span, arena allocation, visitor pattern, error types
- Integration tests: Deno subprocess communication, TypeScript parsing
- Manual verification: Parsing of representative TypeScript samples

### Quality Checks

- No unsafe code (unless justified)
- No unwrap() calls in production code
- Proper error handling with context
- Immutable AST after construction
- Zero-based indexing consistent
- Thread-safe where applicable (tokio async)

---

## Open Questions Resolved by Implementation

1. **Deno TypeScript AST format**: Resolved by creating deno_parser.ts that serializes TypeScript Compiler API AST to JSON/MessagePack
2. **MessagePack vs JSON**: Implementation can start with JSON for easier debugging, switch to MessagePack via feature flag if needed
3. **Error recovery granularity**: Implement basic recovery (skip tokens) for simple syntax errors, abort on structural errors
4. **Deno process management**: Single persistent process with configurable timeout for Phase 02

---

## Notes

- Deno installation is required for Wave 5 tests to pass
- Phase 02 creates the foundation for Phase 03 (Semantic Analysis Layer)
- AST design is intentionally extensible for future TypeScript feature additions
- Arena allocation provides significant memory efficiency for large TypeScript files

---

*Phase: 02-parser*
*Plan generated: 2026-03-01*
