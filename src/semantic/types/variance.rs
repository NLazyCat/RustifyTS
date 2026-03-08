//! Variance tracking for generic types.
//!
//! This module implements variance annotations for generic type parameters,
//! supporting covariant, contravariant, invariant, and bivariant variance.

use fxhash::FxHashMap;

/// Variance of a generic type parameter.
///
/// Variance describes how a generic type's subtyping relationship
/// relates to the subtyping relationships of its type parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variance {
    /// Covariant: `T <: U` implies `Generic<T> <: Generic<U>`
    ///
    /// Most read-only containers are covariant (e.g., Array, Promise, Readonly).
    Covariant,

    /// Contravariant: `T <: U` implies `Generic<U> <: Generic<T>`
    ///
    /// Only function parameters (in certain contexts) are contravariant.
    Contravariant,

    /// Invariant: Both must be equal for subtyping
    ///
    /// Most write-capable containers are invariant (e.g., mutable boxes).
    Invariant,

    /// Bivariant: Allows subtyping in both directions
    ///
    /// Rare, mostly for TypeScript legacy compatibility (e.g., Function type parameters).
    Bivariant,
}

/// Registry mapping generic type names to their variance profiles.
///
/// This registry stores the variance rules for built-in and user-defined
/// generic types, allowing the type system to correctly determine subtyping
/// relationships for generic type instantiations.
pub struct VarianceRegistry {
    /// Map from type name to variance profile
    variances: FxHashMap<String, Vec<Variance>>,
}

impl VarianceRegistry {
    /// Create a new variance registry with built-in TypeScript types populated.
    pub fn new() -> Self {
        let mut registry = Self {
            variances: FxHashMap::default(),
        };
        registry.populate_builtins();
        registry
    }

    /// Register variance for a generic type.
    pub fn register(&mut self, name: String, variances: Vec<Variance>) {
        self.variances.insert(name, variances);
    }

    /// Get variance for a generic type by name.
    pub fn get(&self, name: &str) -> Option<&[Variance]> {
        self.variances.get(name).map(|v| v.as_slice())
    }

    /// Populate the registry with built-in TypeScript types.
    fn populate_builtins(&mut self) {
        // Array<T>: covariant in element type
        self.register("Array".to_string(), vec![Variance::Covariant]);

        // ReadonlyArray<T>: covariant in element type
        self.register("ReadonlyArray".to_string(), vec![Variance::Covariant]);

        // Promise<T>: covariant in value type
        self.register("Promise".to_string(), vec![Variance::Covariant]);

        // Map<K, V>: covariant in both key and value types
        self.register("Map".to_string(), vec![Variance::Covariant, Variance::Covariant]);

        // Set<T>: covariant in element type
        self.register("Set".to_string(), vec![Variance::Covariant]);

        // Function type parameters are bivariant for TypeScript legacy reasons
        // This is handled separately in the subtype checking logic

        // Readonly<T>: covariant
        self.register("Readonly".to_string(), vec![Variance::Covariant]);

        // Record<K, V>: covariant in both key and value types
        self.register("Record".to_string(), vec![Variance::Covariant, Variance::Covariant]);

        // Partial<T>: covariant
        self.register("Partial".to_string(), vec![Variance::Covariant]);

        // Required<T>: covariant
        self.register("Required".to_string(), vec![Variance::Covariant]);

        // Pick<T, K>: covariant in T, invariant in K (keys)
        self.register("Pick".to_string(), vec![Variance::Covariant, Variance::Invariant]);

        // Omit<T, K>: covariant in T, invariant in K (keys)
        self.register("Omit".to_string(), vec![Variance::Covariant, Variance::Invariant]);
    }
}

impl Default for VarianceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = VarianceRegistry::new();

        // Test Array variance
        assert_eq!(registry.get("Array"), Some(&[Variance::Covariant][..]));

        // Test Promise variance
        assert_eq!(registry.get("Promise"), Some(&[Variance::Covariant][..]));

        // Test Map variance
        assert_eq!(registry.get("Map"), Some(&[Variance::Covariant, Variance::Covariant][..]));

        // Test unknown type
        assert_eq!(registry.get("UnknownType"), None);
    }

    #[test]
    fn test_custom_registration() {
        let mut registry = VarianceRegistry::new();

        // Register a custom generic type
        registry.register("MyGeneric".to_string(), vec![
            Variance::Covariant,
            Variance::Contravariant,
            Variance::Invariant,
        ]);

        assert_eq!(registry.get("MyGeneric"), Some(&[
            Variance::Covariant,
            Variance::Contravariant,
            Variance::Invariant,
        ][..]));
    }

    #[test]
    fn test_variance_equality() {
        assert_eq!(Variance::Covariant, Variance::Covariant);
        assert_ne!(Variance::Covariant, Variance::Contravariant);
        assert_ne!(Variance::Invariant, Variance::Bivariant);
    }
}
