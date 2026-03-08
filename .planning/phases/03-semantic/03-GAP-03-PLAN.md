---
wave: 2
depends_on: ["03-GAP-01", "03-GAP-02"]
files_modified:
  - src/semantic/analyzer.rs
  - src/semantic/flow/builder.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-03: CFG Integration into Main Analyzer

## Overview
Integrate the CFG builder into the main semantic analyzer's `build_cfgs_for_functions()` method to actually build control flow graphs for all functions in the module.

## Context
The CFGBuilder is fully implemented but not being called. The `build_cfgs_for_functions()` method in `SemanticAnalyzer` is a stub that just returns `Ok(())` without building any CFGs. This prevents the semantic module from containing complete control flow information.

## Goal
Implement full CFG construction by traversing the AST, finding all function definitions, and using CFGBuilder to construct CFGs for each function.

## Requirements

### Must-Haves
1. `build_cfgs_for_functions()` finds all function symbols in the module
2. CFGBuilder is instantiated and called for each function definition
3. Built CFGs are added to `SemanticModule.functions` vector
4. CFGs include proper entry and exit blocks

### Should-Haves
1. Handle different function types: declarations, expressions, arrow functions
2. Extract function parameters from symbol table
3. Map function names to their corresponding CFGs

### Nice-to-Haves
1. Optimize by skipping duplicate function processing
2. Add CFG validation to ensure correctness

## Tasks

### Task 1: Find all function definitions
Implement AST traversal to locate all function definitions:
- Traverse the AST to find `NodeKind::FunctionDeclaration`
- Also find `NodeKind::FunctionExpression` and `NodeKind::ArrowFunction`
- Collect function information: name, span, AST node reference

**Implementation details:**
- Create a helper struct `FunctionInfo` to store collected data
- Implement a simple visitor or manual traversal to collect functions
- Use `node.kind()` to identify function nodes
- Store results in a `Vec<FunctionInfo>` for processing

### Task 2: Create CFG for each function
Implement CFG construction loop:
- For each function found, create a new `Function` object
- Extract function symbol information from the symbol table
- Initialize entry and exit blocks
- Instantiate CFGBuilder with the function
- Traverse function body with CFGBuilder

**Implementation details:**
- For each function in collected list:
  - Look up function symbol in `module.symbols`
  - Create `Function` with appropriate name and ID
  - Extract parameter types from symbol (if available)
  - Call `CFGBuilder::new(&mut function)`
  - Call `cfg_builder.visit_node(function_body_node)`
  - Add function to `module.functions` vector

### Task 3: Wire CFGBuilder visitor to function body
Ensure CFGBuilder correctly processes function bodies:
- CFGBuilder should visit only the function body, not the entire AST
- Handle parameter declarations in entry block
- Create proper entry block with parameter loads
- Create exit block with return instructions

**Implementation details:**
- Extract function body node from the AST structure
- For function declarations, body is typically a child node
- Ensure CFGBuilder's state is initialized correctly for each function
- Handle edge cases: empty functions, functions with single expressions

### Task 4: Add function parameters to CFG
Implement parameter handling in CFG entry block:
- Create `Alloca` instructions for each parameter
- Store parameter values in allocated slots
- Add parameter symbols to CFG's symbol mapping

**Implementation details:**
- Extract parameter list from function symbol or AST
- For each parameter:
  - Create an `Alloca` instruction in entry block
  - Store the parameter value in the allocated slot
  - Map parameter name to the allocated value
- Ensure parameter types are correctly set

### Task 5: Handle function expressions and arrow functions
Extend CFG building for anonymous and arrow functions:
- Generate synthetic names for anonymous functions
- Handle arrow function shorthand syntax
- Process arrow function parameters correctly

**Implementation details:**
- For anonymous functions, use generated names (e.g., `anon_0`, `anon_1`)
- Extract parameters from arrow function AST structure
- Handle arrow function bodies that are single expressions vs block statements
- Ensure CFG structure is consistent across all function types

## Verification Criteria

1. All existing tests pass
2. New integration tests verify:
   - Function declarations produce CFGs in module.functions
   - Function expressions produce CFGs
   - Arrow functions produce CFGs
   - CFGs have proper entry and exit blocks
   - CFGs contain expected basic blocks and edges
3. No stub implementation remains in `build_cfgs_for_functions()`
4. Module.functions is non-empty for modules with function definitions

## Success Metrics

- All function definitions in test modules have corresponding CFGs
- CFGs contain at minimum: entry block, exit block, and body blocks
- Verification gap "CFGs are built and integrated in main analyzer" is closed
- Integration test coverage > 80% for CFG construction

## Notes

- Need to handle the fact that CFGBuilder is a visitor that expects a function body node
- Must be careful about lifetime management - CFGBuilder takes `&mut Function`
- Consider error handling for malformed functions
- Ensure CFGs are not built for the same function multiple times

## Dependencies

- 03-GAP-01: Type unification (for type checking in CFG)
- 03-GAP-02: Type assignability (for validation)
- Existing CFGBuilder implementation
- Existing IR module structures

## Estimated Time

2-3 hours for implementation and integration testing
