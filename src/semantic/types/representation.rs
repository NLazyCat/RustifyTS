//! Type representation for TypeScript types.
//!
//! This module defines the core Type enum and supporting types that represent
//! all TypeScript type constructs in a memory-efficient, immutable format.

use fxhash::FxHashMap;
use lasso::Key;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU32;

/// Unique identifier for an interned type.
///
/// TypeId is a newtype around u32 that implements Copy and Eq for fast
/// equality checks. Types are compared by their TypeId for O(1) equality checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(u32);

impl TypeId {
    /// Create a new TypeId from a raw u32 value.
    ///
    /// This is only safe if the value corresponds to a valid, interned type.
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw u32 value of this TypeId.
    pub fn into_u32(self) -> u32 {
        self.0
    }
}

unsafe impl Key for TypeId {
    #[inline]
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    fn try_from_usize(int: usize) -> Option<Self> {
        // Use NonZeroU32 to ensure we don't create zero TypeIds
        // This allows for niche optimization in Option<TypeId>
        NonZeroU32::new(int as u32).map(|nz| Self(nz.get()))
    }
}

/// Primitive TypeScript types.
///
/// These are the built-in primitive types that form the foundation of the
/// TypeScript type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// String type: `string`
    String,
    /// Number type: `number`
    Number,
    /// Boolean type: `boolean`
    Boolean,
    /// Null type: `null`
    Null,
    /// Undefined type: `undefined`
    Undefined,
    /// Void type: `void`
    Void,
    /// Never type: `never`
    Never,
    /// Unknown type: `unknown`
    Unknown,
    /// Any type: `any`
    Any,
}

impl PartialOrd for PrimitiveType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrimitiveType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = *self as i32;
        let b = *other as i32;
        a.cmp(&b)
    }
}

/// Object type: `{ prop: T; [key: K]: V }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    /// Named properties of the object
    pub properties: FxHashMap<String, Type>,
    /// Optional index signature, if present
    pub index_signature: Option<Box<Type>>,
}

impl Hash for ObjectType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash properties in sorted order to ensure consistent hashing
        let mut entries: Vec<_> = self.properties.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        for (key, value) in entries {
            key.hash(state);
            value.hash(state);
        }
        self.index_signature.hash(state);
    }
}

/// A TypeScript type.
///
/// This enum represents all possible TypeScript type constructs. Each variant
/// contains the necessary data to fully describe the type. Types are immutable
/// once created and are interned for fast comparison and deduplication.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Primitive type (string, number, boolean, etc.)
    Primitive(PrimitiveType),

    /// Array type: `T[]` or `Array<T>`
    Array(Box<Type>),

    /// Tuple type: `[T1, T2, T3]`
    Tuple(Vec<Type>),

    /// Object type: `{ prop: T; [key: K]: V }`
    Object(ObjectType),

    /// Function type: `(param: T) => R`
    Function {
        /// Parameter types
        params: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
        /// Generic type parameters
        type_params: Vec<TypeParameter>,
    },

    /// Union type: `T | U | V`
    Union(Vec<Type>),

    /// Intersection type: `T & U & V`
    Intersection(Vec<Type>),

    /// Generic type parameter: `T extends Constraint = Default`
    TypeParameter(TypeParameter),

    /// Instantiated generic type: `Generic<T, U>`
    Generic {
        /// Base generic type identifier
        base: TypeId,
        /// Type arguments applied to the generic
        args: Vec<Type>,
    },

    /// Named type reference: `TypeReference` or `TypeReference<T>`
    Reference {
        /// Name of the referenced type
        name: String,
        /// Optional type arguments
        type_args: Vec<Type>,
    },
}

/// A generic type parameter definition.
///
/// Represents a type parameter in a generic function, interface, or type alias.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParameter {
    /// Name of the type parameter (e.g., "T")
    pub name: String,
    /// Optional constraint type (e.g., `extends string`)
    pub constraint: Option<Box<Type>>,
    /// Optional default type
    pub default: Option<Box<Type>>,
}
