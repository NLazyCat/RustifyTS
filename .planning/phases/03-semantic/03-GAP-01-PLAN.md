---
wave: 1
depends_on: []
files_modified:
  - src/semantic/types/unify.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-01: Type Unification Implementation

## Overview
Implement the `unify()` function in `src/semantic/types/unify.rs` to compute the most general common type of two types.

## Context
The `unify()` function is currently a `todo!()` stub, preventing full type checking capabilities. This function is needed for computing the least upper bound (LUB) of types, which is essential for union type formation and type inference scenarios.

## Goal
Implement a complete type unification algorithm that follows TypeScript semantics for computing the most general common type.

## Requirements

### Must-Haves
1. `unify()` function returns a valid `Result<Type, UnificationError>` for all type combinations
2. Unification handles all type variants: Primitive, Array, Tuple, Object, Function, Union, Intersection, Generic
3. Unification respects TypeScript's subtyping relationships
4. Error messages are clear and descriptive

### Should-Haves
1. Recursive unification for complex types
2. Efficient performance through memoization (optional)

### Nice-to-Haves
1. Detailed error context (line/column information)

## Tasks

### Task 1: Implement primitive type unification
Implement unification for primitive types:
- Identical primitives unify to themselves
- Different primitives return `UnificationError::IncompatibleTypes`
- Special cases for `Any`, `Never`, `Unknown` types

**Implementation details:**
- Match on `(Type::Primitive, Type::Primitive)` pattern
- Return the type directly for identical primitives
- Return error for incompatible primitives
- Handle top/bottom types (`Any`, `Never`, `Unknown`) correctly

### Task 2: Implement array and tuple unification
Implement unification for array and tuple types:
- Arrays unify if elements unify
- Tuples unify if length matches and all elements unify
- Return appropriate error for mismatched structures

**Implementation details:**
- For arrays: recursively unify element types
- For tuples: check length equality first, then unify each element positionally
- Return `UnificationError::IncompatibleTypes` for structure mismatches

### Task 3: Implement object type unification
Implement unification for object types using structural typing:
- Unify property types by name (intersection of properties)
- Handle index signatures correctly
- Return unified object type with merged properties

**Implementation details:**
- Create unified properties map containing properties from both objects
- For overlapping properties, unify their types
- Merge index signatures (must be compatible)
- Result contains all properties from both objects with unified types

### Task 4: Implement function type unification
Implement unification for function types following function subtyping rules:
- Parameter types unify using contravariance (supertype)
- Return type unifies using covariance (subtype)
- Type parameters must be compatible

**Implementation details:**
- Check parameter count equality
- Unify parameters in reverse order (contravariant)
- Unify return types normally (covariant)
- Handle type parameter constraints

### Task 5: Implement union and intersection unification
Implement unification for union and intersection types:
- Unions: compute LUB of all type pairs
- Intersections: compute GLB of all type pairs
- Flatten nested unions/intersections

**Implementation details:**
- For unions: unify all types pairwise, collect results, remove duplicates
- For intersections: compute greatest lower bound by intersecting all types
- Use `FxHashMap` for deduplication of results
- Handle edge cases (empty unions/intersections)

### Task 6: Implement generic type unification
Implement unification for generic types with variance awareness:
- Covariant type parameters (arrays, tuples)
- Contravariant type parameters (function parameters)
- Invariant type parameters (object properties)

**Implementation details:**
- Recursively unify generic arguments based on variance rules
- Handle base type unification for generic types
- Return `UnificationError::ArityMismatch` for argument count mismatches
- For now, implement basic handling (full variance is a nice-to-have)

## Verification Criteria

1. All existing tests pass
2. New tests added for each unification scenario:
   - Primitive types (including Any, Never, Unknown)
   - Arrays and tuples
   - Object types with properties and index signatures
   - Function types with variance
   - Union and intersection types
   - Generic types (basic)
3. No `todo!()` macros remain in `unify()` function
4. Unification errors have clear, descriptive messages

## Success Metrics

- Test coverage > 90% for unify module
- All verification gaps related to type unification are closed
- Function completes without panics for all type combinations

## Notes

- Follow TypeScript's type unification semantics as closely as possible
- Use existing `is_subtype()` function to guide unification logic
- Consider performance implications of recursive unification
- Error messages should reference both input types for clarity

## Dependencies

None - this is a standalone implementation within the existing type system module.

## Estimated Time

2-3 hours for implementation and testing
