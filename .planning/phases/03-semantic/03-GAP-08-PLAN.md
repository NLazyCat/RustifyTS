---
wave: 3
depends_on: []
files_modified:
  - src/semantic/types/resolver.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-08: Type Resolution Error Collection

## Overview
Implement proper error collection in `TypeResolver` instead of silently ignoring type resolution errors.

## Context
The `visit_variable_statement` method in `TypeResolver` has a TODO (line 403) indicating that type resolution errors are being silently ignored. This prevents proper error reporting to users when type resolution fails.

## Goal
Implement comprehensive error collection in `TypeResolver` with detailed error information and proper error propagation.

## Requirements

### Must-Haves
1. Type resolution errors are collected instead of ignored
2. Errors include detailed context (location, expected type, found type)
3. Errors are returned from the `analyze()` method
4. Error messages are clear and actionable

### Should-Haves
1. Error collection is thread-safe (if needed)
2. Errors can be filtered by severity
3. Error locations point to exact source positions

### Nice-to-Haves
1. Error suggestions for fixes
2. Error deduplication
3. Error categorization (type error, missing type, etc.)

## Tasks

### Task 1: Add error collection infrastructure
Implement error storage in TypeResolver:
- Add error vector to TypeResolver struct
- Create error types for different resolution failures
- Implement error collection methods

**Implementation details:**
- Add `errors: Vec<ResolutionError>` field to `TypeResolver`
- Update `ResolutionError` enum with more detailed variants:
  - `TypeNotFound(String, Span)` - type reference couldn't be resolved
  - `InvalidTypeAnnotation(String, Span)` - malformed type annotation
  - `CircularTypeReference(String, Span)` - circular type dependency
  - `GenericArityMismatch { expected: usize, actual: usize, span: Span }`
- Add `add_error(&mut self, error: ResolutionError)` helper method

### Task 2: Replace silent failures with error collection
Update all type resolution sites to collect errors:
- Find all places returning `Err(...)` without handling
- Replace with `self.add_error(error)` and continue
- Ensure type resolution continues despite errors

**Implementation details:**
- In `visit_variable_statement`:
  - Replace `Err(_) => None` with `self.add_error(error)` followed by `None`
  - Ensure error context includes the variable name and location
- In `resolve_type`:
  - Collect errors instead of just returning Err
  - Store error information for later reporting
- In `type_annotation_to_type`:
  - Add error collection for invalid type annotations
  - Include span information for error reporting

### Task 3: Implement error context tracking
Add detailed context to errors:
- Track the scope where error occurred
- Track the AST node causing the error
- Track expected vs actual types

**Implementation details:**
- Enhance `ResolutionError` with context fields:
  - `scope_id: ScopeId`
  - `expected_type: Option<TypeId>`
  - `found_type: Option<TypeId>`
  - `suggestion: Option<String>`
- Update error creation to include relevant context
- Store context during resolution for error reporting

### Task 4: Return errors from analyze method
Implement error return from TypeResolver:
- Change `analyze()` signature to return errors
- Collect all errors during resolution
- Return vector of errors for reporting

**Implementation details:**
- Update `TypeResolver::analyze()` to return `Result<(), Vec<ResolutionError>>`
- Or add `fn take_errors(&mut self) -> Vec<ResolutionError>` method
- Ensure all errors are collected before method returns
- Update `SemanticAnalyzer` to handle resolution errors

### Task 5: Update SemanticAnalyzer error handling
Propagate resolution errors to main analyzer:
- Handle resolution errors from TypeResolver
- Include resolution errors in SemanticError
- Provide user-facing error messages

**Implementation details:**
- Update `SemanticError` enum to include `ResolutionErrors(Vec<ResolutionError>)`
- In `SemanticAnalyzer::analyze()`:
  - After type resolution pass, collect errors
  - Return error if any resolution errors occurred
  - Format errors into user-friendly messages
- Ensure errors include source locations for IDE integration

### Task 6: Add error formatting
Implement user-friendly error formatting:
- Format errors with source context
- Include line/column information
- Provide error code for categorization

**Implementation details:**
- Implement `Display` trait for `ResolutionError`
- Format includes: error code, message, location, suggestion
- Example format:
  ```
  Error TS2304: Cannot find name 'UnknownType'
    at file.ts:10:5
  ```
- Consider using `codespan-reporting` crate for consistent formatting

## Verification Criteria

1. All existing tests pass
2. New tests verify:
   - Type resolution errors are collected
   - Errors include detailed context
   - Errors are returned from analyze method
   - Error messages are clear and actionable
3. TODO comment at line 403 is removed
4. No type resolution errors are silently ignored

## Success Metrics

- All type resolution errors are collected and reported
- Verification gap "Type resolution errors are properly collected and reported" is closed
- Test coverage > 85% for error handling
- Error messages are clear and point to exact source locations

## Notes

- Error collection should not stop analysis (continue reporting multiple errors)
- Error messages should be consistent with TypeScript's error format
- Consider using error codes for IDE integration
- Performance impact should be minimal (only collect when errors occur)

## Dependencies

None - standalone improvement to error handling.

## Estimated Time

2-3 hours for implementation and testing
