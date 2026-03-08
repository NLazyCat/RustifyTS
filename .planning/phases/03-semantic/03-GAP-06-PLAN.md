---
wave: 3
depends_on: []
files_modified:
  - src/semantic/scope/analyzer.rs
autonomous: true
gap_closure: true
---

# Plan 03-GAP-06: Class Type Information Extraction

## Overview
Extract type information from class declarations and associate it with class symbols.

## Context
The `visit_class_declaration` method in `ScopeAnalyzer` has a TODO (line 415) indicating that class type information is not being extracted. This prevents proper type checking for class instances and methods.

## Goal
Extract type information from class declarations, including properties, methods, and inheritance, and create appropriate Type representations.

## Requirements

### Must-Haves
1. Extract class properties and their types
2. Extract class methods and their signatures
3. Create appropriate Type representation for the class
4. Associate the type with the class symbol

### Should-Haves
1. Handle class inheritance (extends clause)
2. Handle class modifiers (public, private, protected)
3. Handle static members

### Nice-to-Haves
1. Support for class generics
2. Support for abstract classes
3. Support for constructor parameter properties

## Tasks

### Task 1: Extract class member information
Implement class member extraction:
- Parse class body to find properties and methods
- Extract member names and type annotations
- Track member modifiers (public, private, protected, static)

**Implementation details:**
- In `visit_class_declaration`, examine the `members` field
- Iterate through class members
- For each member, extract:
  - Member kind (property, method, constructor)
  - Member name
  - Type annotation (for properties)
  - Parameter types (for methods)
  - Return type (for methods)
  - Modifiers

### Task 2: Create ObjectType for class
Implement type creation for classes:
- Create an `ObjectType` with class properties
- Handle method signatures as function-typed properties
- Include index signature if present
- Store type information in class symbol

**Implementation details:**
- Create `ObjectType` struct with:
  - Properties map (name -> TypeId)
  - Optional index signature
- For each property member:
  - Convert type annotation to TypeId
  - Add to properties map
- For each method:
  - Create function type with parameters and return type
  - Add function type to properties
- For constructor:
  - Add to special properties if needed

### Task 3: Handle class inheritance
Implement extends clause processing:
- Extract base class name from extends clause
- Resolve base class type from symbol table
- Create type relationship (subtyping)

**Implementation details:**
- If class has `extends` clause:
  - Extract base class name
  - Look up base class in symbol table
  - Get base class type from its symbol
  - Create extended type with base and derived class information
- For now, store base class name in class metadata
- Full inheritance resolution may be deferred

### Task 4: Handle static and instance members
Distinguish between static and instance members:
- Static members belong to the class type itself
- Instance members belong to instances
- Create separate type representations if needed

**Implementation details:**
- Track static modifier on members
- For static members: add to class type (constructor function type)
- For instance members: add to instance type
- Consider creating two types: class type and instance type
- For simplicity, may combine into single type with flags

### Task 5: Integrate class type with symbol
Add class type information to class symbol:
- Set type_id on class symbol
- Store class metadata (members, inheritance)
- Ensure type is accessible from symbol table

**Implementation details:**
- After creating class type:
  - Intern the type using `type_interner`
  - Set the type_id on the class symbol
  - Store additional class metadata (member list, base class)
  - Remove TODO comment at line 415
- Ensure class symbol has complete information for downstream analysis

## Verification Criteria

1. All existing tests pass
2. New tests verify:
   - Class declarations have type information
   - Class properties are in type
   - Class methods are in type
   - Inheritance relationships are tracked
   - Static vs instance members are distinguished
3. TODO comment at line 415 is removed
4. Class types are accessible via symbol table lookup

## Success Metrics

- All class declarations have complete type information
- Verification gap "Class type information is extracted from class declarations" is closed
- Test coverage > 75% for class type extraction

## Notes

- TypeScript classes have both static and instance sides
- May need to create two types per class (class type and instance type)
- Constructor properties are a TypeScript feature that adds complexity
- Full inheritance resolution may require separate phase

## Dependencies

None - standalone improvement to scope and type analysis.

## Estimated Time

3-4 hours for implementation and testing

## Risks

- Complex class features (generics, decorators, mixins) add complexity
- Interaction with existing type system may be challenging
- Class type representation may need iteration
