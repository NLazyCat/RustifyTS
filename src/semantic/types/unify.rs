//! Type unification and compatibility checking.
//!
//! This module implements the type unification algorithm that determines if two
//! types are compatible and computes the most general type that both can be
//! unified to.

use super::*;
use fxhash::FxHashMap;

/// Result of a type unification attempt.
pub type UnificationResult<T = ()> = Result<T, UnificationError>;

/// Error that occurs when type unification fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UnificationError {
    /// Types are incompatible and cannot be unified
    #[error("types `{0}` and `{1}` are incompatible")]
    IncompatibleTypes(String, String),

    /// Occurs check failure (recursive type without proper indirection)
    #[error("recursive type constraint: `{0}` appears within `{1}`")]
    RecursiveType(String, String),

    /// Type parameter constraint violation
    #[error("type `{0}` does not satisfy constraint `{1}`")]
    ConstraintViolation(String, String),

    /// Generic arity mismatch
    #[error("generic type expects {expected} arguments, got {actual}")]
    ArityMismatch { expected: usize, actual: usize },
}

/// Check if type `a` is a subtype of type `b` according to TypeScript rules.
///
/// Returns true if a value of type `a` can be assigned to a variable of type `b`.
pub fn is_subtype(a: TypeId, b: TypeId, interner: &TypeInterner) -> bool {
    // Fast path: identical types are always subtypes
    if a == b {
        return true;
    }

    let a_ty = match interner.get(a) {
        Some(ty) => ty,
        None => return false,
    };

    let b_ty = match interner.get(b) {
        Some(ty) => ty,
        None => return false,
    };

    is_subtype_internal(a_ty, b_ty, interner)
}

/// Internal subtype check implementation that works with Type references.
fn is_subtype_internal(a: &Type, b: &Type, interner: &TypeInterner) -> bool {
    use PrimitiveType::*;

    match (a, b) {
        // Any type is compatible with anything
        (_, Type::Primitive(Any)) => true,
        (Type::Primitive(Any), _) => true,

        // Never is a subtype of every type
        (Type::Primitive(Never), _) => true,

        // Every type is a subtype of unknown
        (_, Type::Primitive(Unknown)) => true,

        // Primitive type equality
        (Type::Primitive(a_prim), Type::Primitive(b_prim)) => a_prim == b_prim,

        // Union vs Union: A1 | A2 <: B1 | B2 if each Ai is subtype of some Bj
        (Type::Union(a_types), Type::Union(b_types)) => {
            a_types.iter().all(|a_ty| {
                b_types.iter().any(|b_ty| is_subtype_internal(a_ty, b_ty, interner))
            })
        }

        // Union types: A <: B | C if A <: B or A <: C
        (_, Type::Union(b_types)) => {
            b_types.iter().any(|b_ty| is_subtype_internal(a, b_ty, interner))
        }

        // A | B <: C if A <: C and B <: C
        (Type::Union(a_types), _) => {
            a_types.iter().all(|a_ty| is_subtype_internal(a_ty, b, interner))
        }

        // Intersection types: A & B <: A and A & B <: B
        (Type::Intersection(a_types), _) => {
            a_types.iter().any(|a_ty| is_subtype_internal(a_ty, b, interner))
        }

        // A <: B & C if A <: B and A <: C
        (_, Type::Intersection(b_types)) => {
            b_types.iter().all(|b_ty| is_subtype_internal(a, b_ty, interner))
        }

        // Array types: covariant element types (TypeScript behavior)
        (Type::Array(a_elem), Type::Array(b_elem)) => {
            is_subtype_internal(a_elem, b_elem, interner)
        }

        // Tuple types: length must match, elements must be subtypes in order
        (Type::Tuple(a_elems), Type::Tuple(b_elems)) => {
            if a_elems.len() != b_elems.len() {
                return false;
            }
            a_elems.iter().zip(b_elems.iter()).all(|(a_e, b_e)| {
                is_subtype_internal(a_e, b_e, interner)
            })
        }

        // Object types: structural typing
        (Type::Object(a_obj), Type::Object(b_obj)) => {
            // All properties in b must exist in a with compatible types
            for (prop_name, b_prop_ty) in &b_obj.properties {
                match a_obj.properties.get(prop_name) {
                    Some(a_prop_ty) => {
                        if !is_subtype_internal(a_prop_ty, b_prop_ty, interner) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }

            // If b has an index signature, a must have compatible index signature
            // or all properties must match the index signature type
            if let Some(b_index_ty) = &b_obj.index_signature {
                match &a_obj.index_signature {
                    Some(a_index_ty) => {
                        if !is_subtype_internal(a_index_ty, b_index_ty, interner) {
                            return false;
                        }
                    }
                    None => {
                        // Check all properties match the index signature
                        for a_prop_ty in a_obj.properties.values() {
                            if !is_subtype_internal(a_prop_ty, b_index_ty, interner) {
                                return false;
                            }
                        }
                    }
                }
            }

            // Extra properties in a are allowed (structural typing)
            true
        }

        // Function types: parameter contravariance, return type covariance
        (
            Type::Function { params: a_params, return_type: a_return, .. },
            Type::Function { params: b_params, return_type: b_return, .. }
        ) => {
            // Parameters: b must accept all parameters that a accepts
            // (contravariance: b's parameters must be supertypes of a's)
            if a_params.len() != b_params.len() {
                return false;
            }

            for (a_param, b_param) in a_params.iter().zip(b_params.iter()) {
                if !is_subtype_internal(b_param, a_param, interner) {
                    return false;
                }
            }

            // Return type: a's return type must be a subtype of b's
            is_subtype_internal(a_return, b_return, interner)
        }

        // TODO: Generic type handling
        (Type::Generic { .. }, Type::Generic { .. }) => {
            // For now, only identical generics are compatible
            // Full variance handling will be implemented later
            a == b
        }

        // Default: types are not compatible
        _ => false,
    }
}

/// Substitute type parameters in a type with their concrete types.
///
/// Takes a type and a substitution map from type parameter IDs to their replacement types,
/// and returns a new type with all type parameters replaced according to the map.
pub fn substitute_type_params(
    ty: TypeId,
    substitutions: &FxHashMap<TypeId, TypeId>,
    interner: &mut TypeInterner,
) -> TypeId {
    // Check if this type is directly in the substitution map
    if let Some(&replacement) = substitutions.get(&ty) {
        return replacement;
    }

    let type_val = match interner.get(ty) {
        Some(t) => t.clone(),
        None => return ty,
    };

    let substituted = match type_val {
        // Simple types that don't contain other types
        Type::Primitive(_) => type_val,

        // Array: substitute element type
        Type::Array(elem) => {
            let substituted_elem = substitute_type_params(
                interner.intern(*elem),
                substitutions,
                interner,
            );
            Type::Array(Box::new(interner.get(substituted_elem).unwrap().clone()))
        }

        // Tuple: substitute each element type
        Type::Tuple(elems) => {
            let substituted_elems: Vec<_> = elems
                .into_iter()
                .map(|elem| {
                    let elem_id = interner.intern(elem);
                    let substituted_id = substitute_type_params(elem_id, substitutions, interner);
                    interner.get(substituted_id).unwrap().clone()
                })
                .collect();
            Type::Tuple(substituted_elems)
        }

        // Object: substitute property types and index signature
        Type::Object(obj) => {
            let mut new_properties = FxHashMap::default();
            for (name, prop_ty) in obj.properties {
                let prop_id = interner.intern(prop_ty);
                let substituted_id = substitute_type_params(prop_id, substitutions, interner);
                new_properties.insert(name, interner.get(substituted_id).unwrap().clone());
            }

            let new_index_signature = obj.index_signature.map(|sig| {
                let sig_id = interner.intern(*sig);
                let substituted_id = substitute_type_params(sig_id, substitutions, interner);
                Box::new(interner.get(substituted_id).unwrap().clone())
            });

            Type::Object(ObjectType {
                properties: new_properties,
                index_signature: new_index_signature,
            })
        }

        // Function: substitute parameter types, return type, and type parameter constraints
        Type::Function { params, return_type, type_params } => {
            let substituted_params: Vec<_> = params
                .into_iter()
                .map(|param| {
                    let param_id = interner.intern(param);
                    let substituted_id = substitute_type_params(param_id, substitutions, interner);
                    interner.get(substituted_id).unwrap().clone()
                })
                .collect();

            let return_id = interner.intern(*return_type);
            let substituted_return_id = substitute_type_params(return_id, substitutions, interner);
            let substituted_return = Box::new(interner.get(substituted_return_id).unwrap().clone());

            let substituted_type_params: Vec<_> = type_params
                .into_iter()
                .map(|tp| {
                    let substituted_constraint = tp.constraint.map(|c| {
                        let c_id = interner.intern(*c);
                        let substituted_c_id = substitute_type_params(c_id, substitutions, interner);
                        Box::new(interner.get(substituted_c_id).unwrap().clone())
                    });

                    let substituted_default = tp.default.map(|d| {
                        let d_id = interner.intern(*d);
                        let substituted_d_id = substitute_type_params(d_id, substitutions, interner);
                        Box::new(interner.get(substituted_d_id).unwrap().clone())
                    });

                    TypeParameter {
                        name: tp.name,
                        constraint: substituted_constraint,
                        default: substituted_default,
                    }
                })
                .collect();

            Type::Function {
                params: substituted_params,
                return_type: substituted_return,
                type_params: substituted_type_params,
            }
        }

        // Union: substitute each type in the union
        Type::Union(types) => {
            let substituted_types: Vec<_> = types
                .into_iter()
                .map(|ty| {
                    let ty_id = interner.intern(ty);
                    let substituted_id = substitute_type_params(ty_id, substitutions, interner);
                    interner.get(substituted_id).unwrap().clone()
                })
                .collect();
            Type::Union(substituted_types)
        }

        // Intersection: substitute each type in the intersection
        Type::Intersection(types) => {
            let substituted_types: Vec<_> = types
                .into_iter()
                .map(|ty| {
                    let ty_id = interner.intern(ty);
                    let substituted_id = substitute_type_params(ty_id, substitutions, interner);
                    interner.get(substituted_id).unwrap().clone()
                })
                .collect();
            Type::Intersection(substituted_types)
        }

        // Type parameter: if it's in substitutions, use that, otherwise keep as is
        Type::TypeParameter(_) => type_val,

        // Generic: substitute base and arguments
        Type::Generic { base, args } => {
            let substituted_base = substitute_type_params(base, substitutions, interner);
            let substituted_args: Vec<_> = args
                .into_iter()
                .map(|arg| {
                    let arg_id = interner.intern(arg);
                    let substituted_id = substitute_type_params(arg_id, substitutions, interner);
                    interner.get(substituted_id).unwrap().clone()
                })
                .collect();

            Type::Generic {
                base: substituted_base,
                args: substituted_args,
            }
        }

        // Reference: substitute type arguments
        Type::Reference { name, type_args } => {
            let substituted_args: Vec<_> = type_args
                .into_iter()
                .map(|arg| {
                    let arg_id = interner.intern(arg);
                    let substituted_id = substitute_type_params(arg_id, substitutions, interner);
                    interner.get(substituted_id).unwrap().clone()
                })
                .collect();

            Type::Reference {
                name,
                type_args: substituted_args,
            }
        }
    };

    interner.intern(substituted)
}

/// Unify two types, returning their most general common type.
///
/// # Errors
///
/// Returns an error if the types cannot be unified.
pub fn unify(ty1: &Type, ty2: &Type, interner: &TypeInterner) -> UnificationResult<Type> {
    todo!("Implement type unification");
}

/// Check if a type is assignable to another type.
///
/// Returns true if a value of type `from` can be assigned to a variable of type `to`.
pub fn is_assignable(from: &Type, to: &Type, interner: &TypeInterner) -> bool {
    todo!("Implement assignability check");
}
