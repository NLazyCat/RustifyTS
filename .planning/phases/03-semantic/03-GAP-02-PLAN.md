---
wave: 1
depends_on: []
files_modified:
  - src/semantic/types/unify.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-02: Type Assignability Check Implementation

## Overview
Implement the `is_assignable()` function in `src/semantic/types/unify.rs` to check if a value of one type can be assigned to a variable of another type.

## Context
The `is_assignable()` function is currently a `todo!()` stub. This function is essential for type checking assignments, function returns, and parameter passing. It determines whether a type `from` can be assigned to type `to`.

## Goal
Implement a complete type assignability check that follows TypeScript's assignment rules, including bidirectional type checking and type inference hints.

## Requirements

### Must-Haves
1. `is_assignable()` returns correct boolean result for all type combinations
2. Assignability respects TypeScript's subtyping and structural typing
3. Handles all type variants: Primitive, Array, Tuple, Object, Function, Union, Intersection, Generic
4. Considers both `is_subtype(from, to)` and `is_subtype(to, from)` for inference

### Should-Haves
1. Bidirectional checking for literal types and inference scenarios
2. Proper handling of type widening and narrowing

### Nice-to-Haves
1. Optimization through caching (optional)
2. Detailed assignability reasons for error messages

## Tasks

### Task 1: Implement base assignability using subtyping
Create the core assignability logic based on subtype relationships:
- A value of type `from` is assignable to type `to` if `is_subtype(from, to)`
- Special handling for bidirectional checking when `from` is more specific

**Implementation details:**
- Use existing `is_subtype()` function as the base
- Check if `from` is subtype of `to` - if true, return true
- For type inference scenarios, check if `to` is subtype of `from` as well
- Handle literal type widening (e.g., `1` to `number`)

### Task 2: Implement assignability for primitive types
Handle primitive type assignability including special types:
- Identical primitives are assignable
- `Any` accepts any type
- `Never` can be assigned to any type
- `Unknown` accepts any type (assignable from anything)
- Other primitives require exact match

**Implementation details:**
- Match on primitive type pairs
- Return true for identical types
- Handle `Any`, `Never`, `Unknown` special cases correctly
- `Any` type is assignable from anything and to anything
- `Never` type can be assigned to anything (bottom type)

### Task 3: Implement assignability for composite types
Handle assignability for arrays, tuples, and objects:
- Arrays: covariant element type checking
- Tuples: length and positional element assignability
- Objects: structural typing with property existence and type compatibility
- Index signatures checked for compatibility

**Implementation details:**
- For arrays: check `is_assignable(from_elem, to_elem)`
- For tuples: verify length equality and assignability of each element
- For objects: verify all properties in `to` exist in `from` with assignable types
- Handle index signatures: check if index types are assignable
- Allow extra properties in `from` (structural typing)

### Task 4: Implement function type assignability
Handle function type assignability with variance rules:
- Parameter contravariance: `to` parameters must accept `from` parameters
- Return type covariance: `from` return type must be assignable to `to` return type
- Type parameter constraints must be satisfied

**Implementation details:**
- Check parameter count equality
- For each parameter: `is_assignable(to_param, from_param)` (contravariant)
- Check return type: `is_assignable(from_return, to_return)` (covariant)
- Handle optional and rest parameters correctly
- Verify type parameter constraints are compatible

### Task 5: Implement union and intersection assignability
Handle assignability for union and intersection types:
- Union `A | B` assignable to `T` if `A` and `B` are assignable to `T`
- `T` assignable to union `A | B` if `T` is assignable to either `A` or `B`
- Intersection `A & B` assignable to `T` if `A` or `B` is assignable to `T`
- `T` assignable to intersection `A & B` if `T` is assignable to both `A` and `B`

**Implementation details:**
- For union to type: all union members must be assignable to target type
- For type to union: source type must be assignable to at least one union member
- For intersection to type: at least one intersection member must be assignable to target
- For type to intersection: source type must be assignable to all intersection members

### Task 6: Implement generic type assignability
Handle assignability for generic types:
- Simple case: identical generic types are assignable
- Advanced: consider type parameter variance (nice-to-have for this plan)

**Implementation details:**
- Check if generic names match
- Verify type argument counts are equal
- Recursively check assignability of type arguments
- For basic implementation, require exact match of generic base type
- Full variance handling deferred to generic variance improvement plan

## Verification Criteria

1. All existing tests pass
2. New tests added for assignability scenarios:
   - Primitive types including Any, Never, Unknown
   - Arrays and tuples with covariant checking
   - Objects with structural typing
   - Functions with contravariant parameters and covariant returns
   - Union and intersection types
   - Generic types (basic matching)
3. No `todo!()` macros remain in `is_assignable()` function
4. Assignability checks correctly identify both assignable and non-assignable cases

## Success Metrics

- Test coverage > 85% for assignability checking
- All verification gaps related to type assignability are closed
- Function returns correct results for TypeScript's standard type checking scenarios

## Notes

- Assignability is more permissive than subtyping in some cases (type inference)
- Use `is_subtype()` as the foundation but add additional checks for inference
- Consider TypeScript's bidirectional type checking for function parameters
- Error messages (when extended) should indicate why types are not assignable

## Dependencies

None - depends only on existing `is_subtype()` function and type system infrastructure.

## Estimated Time

1.5-2.5 hours for implementation and testing
