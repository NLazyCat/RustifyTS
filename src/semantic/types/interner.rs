//! Type interner for deduplicating TypeScript types.
//!
//! This module provides a TypeInterner that stores unique Type instances and
//! assigns them TypeIds for fast equality checks and lookups. Identical types
//! are automatically deduplicated to save memory and enable O(1) type comparison.

use super::*;
use fxhash::FxHashMap;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU32, Ordering};

/// A type interner that deduplicates Type instances and provides fast lookups.
///
/// The TypeInterner uses a hash map to store unique Type instances and assign
/// them unique TypeIds. This allows for O(1) type equality checks (just compare
/// TypeIds) and significant memory savings when dealing with many identical types.
#[derive(Debug)]
pub struct TypeInterner {
    /// Map from Type to its TypeId for deduplication
    type_to_id: FxHashMap<Type, TypeId>,
    /// Map from TypeId to Type for lookups
    id_to_type: Vec<Type>,
    /// Counter for generating new TypeIds
    next_id: AtomicU32,
}

impl Default for TypeInterner {
    fn default() -> Self {
        Self {
            type_to_id: FxHashMap::default(),
            id_to_type: Vec::new(),
            next_id: AtomicU32::new(0),
        }
    }
}

impl TypeInterner {
    /// Create a new empty TypeInterner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a type, returning its unique TypeId.
    ///
    /// If the type already exists in the interner, returns the existing TypeId.
    /// Otherwise, stores the type and returns a new TypeId.
    pub fn intern(&mut self, ty: Type) -> TypeId {
        if let Some(&id) = self.type_to_id.get(&ty) {
            return id;
        }

        let id = TypeId::new(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.type_to_id.insert(ty.clone(), id);
        self.id_to_type.push(ty);
        id
    }

    /// Get a reference to a type by its TypeId.
    ///
    /// Returns None if the TypeId is not present in the interner.
    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.id_to_type.get(id.into_u32() as usize)
    }

    /// Intern a primitive type, returning its TypeId.
    ///
    /// Convenience method for interning primitive types without having to
    /// construct the Type::Primitive variant manually.
    pub fn get_or_intern_primitive(&mut self, prim: PrimitiveType) -> TypeId {
        self.intern(Type::Primitive(prim))
    }

    /// Intern an array type with the given element type.
    ///
    /// Convenience method for interning array types. Takes the TypeId of the
    /// element type, constructs the Array variant, and interns it.
    pub fn get_or_intern_array(&mut self, element: TypeId) -> TypeId {
        let element_ty = self.get(element).expect("Invalid TypeId for array element").clone();
        self.intern(Type::Array(Box::new(element_ty)))
    }

    /// Intern a union type from a list of TypeIds.
    ///
    /// Automatically sorts and deduplicates the types to ensure consistent
    /// representation of union types. Unions with zero types are converted to
    /// Never, unions with one type return that type directly.
    pub fn get_or_intern_union(&mut self, types: Vec<TypeId>) -> TypeId {
        // Deduplicate and sort types for consistent representation
        let unique_types: BTreeSet<_> = types.into_iter().collect();

        match unique_types.len() {
            0 => self.get_or_intern_primitive(PrimitiveType::Never),
            1 => unique_types.into_iter().next().unwrap(),
            _ => {
                let types: Vec<_> = unique_types
                    .into_iter()
                    .map(|id| self.get(id).expect("Invalid TypeId in union").clone())
                    .collect();
                self.intern(Type::Union(types))
            }
        }
    }
}
