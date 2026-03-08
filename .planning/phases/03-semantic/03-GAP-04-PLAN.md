---
wave: 2
depends_on: []
files_modified:
  - src/semantic/scope/analyzer.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-04: Function Parameter Handling

## Overview
Add function parameters to function scopes in `ScopeAnalyzer` for function declarations, arrow functions, and function expressions.

## Context
Multiple TODOs in `src/semantic/scope/analyzer.rs` indicate that function parameters are not being added to function scopes:
- Line 293: Function declaration parameters
- Line 441: Arrow function parameters
- Line 455: Function expression parameters

This prevents proper symbol table population and type checking for function parameters.

## Goal
Implement complete parameter handling for all function types, ensuring parameters are properly declared in function scopes with correct type information.

## Requirements

### Must-Haves
1. Function declaration parameters added to function scope
2. Arrow function parameters added to function scope
3. Function expression parameters added to function scope
4. Parameters have correct names, types, and spans

### Should-Haves
1. Handle default parameter values
2. Handle rest parameters
3. Handle destructured parameters

### Nice-to-Haves
1. Validate parameter uniqueness
2. Support parameter type annotations

## Tasks

### Task 1: Extract parameter information from AST nodes
Implement parameter extraction logic:
- Create helper function to extract parameters from function nodes
- Handle different function node kinds: FunctionDeclaration, FunctionExpression, ArrowFunction
- Return parameter information: name, type annotation, span

**Implementation details:**
- Add helper method `extract_parameters(&self, node: &AstNode) -> Vec<ParameterInfo>`
- `ParameterInfo` struct contains: name, type_annotation, span, has_default
- Extract from `NodeKind` properties for each function type
- Handle arrow function shorthand: `x => x` (no parentheses)

### Task 2: Add parameters to function declarations
Implement parameter handling in `visit_function_declaration`:
- Extract parameters from the function node
- Create parameter symbols in the function scope
- Set parameter types from type annotations
- Handle parameter hoisting rules

**Implementation details:**
- In `visit_function_declaration`, after pushing function scope:
  - Extract parameters using helper function
  - For each parameter:
    - Create symbol with `SymbolKind::Variable` or add new `SymbolKind::Parameter`
    - Extract type annotation and intern it
    - Call `declare_variable()` or similar to add to scope
    - Set correct span for parameter location
- Remove TODO comment at line 293

### Task 3: Add parameters to arrow functions
Implement parameter handling in `visit_arrow_function`:
- Extract parameters from arrow function node
- Create parameter symbols in the function scope
- Handle arrow function shorthand syntax correctly

**Implementation details:**
- In `visit_arrow_function`, after pushing function scope:
  - Extract parameters (may be single identifier or parameter list)
  - Handle shorthand: `x => x` has single identifier parameter
  - Handle list: `(x, y) => x + y` has multiple parameters
  - Create symbols for each parameter
  - Set types from type annotations if present
- Remove TODO comment at line 441

### Task 4: Add parameters to function expressions
Implement parameter handling in `visit_function_expression`:
- Extract parameters from function expression node
- Create parameter symbols in the function scope
- Handle named function expressions correctly

**Implementation details:**
- In `visit_function_expression`, after pushing function scope:
  - Extract parameters from function expression node
  - Create symbols for each parameter
  - Handle optional name binding (named function expressions)
  - If function has a name, add it to function scope as well
  - Set types from type annotations
- Remove TODO comments at lines 455-456

### Task 5: Handle parameter type annotations
Implement type annotation processing for parameters:
- Extract type annotation from parameter AST nodes
- Convert type annotation to Type
- Intern the type and associate with parameter symbol
- Handle missing type annotations (default to Unknown)

**Implementation details:**
- In parameter extraction, include type annotation information
- For each parameter with type annotation:
  - Use `intern_type_annotation()` helper to convert annotation to TypeId
  - Set the type_id field on the parameter symbol
  - If no annotation, set to `Unknown` type or leave as None

### Task 6: Handle default and rest parameters
Implement advanced parameter features:
- Default parameter values
- Rest parameters (spread operator)
- Destructured parameters (object/array patterns)

**Implementation details:**
- For default parameters:
  - Extract default value expression
  - Mark parameter as having default in symbol flags
- For rest parameters:
  - Extract rest parameter name and type
  - Mark as rest parameter in symbol
  - Type should be array of element type
- For destructured parameters:
  - Extract pattern structure
  - Create individual symbols for destructured variables
  - Set types based on pattern types

## Verification Criteria

1. All existing tests pass
2. New tests verify:
   - Function declaration parameters are in symbol table
   - Arrow function parameters are in symbol table
   - Function expression parameters are in symbol table
   - Parameters have correct types from annotations
   - Default parameters are handled
   - Rest parameters are handled
3. All TODO comments related to parameters are removed
4. Parameter symbols are accessible from function scope using `lookup_lexical()`

## Success Metrics

- All function types have complete parameter symbols
- Parameters appear in scope when queried
- Verification gaps related to function parameters are closed
- Test coverage > 85% for parameter handling

## Notes

- Function parameters are hoisted in the function scope (visible throughout)
- TypeScript allows duplicate parameter names with certain restrictions (may ignore for now)
- Type annotations on parameters are optional
- Consider how to handle optional parameters (`?` syntax)

## Dependencies

None - standalone improvement to scope analysis.

## Estimated Time

2-3 hours for implementation and testing

## Risks

- Complex parameter structures (nested destructuring) may be challenging
- Edge cases in arrow function shorthand syntax
- Interaction with existing scope management code
