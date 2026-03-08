---
wave: 2
depends_on: ["03-GAP-04"]
files_modified:
  - src/semantic/scope/analyzer.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-05: Exception Parameter Handling

## Overview
Add exception parameters to catch scopes in try-catch statements.

## Context
The `visit_try` method in `ScopeAnalyzer` creates a catch scope but does not add the exception parameter (line 392 TODO). This prevents proper error handling analysis and type checking for caught exceptions.

## Goal
Extract the exception parameter from catch clauses and add it to the catch scope with appropriate type information.

## Requirements

### Must-Haves
1. Exception parameter extracted from catch clause AST node
2. Exception parameter added to catch scope
3. Parameter has correct name and type (typically `any` or `Error`)

### Should-Haves
1. Handle type annotations on catch parameters
2. Handle optional catch parameters (TypeScript 4.0+)

### Nice-to-Haves
1. Support for catch binding patterns (destructuring)

## Tasks

### Task 1: Extract catch parameter from AST
Implement catch parameter extraction:
- Parse the catch clause node structure
- Extract the exception variable name
- Extract type annotation if present

**Implementation details:**
- In `visit_try`, examine the `catch_clause` option
- Access the catch clause's variable declaration
- Extract parameter name from the catch clause AST
- Extract type annotation if present (e.g., `catch (e: Error)`)

### Task 2: Add exception parameter to catch scope
Implement parameter symbol creation:
- Create a variable symbol for the exception parameter
- Set appropriate type (typically `any` if no annotation)
- Add symbol to the catch scope
- Set correct span for error reporting

**Implementation details:**
- After pushing catch scope:
  - If catch clause has a parameter:
    - Create symbol with `SymbolKind::Variable`
    - Set name to the exception parameter name
    - Extract type annotation if present, use `any` type otherwise
    - Intern the type and set on symbol
    - Add to catch scope using `declare_variable()` or directly to symbol table
- Remove TODO comment at line 392

### Task 3: Handle type annotations on catch parameters
Implement type annotation processing:
- Parse type annotation from catch parameter
- Convert to Type and intern
- Set as type_id on exception symbol

**Implementation details:**
- If catch parameter has type annotation (e.g., `catch (e: Error)`):
  - Use `intern_type_annotation()` to convert to TypeId
  - Set the type_id on the exception symbol
  - If no annotation, use `any` type (most common for catch parameters)
  - Consider TypeScript's built-in `Error` type

### Task 4: Handle optional catch parameters
Support catch blocks without parameters:
- TypeScript 4.0+ allows `catch { }` without a parameter
- Catch scope should still be created but without exception symbol

**Implementation details:**
- Check if catch clause has a parameter
- If no parameter, skip symbol creation
- Still create catch scope for proper scoping
- Ensure catch body is visited in correct scope

## Verification Criteria

1. All existing tests pass
2. New tests verify:
   - Catch parameters are in symbol table
   - Catch parameters have correct types
   - Catch without parameter still creates scope
   - Exception parameters are accessible in catch block
3. TODO comment at line 392 is removed
4. Exception parameters visible only within catch scope

## Success Metrics

- All try-catch statements have properly scoped exception parameters
- Verification gap "Exception handling adds catch parameters to catch scope" is closed
- Test coverage > 80% for exception parameter handling

## Notes

- Exception parameters are scoped only to the catch block
- TypeScript allows catch without parameter (catch {})
- Exception type is often `any` but can be specified
- Consider TypeScript's built-in Error type for type checking

## Dependencies

- 03-GAP-04: Function parameter handling (reuses parameter processing logic)

## Estimated Time

1-1.5 hours for implementation and testing
