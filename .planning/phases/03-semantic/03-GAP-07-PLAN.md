---
wave: 3
depends_on: ["03-GAP-01", "03-GAP-02"]
files_modified:
  - src/semantic/types/unify.rs
  - src/semantic/types/resolver.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-07: Generic Type Variance Support

## Overview
Implement full variance support for generic type parameters in subtyping and unification.

## Context
The `is_subtype_internal()` function in `unify.rs` has a TODO (line 174) indicating minimal generic handling. Generic type variance is essential for proper TypeScript type checking of generic containers and functions.

## Goal
Implement complete variance rules for generic types, handling covariant, contravariant, and invariant type parameters correctly.

## Requirements

### Must-Haves
1. Generic types with covariant type parameters handled correctly
2. Generic types with contravariant type parameters handled correctly
3. Generic types with invariant type parameters handled correctly
4. Built-in generic types have correct variance annotations

### Should-Haves
1. Support for custom generic types with variance annotations
2. Handle generic type parameter constraints
3. Handle generic type parameter defaults

### Nice-to-Haves
1. Bivariant type parameters (for certain TypeScript edge cases)
2. Optimized variance checking through caching

## Tasks

### Task 1: Define variance tracking for types
Implement variance annotation system:
- Add variance information to type system
- Track variance for each type parameter
- Define variance enum: Covariant, Contravariant, Invariant, Bivariant

**Implementation details:**
- Create `Variance` enum with variants: Covariant, Contravariant, Invariant, Bivariant
- Add variance tracking to `Generic` type variant
- Add variance tracking to `TypeParameter` struct
- Create a mapping of built-in generic types to their variance:
  - Array: covariant in element type
  - Promise: covariant in value type
  - Function: contravariant in parameters, covariant in return
  - Readonly: covariant
  - Map, Set: covariant in value type

### Task 2: Implement covariant generic subtyping
Handle covariant type parameters:
- For generic types with covariant parameters, check subtyping recursively
- Example: `Array<string>` is subtype of `Array<any>`

**Implementation details:**
- In `is_subtype_internal`, for generic type comparison:
  - Check if base type names match
  - For each type parameter:
    - If variance is covariant, check `is_subtype(from_arg, to_arg)`
    - Apply recursively for nested generics
- Update the TODO at line 174 to implement covariant case

### Task 3: Implement contravariant generic subtyping
Handle contravariant type parameters:
- For generic types with contravariant parameters, reverse subtyping
- Example: `Function<any>` is subtype of `Function<string>`

**Implementation details:**
- In `is_subtype_internal`, handle contravariant variance:
  - For contravariant type parameters, check `is_subtype(to_arg, from_arg)`
  - This is the opposite of covariant checking
- Apply to function generic parameters and other contravariant cases

### Task 4: Implement invariant generic subtyping
Handle invariant type parameters:
- For invariant type parameters, require exact equality
- Example: `MutableBox<string>` is not subtype of `MutableBox<any>`

**Implementation details:**
- In `is_subtype_internal`, handle invariant variance:
  - For invariant type parameters, check `from_arg == to_arg`
  - No subtyping allowed, must be identical
- Apply to mutable containers and generic types with both read/write access

### Task 5: Implement bivariant generic subtyping
Handle bivariant type parameters (TypeScript edge cases):
- Bivariant types allow subtyping in both directions
- Rare but needed for full TypeScript compatibility

**Implementation details:**
- In `is_subtype_internal`, handle bivariant variance:
  - For bivariant parameters, always return true
  - Or check `is_subtype(from_arg, to_arg) || is_subtype(to_arg, from_arg)`
- Apply to specific TypeScript cases that require bivariance

### Task 6: Create variance registry for built-in types
Implement built-in type variance mappings:
- Create registry mapping generic type names to variance rules
- Populate with TypeScript standard library types
- Support user-defined variance annotations

**Implementation details:**
- Create `VarianceRegistry` struct
- Map type names to their variance profiles
- Populate with standard types:
  - `Array<T>`: covariant T
  - `Promise<T>`: covariant T
  - `ReadonlyArray<T>`: covariant T
  - `Map<K, V>`: covariant K, covariant V
  - `Set<T>`: covariant T
  - `Function<T>`: contravariant T
- For now, hardcode variance for built-in types

### Task 7: Update generic type resolution
Ensure generic type resolution respects variance:
- When resolving generic type references, apply variance rules
- Handle generic type parameter constraints correctly
- Support generic type parameter defaults

**Implementation details:**
- In `TypeResolver`, when resolving generic types:
  - Look up variance for the generic type
  - Apply variance rules when checking compatibility
  - Ensure constraints are satisfied for type arguments
  - Apply defaults for missing type arguments

## Verification Criteria

1. All existing tests pass
2. New tests verify:
   - Covariant generic subtyping works correctly
   - Contravariant generic subtyping works correctly
   - Invariant generic subtyping requires exact equality
   - Built-in generic types have correct variance
   - Generic type constraints are respected
3. TODO comment at line 174 is removed or fully implemented
4. Generic type checking matches TypeScript behavior

## Success Metrics

- All generic type subtyping scenarios work correctly
- Verification gap "Generic type subtyping has full variance support" is closed
- Test coverage > 80% for generic variance handling

## Notes

- Variance is complex and TypeScript has some special cases
- Some TypeScript types (like Function) are bivariant for legacy reasons
- Full variance support may require multiple iterations
- Consider creating variance annotations for user-defined types

## Dependencies

- 03-GAP-01: Type unification (for unifying generic types)
- 03-GAP-02: Type assignability (for checking generic assignability)

## Estimated Time

4-5 hours for implementation and comprehensive testing

## Risks

- Variance rules are subtle and easy to get wrong
- TypeScript has special cases that may be surprising
- Complex generic hierarchies may cause performance issues
- Interaction with generic type resolution may be complex
