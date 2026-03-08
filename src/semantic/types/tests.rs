//! Test suite for type system module.

use super::*;

#[test]
fn test_type_basic_types() {
    // Test basic TypeScript type representations
    let _string_ty = Type::Primitive(PrimitiveType::String);
    let _number_ty = Type::Primitive(PrimitiveType::Number);
    let _boolean_ty = Type::Primitive(PrimitiveType::Boolean);
    let _null_ty = Type::Primitive(PrimitiveType::Null);
    let _undefined_ty = Type::Primitive(PrimitiveType::Undefined);
    let _void_ty = Type::Primitive(PrimitiveType::Void);
    let _never_ty = Type::Primitive(PrimitiveType::Never);
    let _unknown_ty = Type::Primitive(PrimitiveType::Unknown);
    let _any_ty = Type::Primitive(PrimitiveType::Any);
}

#[test]
fn test_type_compatibility() {
    // Test type compatibility checks
    todo!("Implement type compatibility test");
}

#[test]
fn test_type_inference() {
    // Test type inference functionality
    todo!("Implement type inference test");
}

mod unify {
    use super::*;
    use fxhash::FxHashMap;

    #[test]
    fn test_primitive_subtyping() {
        let mut interner = TypeInterner::new();

        let string_id = interner.get_or_intern_primitive(PrimitiveType::String);
        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);
        let boolean_id = interner.get_or_intern_primitive(PrimitiveType::Boolean);
        let never_id = interner.get_or_intern_primitive(PrimitiveType::Never);
        let unknown_id = interner.get_or_intern_primitive(PrimitiveType::Unknown);
        let any_id = interner.get_or_intern_primitive(PrimitiveType::Any);

        // Reflexivity
        assert!(is_subtype(string_id, string_id, &interner));
        assert!(is_subtype(number_id, number_id, &interner));

        // Never is subtype of everything
        assert!(is_subtype(never_id, string_id, &interner));
        assert!(is_subtype(never_id, number_id, &interner));
        assert!(is_subtype(never_id, unknown_id, &interner));

        // Everything is subtype of unknown
        assert!(is_subtype(string_id, unknown_id, &interner));
        assert!(is_subtype(number_id, unknown_id, &interner));
        assert!(is_subtype(boolean_id, unknown_id, &interner));

        // Any is compatible with everything
        assert!(is_subtype(any_id, string_id, &interner));
        assert!(is_subtype(string_id, any_id, &interner));
        assert!(is_subtype(any_id, number_id, &interner));
        assert!(is_subtype(number_id, any_id, &interner));

        // Incompatible primitives
        assert!(!is_subtype(string_id, number_id, &interner));
        assert!(!is_subtype(number_id, boolean_id, &interner));
        assert!(!is_subtype(boolean_id, string_id, &interner));
    }

    #[test]
    fn test_union_subtyping() {
        let mut interner = TypeInterner::new();

        let string_id = interner.get_or_intern_primitive(PrimitiveType::String);
        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);
        let boolean_id = interner.get_or_intern_primitive(PrimitiveType::Boolean);

        // Union of string | number
        let string_or_number_id = interner.get_or_intern_union(vec![string_id, number_id]);

        // string <: string | number
        assert!(is_subtype(string_id, string_or_number_id, &interner));
        // number <: string | number
        assert!(is_subtype(number_id, string_or_number_id, &interner));
        // boolean <: string | number? No
        assert!(!is_subtype(boolean_id, string_or_number_id, &interner));

        // string | number <: string? No
        assert!(!is_subtype(string_or_number_id, string_id, &interner));

        // Union of string | number | boolean
        let string_number_bool_id = interner.get_or_intern_union(vec![string_id, number_id, boolean_id]);

        // string | number <: string | number | boolean
        assert!(is_subtype(string_or_number_id, string_number_bool_id, &interner));
        // string | number | boolean <: string | number? No
        assert!(!is_subtype(string_number_bool_id, string_or_number_id, &interner));
    }

    #[test]
    fn test_intersection_subtyping() {
        let mut interner = TypeInterner::new();

        // Create object types
        let obj1 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("name".to_string(), Type::Primitive(PrimitiveType::String));
                props
            },
            index_signature: None,
        });

        let obj2 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("age".to_string(), Type::Primitive(PrimitiveType::Number));
                props
            },
            index_signature: None,
        });

        let obj1_id = interner.intern(obj1.clone());
        let obj2_id = interner.intern(obj2.clone());

        // Intersection of {name: string} & {age: number}
        let intersection_id = interner.intern(Type::Intersection(vec![obj1, obj2]));

        // Intersection is subtype of each component
        assert!(is_subtype(intersection_id, obj1_id, &interner));
        assert!(is_subtype(intersection_id, obj2_id, &interner));

        // Components are not subtypes of intersection
        assert!(!is_subtype(obj1_id, intersection_id, &interner));
        assert!(!is_subtype(obj2_id, intersection_id, &interner));
    }

    #[test]
    fn test_array_subtyping() {
        let mut interner = TypeInterner::new();

        let string_id = interner.get_or_intern_primitive(PrimitiveType::String);
        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);
        let unknown_id = interner.get_or_intern_primitive(PrimitiveType::Unknown);

        let string_array_id = interner.get_or_intern_array(string_id);
        let number_array_id = interner.get_or_intern_array(number_id);
        let unknown_array_id = interner.get_or_intern_array(unknown_id);

        // Covariance: string[] <: unknown[]
        assert!(is_subtype(string_array_id, unknown_array_id, &interner));
        // number[] <: unknown[]
        assert!(is_subtype(number_array_id, unknown_array_id, &interner));
        // string[] <: number[]? No
        assert!(!is_subtype(string_array_id, number_array_id, &interner));
    }

    #[test]
    fn test_tuple_subtyping() {
        let mut interner = TypeInterner::new();

        let string_id = interner.get_or_intern_primitive(PrimitiveType::String);
        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);
        let boolean_id = interner.get_or_intern_primitive(PrimitiveType::Boolean);
        let unknown_id = interner.get_or_intern_primitive(PrimitiveType::Unknown);

        // [string, number]
        let tuple1_id = interner.intern(Type::Tuple(vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
        ]));

        // [unknown, unknown]
        let tuple2_id = interner.intern(Type::Tuple(vec![
            Type::Primitive(PrimitiveType::Unknown),
            Type::Primitive(PrimitiveType::Unknown),
        ]));

        // [string, number, boolean]
        let tuple3_id = interner.intern(Type::Tuple(vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
            Type::Primitive(PrimitiveType::Boolean),
        ]));

        // [string, number] <: [unknown, unknown]
        assert!(is_subtype(tuple1_id, tuple2_id, &interner));
        // [unknown, unknown] <: [string, number]? No
        assert!(!is_subtype(tuple2_id, tuple1_id, &interner));
        // Different lengths are incompatible
        assert!(!is_subtype(tuple1_id, tuple3_id, &interner));
        assert!(!is_subtype(tuple3_id, tuple1_id, &interner));
    }

    #[test]
    fn test_object_subtyping() {
        let mut interner = TypeInterner::new();

        // Object with name: string
        let obj1 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("name".to_string(), Type::Primitive(PrimitiveType::String));
                props
            },
            index_signature: None,
        });

        // Object with name: string, age: number (extra property)
        let obj2 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("name".to_string(), Type::Primitive(PrimitiveType::String));
                props.insert("age".to_string(), Type::Primitive(PrimitiveType::Number));
                props
            },
            index_signature: None,
        });

        // Object with name: number (wrong type)
        let obj3 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("name".to_string(), Type::Primitive(PrimitiveType::Number));
                props
            },
            index_signature: None,
        });

        // Object with index signature
        let obj4 = Type::Object(ObjectType {
            properties: FxHashMap::default(),
            index_signature: Some(Box::new(Type::Primitive(PrimitiveType::String))),
        });

        // Object with string properties matching index signature
        let obj5 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("a".to_string(), Type::Primitive(PrimitiveType::String));
                props.insert("b".to_string(), Type::Primitive(PrimitiveType::String));
                props
            },
            index_signature: None,
        });

        let obj1_id = interner.intern(obj1);
        let obj2_id = interner.intern(obj2);
        let obj3_id = interner.intern(obj3);
        let obj4_id = interner.intern(obj4);
        let obj5_id = interner.intern(obj5);

        // Extra properties allowed: obj2 <: obj1
        assert!(is_subtype(obj2_id, obj1_id, &interner));
        // Missing property: obj1 <: obj2? No
        assert!(!is_subtype(obj1_id, obj2_id, &interner));
        // Wrong property type: obj3 <: obj1? No
        assert!(!is_subtype(obj3_id, obj1_id, &interner));
        // Object with matching properties conforms to index signature
        assert!(is_subtype(obj5_id, obj4_id, &interner));
    }

    #[test]
    fn test_function_subtyping() {
        let mut interner = TypeInterner::new();

        let string_id = interner.get_or_intern_primitive(PrimitiveType::String);
        let _number_id = interner.get_or_intern_primitive(PrimitiveType::Number);
        let _unknown_id = interner.get_or_intern_primitive(PrimitiveType::Unknown);

        // Function type: (string) => number
        let func1 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::String)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Number)),
            type_params: vec![],
        };

        // Function type: (unknown) => number
        let func2 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::Unknown)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Number)),
            type_params: vec![],
        };

        // Function type: (string) => unknown
        let func3 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::String)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Unknown)),
            type_params: vec![],
        };

        let func1_id = interner.intern(func1);
        let func2_id = interner.intern(func2);
        let func3_id = interner.intern(func3);

        // Parameter contravariance: (unknown) => R <: (string) => R
        assert!(is_subtype(func2_id, func1_id, &interner));
        // (string) => R <: (unknown) => R? No
        assert!(!is_subtype(func1_id, func2_id, &interner));

        // Return type covariance: () => string <: () => unknown
        assert!(is_subtype(func1_id, func3_id, &interner));
        // () => unknown <: () => string? No
        assert!(!is_subtype(func3_id, func1_id, &interner));
    }

    #[test]
    fn test_type_substitution() {
        let mut interner = TypeInterner::new();

        // Create a type parameter T
        let t_param = Type::TypeParameter(TypeParameter {
            name: "T".to_string(),
            constraint: None,
            default: None,
        });
        let t_id = interner.intern(t_param);

        // Create a type U
        let u_param = Type::TypeParameter(TypeParameter {
            name: "U".to_string(),
            constraint: None,
            default: None,
        });
        let u_id = interner.intern(u_param);

        // Create concrete types
        let string_id = interner.get_or_intern_primitive(PrimitiveType::String);
        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);

        // Create a generic type: Array<T>
        let array_of_t = Type::Array(Box::new(Type::TypeParameter(TypeParameter {
            name: "T".to_string(),
            constraint: None,
            default: None,
        })));
        let array_of_t_id = interner.intern(array_of_t);

        // Create substitution map: T -> string, U -> number
        let mut substitutions = FxHashMap::default();
        substitutions.insert(t_id, string_id);
        substitutions.insert(u_id, number_id);

        // Substitute T in T -> string
        let substituted_t = substitute_type_params(t_id, &substitutions, &mut interner);
        assert_eq!(substituted_t, string_id);

        // Substitute T in Array<T> -> Array<string>
        let substituted_array = substitute_type_params(array_of_t_id, &substitutions, &mut interner);
        let expected_array = interner.get_or_intern_array(string_id);
        assert_eq!(substituted_array, expected_array);

        // Create function type: (param: T) => U
        let func_type = Type::Function {
            params: vec![Type::TypeParameter(TypeParameter {
                name: "T".to_string(),
                constraint: None,
                default: None,
            })],
            return_type: Box::new(Type::TypeParameter(TypeParameter {
                name: "U".to_string(),
                constraint: None,
                default: None,
            })),
            type_params: vec![],
        };
        let func_id = interner.intern(func_type);

        // Substitute should give (param: string) => number
        let substituted_func = substitute_type_params(func_id, &substitutions, &mut interner);
        let expected_func = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::String)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Number)),
            type_params: vec![],
        };
        let expected_func_id = interner.intern(expected_func);
        assert_eq!(substituted_func, expected_func_id);
    }

    #[test]
    fn test_unify_primitives() {
        let mut interner = TypeInterner::new();

        // Identical primitives unify to themselves
        let string = Type::Primitive(PrimitiveType::String);
        assert_eq!(unify(&string, &string, &mut interner).unwrap(), string);

        // Any absorbs everything
        let any = Type::Primitive(PrimitiveType::Any);
        let number = Type::Primitive(PrimitiveType::Number);
        assert_eq!(unify(&any, &number, &mut interner).unwrap(), any);
        assert_eq!(unify(&number, &any, &mut interner).unwrap(), any);

        // Unknown is LUB of everything
        let unknown = Type::Primitive(PrimitiveType::Unknown);
        assert_eq!(unify(&unknown, &number, &mut interner).unwrap(), unknown);
        assert_eq!(unify(&number, &unknown, &mut interner).unwrap(), unknown);

        // Never is LUB with itself and any other type returns the other type
        let never = Type::Primitive(PrimitiveType::Never);
        assert_eq!(unify(&never, &string, &mut interner).unwrap(), string);
        assert_eq!(unify(&string, &never, &mut interner).unwrap(), string);

        // Different primitives are incompatible
        let result = unify(&string, &number, &mut interner);
        assert!(result.is_err());
        assert!(matches!(result, Err(UnificationError::IncompatibleTypes(_, _))));
    }

    #[test]
    fn test_unify_arrays() {
        let mut interner = TypeInterner::new();

        // Identical element types
        let string_array1 = Type::Array(Box::new(Type::Primitive(PrimitiveType::String)));
        let string_array2 = Type::Array(Box::new(Type::Primitive(PrimitiveType::String)));
        assert_eq!(unify(&string_array1, &string_array2, &mut interner).unwrap(), string_array1);

        // Compatible element types (string and unknown)
        let string_array = Type::Array(Box::new(Type::Primitive(PrimitiveType::String)));
        let unknown_array = Type::Array(Box::new(Type::Primitive(PrimitiveType::Unknown)));
        // This should unify to unknown[]
        let result = unify(&string_array, &unknown_array, &mut interner).unwrap();
        if let Type::Array(elem) = result {
            assert_eq!(*elem, Type::Primitive(PrimitiveType::Unknown));
        } else {
            panic!("Expected Array type");
        }
    }

    #[test]
    fn test_unify_tuples() {
        let mut interner = TypeInterner::new();

        // Identical tuples
        let tuple1 = Type::Tuple(vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
        ]);
        let tuple2 = Type::Tuple(vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
        ]);
        assert_eq!(unify(&tuple1, &tuple2, &mut interner).unwrap(), tuple1);

        // Different lengths are incompatible
        let tuple_short = Type::Tuple(vec![Type::Primitive(PrimitiveType::String)]);
        let result = unify(&tuple1, &tuple_short, &mut interner);
        assert!(result.is_err());

        // Tuples with compatible element types
        let tuple_unknown = Type::Tuple(vec![
            Type::Primitive(PrimitiveType::Unknown),
            Type::Primitive(PrimitiveType::Unknown),
        ]);
        let result = unify(&tuple1, &tuple_unknown, &mut interner).unwrap();
        if let Type::Tuple(elems) = result {
            assert_eq!(elems.len(), 2);
            assert_eq!(elems[0], Type::Primitive(PrimitiveType::Unknown));
            assert_eq!(elems[1], Type::Primitive(PrimitiveType::Unknown));
        } else {
            panic!("Expected Tuple type");
        }
    }

    #[test]
    fn test_unify_objects() {
        let mut interner = TypeInterner::new();

        // Objects with overlapping properties
        let obj1 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("name".to_string(), Type::Primitive(PrimitiveType::String));
                props
            },
            index_signature: None,
        });

        let obj2 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("age".to_string(), Type::Primitive(PrimitiveType::Number));
                props
            },
            index_signature: None,
        });

        let result = unify(&obj1, &obj2, &mut interner).unwrap();
        if let Type::Object(obj) = result {
            assert!(obj.properties.contains_key("name"));
            assert!(obj.properties.contains_key("age"));
            assert_eq!(
                obj.properties.get("name").unwrap(),
                &Type::Primitive(PrimitiveType::String)
            );
            assert_eq!(
                obj.properties.get("age").unwrap(),
                &Type::Primitive(PrimitiveType::Number)
            );
        } else {
            panic!("Expected Object type");
        }

        // Objects with index signatures
        let obj_with_index = Type::Object(ObjectType {
            properties: FxHashMap::default(),
            index_signature: Some(Box::new(Type::Primitive(PrimitiveType::String))),
        });

        let result = unify(&obj1, &obj_with_index, &mut interner).unwrap();
        if let Type::Object(obj) = result {
            assert!(obj.properties.contains_key("name"));
            assert!(obj.index_signature.is_some());
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_unify_functions() {
        let mut interner = TypeInterner::new();

        // Identical functions
        let func1 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::String)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Number)),
            type_params: vec![],
        };
        let func2 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::String)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Number)),
            type_params: vec![],
        };
        assert_eq!(unify(&func1, &func2, &mut interner).unwrap(), func1);

        // Functions with different parameter counts are incompatible
        let func_with_two_params = Type::Function {
            params: vec![
                Type::Primitive(PrimitiveType::String),
                Type::Primitive(PrimitiveType::Number),
            ],
            return_type: Box::new(Type::Primitive(PrimitiveType::Boolean)),
            type_params: vec![],
        };
        let result = unify(&func1, &func_with_two_params, &mut interner);
        assert!(result.is_err());

        // Functions with compatible return types
        let func_return_unknown = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::String)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Unknown)),
            type_params: vec![],
        };
        let result = unify(&func1, &func_return_unknown, &mut interner).unwrap();
        if let Type::Function { return_type, .. } = result {
            assert_eq!(*return_type, Type::Primitive(PrimitiveType::Unknown));
        } else {
            panic!("Expected Function type");
        }
    }

    #[test]
    fn test_unify_unions() {
        let mut interner = TypeInterner::new();

        let string = Type::Primitive(PrimitiveType::String);
        let number = Type::Primitive(PrimitiveType::Number);
        let boolean = Type::Primitive(PrimitiveType::Boolean);

        // Union vs non-union
        let union1 = Type::Union(vec![string.clone()]);
        assert_eq!(unify(&union1, &string, &mut interner).unwrap(), string);

        // Union of string | number
        let union2 = Type::Union(vec![string.clone(), number.clone()]);
        let result = unify(&union2, &string, &mut interner).unwrap();
        // Should keep both string and number
        if let Type::Union(types) = result {
            assert_eq!(types.len(), 2);
        } else {
            panic!("Expected Union type");
        }

        // Union vs Union - flatten and merge
        let union3 = Type::Union(vec![number.clone(), boolean.clone()]);
        let result = unify(&union2, &union3, &mut interner).unwrap();
        if let Type::Union(types) = result {
            // Should contain string, number, boolean
            assert!(types.contains(&string));
            assert!(types.contains(&number));
            assert!(types.contains(&boolean));
        } else {
            panic!("Expected Union type");
        }
    }

    #[test]
    fn test_unify_intersections() {
        let mut interner = TypeInterner::new();

        let obj1 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("name".to_string(), Type::Primitive(PrimitiveType::String));
                props
            },
            index_signature: None,
        });

        let obj2 = Type::Object(ObjectType {
            properties: {
                let mut props = FxHashMap::default();
                props.insert("age".to_string(), Type::Primitive(PrimitiveType::Number));
                props
            },
            index_signature: None,
        });

        // Intersection of objects
        let intersection1 = Type::Intersection(vec![obj1.clone()]);
        let result = unify(&intersection1, &obj2, &mut interner).unwrap();
        if let Type::Intersection(types) = result {
            // Should contain both objects
            assert_eq!(types.len(), 2);
        } else {
            panic!("Expected Intersection type");
        }
    }

    #[test]
    fn test_unify_generics() {
        let mut interner = TypeInterner::new();

        let string = Type::Primitive(PrimitiveType::String);
        let unknown = Type::Primitive(PrimitiveType::Unknown);

        // Create base type IDs (using different primitives to get different IDs)
        let base1_id = interner.intern(Type::Primitive(PrimitiveType::String));
        let base2_id = interner.intern(Type::Primitive(PrimitiveType::Number));

        // Identical generics
        let generic1 = Type::Generic {
            base: base1_id,
            args: vec![Type::Primitive(PrimitiveType::Number)],
        };
        let generic2 = Type::Generic {
            base: base1_id,
            args: vec![Type::Primitive(PrimitiveType::Number)],
        };
        assert_eq!(unify(&generic1, &generic2, &mut interner).unwrap(), generic1);

        // Generics with compatible argument types (using unknown)
        let generic_with_unknown = Type::Generic {
            base: base1_id,
            args: vec![unknown.clone()],
        };
        let result = unify(&generic1, &generic_with_unknown, &mut interner).unwrap();
        if let Type::Generic { args, .. } = result {
            assert_eq!(args.len(), 1);
            // Should be unified to unknown
            assert_eq!(args[0], Type::Primitive(PrimitiveType::Unknown));
        } else {
            panic!("Expected Generic type");
        }

        // Different base types are incompatible
        let generic_different_base = Type::Generic {
            base: base2_id,
            args: vec![Type::Primitive(PrimitiveType::Number)],
        };
        let result = unify(&generic1, &generic_different_base, &mut interner);
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_type_parameters() {
        let mut interner = TypeInterner::new();

        let string = Type::Primitive(PrimitiveType::String);
        let type_param = Type::TypeParameter(TypeParameter {
            name: "T".to_string(),
            constraint: None,
            default: None,
        });

        // Type parameter with concrete type unifies to unknown (conservative)
        let result = unify(&string, &type_param, &mut interner).unwrap();
        assert_eq!(result, Type::Primitive(PrimitiveType::Unknown));
    }

    #[test]
    fn test_unify_references() {
        let mut interner = TypeInterner::new();

        let string = Type::Primitive(PrimitiveType::String);

        // Identical references
        let ref1 = Type::Reference {
            name: "MyType".to_string(),
            type_args: vec![string.clone()],
        };
        let ref2 = Type::Reference {
            name: "MyType".to_string(),
            type_args: vec![string.clone()],
        };
        assert_eq!(unify(&ref1, &ref2, &mut interner).unwrap(), ref1);

        // Different names are incompatible
        let ref_different_name = Type::Reference {
            name: "OtherType".to_string(),
            type_args: vec![],
        };
        let result = unify(&ref1, &ref_different_name, &mut interner);
        assert!(result.is_err());
    }
}

mod representation {
    use super::*;
    use fxhash::FxHashMap;

    #[test]
    fn test_primitive_types() {
        let string_ty = Type::Primitive(PrimitiveType::String);
        let number_ty = Type::Primitive(PrimitiveType::Number);

        assert_ne!(string_ty, number_ty);
        assert_eq!(string_ty, Type::Primitive(PrimitiveType::String));
    }

    #[test]
    fn test_array_type() {
        let element = Type::Primitive(PrimitiveType::Number);
        let array_ty = Type::Array(Box::new(element.clone()));

        if let Type::Array(elem) = array_ty {
            assert_eq!(*elem, element);
        } else {
            panic!("Expected Array type");
        }
    }

    #[test]
    fn test_tuple_type() {
        let types = vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
            Type::Primitive(PrimitiveType::Boolean),
        ];
        let tuple_ty = Type::Tuple(types.clone());

        if let Type::Tuple(elems) = tuple_ty {
            assert_eq!(elems, types);
        } else {
            panic!("Expected Tuple type");
        }
    }

    #[test]
    fn test_object_type() {
        let mut properties = FxHashMap::default();
        properties.insert("name".to_string(), Type::Primitive(PrimitiveType::String));
        properties.insert("age".to_string(), Type::Primitive(PrimitiveType::Number));

        let index_sig = Some(Box::new(Type::Primitive(PrimitiveType::String)));

        let object_ty = Type::Object(ObjectType {
            properties: properties.clone(),
            index_signature: index_sig.clone(),
        });

        if let Type::Object(obj) = object_ty {
            assert_eq!(obj.properties, properties);
            assert_eq!(obj.index_signature, index_sig);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_function_type() {
        let params = vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
        ];
        let return_type = Box::new(Type::Primitive(PrimitiveType::Boolean));
        let type_params = vec![TypeParameter {
            name: "T".to_string(),
            constraint: None,
            default: None,
        }];

        let func_ty = Type::Function {
            params: params.clone(),
            return_type: return_type.clone(),
            type_params: type_params.clone(),
        };

        if let Type::Function { params: p, return_type: r, type_params: tp } = func_ty {
            assert_eq!(p, params);
            assert_eq!(r, return_type);
            assert_eq!(tp, type_params);
        } else {
            panic!("Expected Function type");
        }
    }

    #[test]
    fn test_union_type() {
        let types = vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
        ];
        let union_ty = Type::Union(types.clone());

        if let Type::Union(elems) = union_ty {
            assert_eq!(elems, types);
        } else {
            panic!("Expected Union type");
        }
    }

    #[test]
    fn test_intersection_type() {
        let types = vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
        ];
        let intersection_ty = Type::Intersection(types.clone());

        if let Type::Intersection(elems) = intersection_ty {
            assert_eq!(elems, types);
        } else {
            panic!("Expected Intersection type");
        }
    }

    #[test]
    fn test_type_parameter() {
        let tp = TypeParameter {
            name: "T".to_string(),
            constraint: Some(Box::new(Type::Primitive(PrimitiveType::String))),
            default: Some(Box::new(Type::Primitive(PrimitiveType::String))),
        };

        let ty = Type::TypeParameter(tp.clone());

        if let Type::TypeParameter(param) = ty {
            assert_eq!(param.name, "T");
            assert!(param.constraint.is_some());
            assert!(param.default.is_some());
        } else {
            panic!("Expected TypeParameter type");
        }
    }

    #[test]
    fn test_generic_type() {
        // We need a dummy TypeId for testing - create via interner
        let mut interner = TypeInterner::new();
        let base_id = interner.intern(Type::Primitive(PrimitiveType::Any));
        let args = vec![
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Number),
        ];

        let generic_ty = Type::Generic {
            base: base_id,
            args: args.clone(),
        };

        if let Type::Generic { base, args: a } = generic_ty {
            assert_eq!(base, base_id);
            assert_eq!(a, args);
        } else {
            panic!("Expected Generic type");
        }
    }

    #[test]
    fn test_reference_type() {
        let name = "MyType".to_string();
        let type_args = vec![Type::Primitive(PrimitiveType::String)];

        let ref_ty = Type::Reference {
            name: name.clone(),
            type_args: type_args.clone(),
        };

        if let Type::Reference { name: n, type_args: ta } = ref_ty {
            assert_eq!(n, name);
            assert_eq!(ta, type_args);
        } else {
            panic!("Expected Reference type");
        }
    }
}

mod interner {
    use super::*;
    use fxhash::FxHashMap;

    #[test]
    fn test_intern_primitive() {
        let mut interner = TypeInterner::new();

        let string_id1 = interner.get_or_intern_primitive(PrimitiveType::String);
        let string_id2 = interner.get_or_intern_primitive(PrimitiveType::String);
        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);

        assert_eq!(string_id1, string_id2);
        assert_ne!(string_id1, number_id);

        let string_ty = interner.get(string_id1).unwrap();
        assert_eq!(*string_ty, Type::Primitive(PrimitiveType::String));
    }

    #[test]
    fn test_intern_array() {
        let mut interner = TypeInterner::new();

        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);
        let array_id1 = interner.get_or_intern_array(number_id);
        let array_id2 = interner.get_or_intern_array(number_id);

        assert_eq!(array_id1, array_id2);

        let array_ty = interner.get(array_id1).unwrap();
        if let Type::Array(elem) = array_ty {
            assert_eq!(**elem, Type::Primitive(PrimitiveType::Number));
        } else {
            panic!("Expected Array type");
        }
    }

    #[test]
    fn test_intern_union() {
        let mut interner = TypeInterner::new();

        let string_id = interner.get_or_intern_primitive(PrimitiveType::String);
        let number_id = interner.get_or_intern_primitive(PrimitiveType::Number);
        let boolean_id = interner.get_or_intern_primitive(PrimitiveType::Boolean);

        // Test union deduplication
        let union_id1 = interner.get_or_intern_union(vec![string_id, number_id, string_id]);
        let union_id2 = interner.get_or_intern_union(vec![number_id, string_id]);

        assert_eq!(union_id1, union_id2);

        // Test single type union returns the type itself
        let single_union_id = interner.get_or_intern_union(vec![string_id]);
        assert_eq!(single_union_id, string_id);

        // Test empty union returns never
        let empty_union_id = interner.get_or_intern_union(vec![]);
        let never_id = interner.get_or_intern_primitive(PrimitiveType::Never);
        assert_eq!(empty_union_id, never_id);

        // Test union with 3 types
        let union_id3 = interner.get_or_intern_union(vec![string_id, number_id, boolean_id]);
        let union_ty = interner.get(union_id3).unwrap();

        if let Type::Union(types) = union_ty {
            assert_eq!(types.len(), 3);
            // Types are sorted by discriminant order: String, Number, Boolean
            assert_eq!(types[0], Type::Primitive(PrimitiveType::String));
            assert_eq!(types[1], Type::Primitive(PrimitiveType::Number));
            assert_eq!(types[2], Type::Primitive(PrimitiveType::Boolean));
        } else {
            panic!("Expected Union type");
        }
    }

    #[test]
    fn test_intern_deduplication() {
        let mut interner = TypeInterner::new();

        // Create identical object types
        let mut props1 = FxHashMap::default();
        props1.insert("a".to_string(), Type::Primitive(PrimitiveType::Number));
        props1.insert("b".to_string(), Type::Primitive(PrimitiveType::String));

        let mut props2 = FxHashMap::default();
        props2.insert("b".to_string(), Type::Primitive(PrimitiveType::String));
        props2.insert("a".to_string(), Type::Primitive(PrimitiveType::Number));

        let obj1 = Type::Object(ObjectType {
            properties: props1,
            index_signature: None,
        });

        let obj2 = Type::Object(ObjectType {
            properties: props2,
            index_signature: None,
        });

        let id1 = interner.intern(obj1);
        let id2 = interner.intern(obj2);

        assert_eq!(id1, id2);
    }
}