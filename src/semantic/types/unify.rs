//! Type unification and compatibility checking.
//!
//! This module implements the type unification algorithm that determines if two
//! types are compatible and computes the most general type that both can be
//! unified to.

use super::*;
use super::variance::{Variance, VarianceRegistry};
use fxhash::FxHashMap;

// Global variance registry with built-in types
thread_local! {
    static VARIANCE_REGISTRY: VarianceRegistry = VarianceRegistry::new();
}

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

/// Get variance information for a generic type reference.
///
/// Returns the variance profile for a named generic type, or None if
/// the type is not a known generic or has no variance information.
pub fn get_generic_variance(type_name: &str) -> Option<Vec<Variance>> {
    VARIANCE_REGISTRY.with(|registry| {
        registry.get(type_name).map(|v| v.to_vec())
    })
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

        // Generic type subtyping with variance support
        (Type::Generic { base: base1, args: args1 }, Type::Generic { base: base2, args: args2 }) => {
            // Check if base types are the same
            if base1 != base2 {
                return false;
            }

            // Check if the base type is a reference to determine variance
            let base_ty = match interner.get(*base1) {
                Some(ty) => ty,
                None => return false,
            };

            // Get variance for this generic type
            let variances: Option<Vec<Variance>> = match base_ty {
                Type::Reference { name, .. } => {
                    VARIANCE_REGISTRY.with(|registry| {
                        registry.get(name).map(|v| v.to_vec())
                    })
                }
                _ => None,
            };

            // If we have variance information, use it for checking
            if let Some(variances) = variances {
                let variances_len = variances.len();
                if args1.len() != args2.len() || args1.len() != variances_len {
                    return false;
                }

                // Check each type argument according to its variance
                for ((arg1, arg2), &variance) in args1.iter().zip(args2.iter()).zip(variances.iter()) {
                    let is_subtype = match variance {
                        // Covariant: arg1 must be subtype of arg2
                        Variance::Covariant => is_subtype_internal(arg1, arg2, interner),

                        // Contravariant: arg2 must be subtype of arg1 (reversed)
                        Variance::Contravariant => is_subtype_internal(arg2, arg1, interner),

                        // Invariant: must be equal
                        Variance::Invariant => arg1 == arg2,

                        // Bivariant: always true
                        Variance::Bivariant => true,
                    };

                    if !is_subtype {
                        return false;
                    }
                }

                true
            } else {
                // If no variance information, treat as invariant (require equality)
                // This is conservative but safe
                a == b
            }
        }

        // Type references with variance support
        (Type::Reference { name: name1, type_args: args1 }, Type::Reference { name: name2, type_args: args2 }) => {
            // Check if the type names match
            if name1 != name2 {
                return false;
            }

            // Get variance for this generic type
            let variances: Option<Vec<Variance>> = VARIANCE_REGISTRY.with(|registry| {
                registry.get(name1).map(|v| v.to_vec())
            });

            // If we have variance information, use it for checking
            if let Some(variances) = variances {
                let variances_len = variances.len();
                if args1.len() != args2.len() || args1.len() != variances_len {
                    return false;
                }

                // Check each type argument according to its variance
                for ((arg1, arg2), &variance) in args1.iter().zip(args2.iter()).zip(variances.iter()) {
                    let is_subtype = match variance {
                        // Covariant: arg1 must be subtype of arg2
                        Variance::Covariant => is_subtype_internal(arg1, arg2, interner),

                        // Contravariant: arg2 must be subtype of arg1 (reversed)
                        Variance::Contravariant => is_subtype_internal(arg2, arg1, interner),

                        // Invariant: must be equal
                        Variance::Invariant => arg1 == arg2,

                        // Bivariant: always true
                        Variance::Bivariant => true,
                    };

                    if !is_subtype {
                        return false;
                    }
                }

                true
            } else {
                // If no variance information, treat as invariant (require equality)
                // This is conservative but safe
                a == b
            }
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
/// This function computes the least upper bound (LUB) of two types, which is the
/// most specific type that both input types are subtypes of. This is essential for
/// union type formation and type inference scenarios.
///
/// # Errors
///
/// Returns an error if the types cannot be unified.
pub fn unify(ty1: &Type, ty2: &Type, interner: &mut TypeInterner) -> UnificationResult<Type> {
    use PrimitiveType::*;

    // Fast path: identical types unify to themselves
    if ty1 == ty2 {
        return Ok(ty1.clone());
    }

    match (ty1, ty2) {
        // Task 1: Primitive type unification
        // ===================================

        // Any absorbs everything (LUB with any is any)
        (Type::Primitive(Any), _) | (_, Type::Primitive(Any)) => {
            Ok(Type::Primitive(Any))
        }

        // Unknown is compatible with everything (LUB is unknown)
        (Type::Primitive(Unknown), _) | (_, Type::Primitive(Unknown)) => {
            Ok(Type::Primitive(Unknown))
        }

        // Never is subtype of everything, so LUB is the other type
        (Type::Primitive(Never), other) => Ok(other.clone()),
        (other, Type::Primitive(Never)) => Ok(other.clone()),

        // Identical primitives
        (Type::Primitive(p1), Type::Primitive(p2)) if p1 == p2 => {
            Ok(Type::Primitive(*p1))
        }

        // Different primitives are incompatible
        (Type::Primitive(_), Type::Primitive(_)) => {
            Err(UnificationError::IncompatibleTypes(format!("{:?}", ty1), format!("{:?}", ty2)))
        }

        // Task 2: Array and tuple unification
        // ===================================

        // Arrays: unify element types
        (Type::Array(elem1), Type::Array(elem2)) => {
            let unified_elem = unify(elem1, elem2, interner)?;
            Ok(Type::Array(Box::new(unified_elem)))
        }

        // Tuples: length must match, unify each element
        (Type::Tuple(elems1), Type::Tuple(elems2)) => {
            if elems1.len() != elems2.len() {
                return Err(UnificationError::IncompatibleTypes(
                    format!("tuple of length {}", elems1.len()),
                    format!("tuple of length {}", elems2.len()),
                ));
            }

            let unified_elems: Result<Vec<_>, _> = elems1
                .iter()
                .zip(elems2.iter())
                .map(|(e1, e2)| unify(e1, e2, interner))
                .collect();

            Ok(Type::Tuple(unified_elems?))
        }

        // Task 3: Object type unification
        // =================================

        // Objects: merge properties (intersection of types for overlapping properties)
        (Type::Object(obj1), Type::Object(obj2)) => {
            let mut unified_properties = FxHashMap::default();

            // Add properties from obj1
            for (name, prop_ty) in &obj1.properties {
                unified_properties.insert(name.clone(), prop_ty.clone());
            }

            // Add/merge properties from obj2
            for (name, prop_ty) in &obj2.properties {
                if let Some(existing_ty) = unified_properties.get(name) {
                    // Overlapping property: unify their types
                    let unified_ty = unify(existing_ty, prop_ty, interner)?;
                    unified_properties.insert(name.clone(), unified_ty);
                } else {
                    unified_properties.insert(name.clone(), prop_ty.clone());
                }
            }

            // Handle index signatures
            let unified_index_signature = match (&obj1.index_signature, &obj2.index_signature) {
                (Some(sig1), Some(sig2)) => {
                    // Both have index signatures - must unify
                    Some(Box::new(unify(sig1, sig2, interner)?))
                }
                (Some(sig), None) | (None, Some(sig)) => {
                    // One has index signature - use that
                    Some(sig.clone())
                }
                (None, None) => None,
            };

            Ok(Type::Object(ObjectType {
                properties: unified_properties,
                index_signature: unified_index_signature,
            }))
        }

        // Task 4: Function type unification
        // ==================================

        // Functions: contravariant parameters, covariant return type
        (
            Type::Function { params: params1, return_type: return1, type_params: type_params1 },
            Type::Function { params: params2, return_type: return2, type_params: type_params2 }
        ) => {
            // Parameter count must match
            if params1.len() != params2.len() {
                return Err(UnificationError::IncompatibleTypes(
                    format!("function with {} parameters", params1.len()),
                    format!("function with {} parameters", params2.len()),
                ));
            }

            // Unify parameters using contravariance (use is_subtype to check direction)
            // For LUB, we need to find the common supertype of parameters
            // This is tricky - for now we'll unify naively
            let unified_params: Result<Vec<_>, _> = params1
                .iter()
                .zip(params2.iter())
                .map(|(p1, p2)| unify(p1, p2, interner))
                .collect();

            // Unify return types using covariance
            let unified_return = unify(return1, return2, interner)?;

            // For type parameters, we use the first set (simplification)
            // Full generic unification would require proper variance analysis
            let unified_type_params = if !type_params1.is_empty() {
                type_params1.clone()
            } else if !type_params2.is_empty() {
                type_params2.clone()
            } else {
                vec![]
            };

            Ok(Type::Function {
                params: unified_params?,
                return_type: Box::new(unified_return),
                type_params: unified_type_params,
            })
        }

        // Task 5: Union and intersection unification
        // ==========================================

        // Union vs Union: flatten and deduplicate
        (Type::Union(types1), Type::Union(types2)) => {
            let mut all_types: Vec<_> = types1.iter().chain(types2.iter()).cloned().collect();

            // Try to unify all pairs to find the minimal set
            let mut unified_types: Vec<Type> = Vec::new();

            for ty in all_types {
                let mut merged = false;
                for existing in &mut unified_types {
                    // Try to merge ty into existing
                    if let Ok(merged_ty) = unify(existing, &ty, interner) {
                        *existing = merged_ty;
                        merged = true;
                        break;
                    }
                }

                if !merged {
                    unified_types.push(ty);
                }
            }

            // Sort for canonical representation
            unified_types.sort_by(|a, b| {
                let a_id = interner.intern(a.clone());
                let b_id = interner.intern(b.clone());
                a_id.cmp(&b_id)
            });

            // Remove duplicates
            unified_types.dedup();

            // Handle empty union (should be never)
            if unified_types.is_empty() {
                return Ok(Type::Primitive(Never));
            }

            // Handle single type (unwrap union)
            if unified_types.len() == 1 {
                return Ok(unified_types.into_iter().next().unwrap());
            }

            Ok(Type::Union(unified_types))
        }

        // Union vs non-union: add non-union to union
        (Type::Union(types), other) | (other, Type::Union(types)) => {
            let mut new_types = types.clone();
            new_types.push(other.clone());

            // Try to merge
            let mut unified_types: Vec<Type> = Vec::new();

            for ty in new_types {
                let mut merged = false;
                for existing in &mut unified_types {
                    if let Ok(merged_ty) = unify(existing, &ty, interner) {
                        *existing = merged_ty;
                        merged = true;
                        break;
                    }
                }

                if !merged {
                    unified_types.push(ty);
                }
            }

            // Sort and dedup
            unified_types.sort_by(|a, b| {
                let a_id = interner.intern(a.clone());
                let b_id = interner.intern(b.clone());
                a_id.cmp(&b_id)
            });
            unified_types.dedup();

            if unified_types.is_empty() {
                return Ok(Type::Primitive(Never));
            }

            if unified_types.len() == 1 {
                return Ok(unified_types.into_iter().next().unwrap());
            }

            Ok(Type::Union(unified_types))
        }

        // Intersection: compute GLB (intersection)
        (Type::Intersection(types1), Type::Intersection(types2)) => {
            let mut all_types: Vec<_> = types1.iter().chain(types2.iter()).cloned().collect();
            Ok(Type::Intersection(all_types))
        }

        // Intersection vs non-intersection
        (Type::Intersection(types), other) | (other, Type::Intersection(types)) => {
            let mut new_types = types.clone();
            new_types.push(other.clone());
            Ok(Type::Intersection(new_types))
        }

        // Task 6: Generic type unification
        // ================================

        // Generics: try to unify if base types match
        (Type::Generic { base: base1, args: args1 }, Type::Generic { base: base2, args: args2 }) => {
            if base1 != base2 {
                return Err(UnificationError::IncompatibleTypes(
                    format!("generic type with base {:?}", base1),
                    format!("generic type with base {:?}", base2),
                ));
            }

            if args1.len() != args2.len() {
                return Err(UnificationError::ArityMismatch {
                    expected: args1.len(),
                    actual: args2.len(),
                });
            }

            // Unify type arguments
            let unified_args: Result<Vec<_>, _> = args1
                .iter()
                .zip(args2.iter())
                .map(|(a1, a2)| unify(a1, a2, interner))
                .collect();

            Ok(Type::Generic {
                base: *base1,
                args: unified_args?,
            })
        }

        // Type parameters: unify to unknown (conservative)
        (Type::TypeParameter(_), _) | (_, Type::TypeParameter(_)) => {
            Ok(Type::Primitive(Unknown))
        }

        // References: unify names and type arguments
        (Type::Reference { name: name1, type_args: args1 }, Type::Reference { name: name2, type_args: args2 }) => {
            if name1 != name2 {
                return Err(UnificationError::IncompatibleTypes(
                    format!("type reference to {}", name1),
                    format!("type reference to {}", name2),
                ));
            }

            if args1.len() != args2.len() {
                return Err(UnificationError::ArityMismatch {
                    expected: args1.len(),
                    actual: args2.len(),
                });
            }

            let unified_args: Result<Vec<_>, _> = args1
                .iter()
                .zip(args2.iter())
                .map(|(a1, a2)| unify(a1, a2, interner))
                .collect();

            Ok(Type::Reference {
                name: name1.clone(),
                type_args: unified_args?,
            })
        }

        // Default: incompatible types
        _ => {
            Err(UnificationError::IncompatibleTypes(format!("{:?}", ty1), format!("{:?}", ty2)))
        }
    }
}

/// Check if a type is assignable to another type.
///
/// Returns true if a value of type `from` can be assigned to a variable of type `to`.
/// Assignability is more permissive than subtyping in some cases, particularly
/// for type inference scenarios where bidirectional checking is used.
///
/// # Parameters
///
/// * `from` - The source type being assigned from
/// * `to` - The target type being assigned to
/// * `interner` - The type interner for looking up type information
pub fn is_assignable(from: &Type, to: &Type, interner: &TypeInterner) -> bool {
    // Fast path: identical types are always assignable
    if from == to {
        return true;
    }

    // Use the existing is_subtype_internal function which handles all type combinations
    // Assignability in TypeScript is based on subtyping relationships
    is_subtype_internal(from, to, interner)
}
