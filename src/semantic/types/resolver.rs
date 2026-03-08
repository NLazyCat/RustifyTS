//! Type resolution for named type references.
//!
//! This module handles resolving named type references (e.g., `interface MyType`)
//! to their actual type definitions in the current scope.

use super::*;
use crate::parser::ast::types::{Parameter, TypeAnnotation};
use crate::parser::ast::visitor::Visitor;
use crate::parser::ast::{AstNode, NodeKind};
use crate::parser::ast::types::VariableDeclaration;
use crate::semantic::scope::{ScopeId, ScopeTable};
use crate::semantic::symbol::{SymbolKind, SymbolTable};
use fxhash::FxHashMap;
use std::cell::RefCell;

/// Type resolver that resolves type references to their actual types.
///
/// The TypeResolver traverses the AST and resolves all type references, storing
/// the resolved TypeId in the corresponding symbol table entries.
pub struct TypeResolver<'a> {
    /// Symbol table for looking up and updating type definitions
    symbol_table: &'a mut SymbolTable,
    /// Scope table for traversing the scope hierarchy
    scope_table: &'a ScopeTable,
    /// Type interner for creating new types
    type_interner: &'a mut TypeInterner,
    /// Current scope during traversal
    current_scope: ScopeId,
    /// Cache of resolved type references to avoid redundant work
    resolution_cache: FxHashMap<(String, ScopeId), TypeId>,
    /// Set of currently resolving types to detect cycles
    resolving: RefCell<FxHashMap<String, ScopeId>>,
}

impl<'a> TypeResolver<'a> {
    /// Create a new TypeResolver with the given context.
    pub fn new(
        symbol_table: &'a mut SymbolTable,
        scope_table: &'a ScopeTable,
        type_interner: &'a mut TypeInterner,
        root_scope: ScopeId,
    ) -> Self {
        Self {
            symbol_table,
            scope_table,
            type_interner,
            current_scope: root_scope,
            resolution_cache: FxHashMap::default(),
            resolving: RefCell::new(FxHashMap::default()),
        }
    }

    /// Resolve a type reference to its actual type.
    ///
    /// Takes a Type::Reference variant and resolves it to the corresponding
    /// type definition in the current scope hierarchy.
    pub fn resolve_type_reference(&mut self, reference: &Type) -> ResolutionResult<TypeId> {
        let (name, type_args) = match reference {
            Type::Reference { name, type_args } => (name, type_args),
            _ => return Err(ResolutionError::NotAType("Not a type reference".to_string())),
        };

        // Check cache first
        let cache_key = (name.clone(), self.current_scope);
        if let Some(&resolved_id) = self.resolution_cache.get(&cache_key) {
            return Ok(resolved_id);
        }

        // Check for cycles
        let mut resolving = self.resolving.borrow_mut();
        if resolving.contains_key(name) && resolving[name] == self.current_scope {
            return Err(ResolutionError::RecursiveReference(name.clone()));
        }
        resolving.insert(name.clone(), self.current_scope);
        drop(resolving);

        // Look up the type in the symbol table
        let symbol_id = self.symbol_table.lookup_lexical(
            name,
            self.current_scope,
            self.scope_table,
        ).ok_or_else(|| ResolutionError::TypeNotFound(name.clone()))?;

        let symbol = self.symbol_table.lookup(symbol_id)
            .ok_or_else(|| ResolutionError::TypeNotFound(name.clone()))?;

        // Check that the symbol is actually a type
        match symbol.kind() {
            SymbolKind::Interface | SymbolKind::TypeAlias | SymbolKind::Enum => {
                // Get the type ID from the symbol
                let type_id = symbol.type_id()
                    .ok_or_else(|| ResolutionError::NotAType(name.clone()))?;

                // Resolve type arguments if this is a generic type
                let resolved_args = self.resolve_type_args(type_args)?;

                // If there are type arguments, apply them to the generic type
                let final_type_id = if !resolved_args.is_empty() {
                    // Clone type_id before borrowing interner immutably
                    let base_type_id = type_id;

                    // First get the base type and check what kind it is
                    let base_type = self.type_interner.get(base_type_id)
                        .ok_or_else(|| ResolutionError::TypeNotFound(name.clone()))?;

                    match base_type {
                        Type::Function { type_params, .. } => {
                            if type_params.len() != resolved_args.len() {
                                return Err(ResolutionError::GenericArityMismatch {
                                    name: name.clone(),
                                    expected: type_params.len(),
                                    actual: resolved_args.len(),
                                });
                            }

                            // Clone type_params to release immutable borrow of interner
                            let type_params_cloned = type_params.clone();

                            // Create substitution map
                            let mut substitutions = FxHashMap::default();
                            for (param, arg) in type_params_cloned.iter().zip(resolved_args.iter()) {
                                // We need to get the TypeId of the parameter
                                let param_id = self.type_interner.intern(Type::TypeParameter(param.clone()));
                                substitutions.insert(param_id, *arg);
                            }

                            // Substitute type parameters in the base type
                            substitute_type_params(base_type_id, &substitutions, self.type_interner)
                        }
                        Type::TypeParameter(_) => {
                            // If base is a type parameter, create a generic instantiation
                            let arg_types: Vec<_> = resolved_args.iter()
                                .map(|&id| self.type_interner.get(id).unwrap().clone())
                                .collect();
                            self.type_interner.intern(Type::Generic {
                                base: base_type_id,
                                args: arg_types,
                            })
                        }
                        _ => {
                            // If the base type isn't generic but we have arguments, it's an error
                            return Err(ResolutionError::GenericArityMismatch {
                                name: name.clone(),
                                expected: 0,
                                actual: resolved_args.len(),
                            });
                        }
                    }
                } else {
                    type_id
                };

                // Cache the result
                self.resolution_cache.insert(cache_key, final_type_id);

                // Remove from resolving set
                let mut resolving = self.resolving.borrow_mut();
                resolving.remove(name);

                Ok(final_type_id)
            }
            _ => Err(ResolutionError::NotAType(name.clone())),
        }
    }

    /// Resolve a list of type arguments.
    fn resolve_type_args(&mut self, type_args: &[Type]) -> ResolutionResult<Vec<TypeId>> {
        let mut resolved = Vec::with_capacity(type_args.len());
        for arg in type_args {
            let resolved_arg = self.resolve_type(arg)?;
            resolved.push(resolved_arg);
        }
        Ok(resolved)
    }

    /// Resolve any type, recursively resolving references.
    pub fn resolve_type(&mut self, ty: &Type) -> ResolutionResult<TypeId> {
        match ty {
            Type::Reference { .. } => self.resolve_type_reference(ty),

            // Recursively resolve types that contain other types
            Type::Array(elem) => {
                let resolved_elem = self.resolve_type(elem)?;
                Ok(self.type_interner.get_or_intern_array(resolved_elem))
            }

            Type::Tuple(elems) => {
                let resolved_elems: Result<Vec<_>, _> = elems.iter()
                    .map(|elem| self.resolve_type(elem))
                    .collect();
                let resolved_elems = resolved_elems?;

                // Convert resolved TypeIds back to Type values for the tuple
                let tuple_types: Vec<_> = resolved_elems.iter()
                    .map(|&id| self.type_interner.get(id).unwrap().clone())
                    .collect();

                Ok(self.type_interner.intern(Type::Tuple(tuple_types)))
            }

            Type::Object(obj) => {
                let mut resolved_properties = FxHashMap::default();
                for (name, prop_ty) in &obj.properties {
                    let resolved_prop = self.resolve_type(prop_ty)?;
                    resolved_properties.insert(
                        name.clone(),
                        self.type_interner.get(resolved_prop).unwrap().clone()
                    );
                }

                let resolved_index = obj.index_signature.as_ref()
                    .map(|sig| self.resolve_type(sig))
                    .transpose()?
                    .map(|id| Box::new(self.type_interner.get(id).unwrap().clone()));

                Ok(self.type_interner.intern(Type::Object(ObjectType {
                    properties: resolved_properties,
                    index_signature: resolved_index,
                })))
            }

            Type::Function { params, return_type, type_params } => {
                let resolved_params: Result<Vec<_>, _> = params.iter()
                    .map(|param| self.resolve_type(param))
                    .collect();
                let resolved_params = resolved_params?;
                let param_types: Vec<_> = resolved_params.iter()
                    .map(|&id| self.type_interner.get(id).unwrap().clone())
                    .collect();

                let resolved_return = self.resolve_type(return_type)?;
                let return_type = Box::new(self.type_interner.get(resolved_return).unwrap().clone());

                // Resolve type parameter constraints and defaults
                let mut resolved_type_params = Vec::with_capacity(type_params.len());
                for tp in type_params {
                    let resolved_constraint = tp.constraint.as_ref()
                        .map(|c| self.resolve_type(c))
                        .transpose()?
                        .map(|id| Box::new(self.type_interner.get(id).unwrap().clone()));

                    let resolved_default = tp.default.as_ref()
                        .map(|d| self.resolve_type(d))
                        .transpose()?
                        .map(|id| Box::new(self.type_interner.get(id).unwrap().clone()));

                    resolved_type_params.push(TypeParameter {
                        name: tp.name.clone(),
                        constraint: resolved_constraint,
                        default: resolved_default,
                    });
                }

                Ok(self.type_interner.intern(Type::Function {
                    params: param_types,
                    return_type,
                    type_params: resolved_type_params,
                }))
            }

            Type::Union(types) => {
                let resolved_types: Result<Vec<_>, _> = types.iter()
                    .map(|ty| self.resolve_type(ty))
                    .collect();
                let resolved_types = resolved_types?;
                Ok(self.type_interner.get_or_intern_union(resolved_types))
            }

            Type::Intersection(types) => {
                let resolved_types: Result<Vec<_>, _> = types.iter()
                    .map(|ty| self.resolve_type(ty))
                    .collect();
                let resolved_types = resolved_types?;

                // Convert to Type values for interning
                let intersection_types: Vec<_> = resolved_types.iter()
                    .map(|&id| self.type_interner.get(id).unwrap().clone())
                    .collect();

                Ok(self.type_interner.intern(Type::Intersection(intersection_types)))
            }

            Type::Generic { base, args } => {
                // Clone base before mutable borrow of self for resolve_type
                let base_id = *base;

                // Resolve arguments first
                let resolved_args: Result<Vec<_>, _> = args.iter()
                    .map(|arg| self.resolve_type(arg))
                    .collect();
                let resolved_args = resolved_args?;

                // Now resolve base type
                let base_ty = self.type_interner.get(base_id)
                    .ok_or_else(|| ResolutionError::TypeNotFound("Generic base type not found".to_string()))?
                    .clone();
                let resolved_base = self.resolve_type(&base_ty)?;

                let arg_types: Vec<_> = resolved_args.iter()
                    .map(|&id| self.type_interner.get(id).unwrap().clone())
                    .collect();

                Ok(self.type_interner.intern(Type::Generic {
                    base: resolved_base,
                    args: arg_types,
                }))
            }

            // Primitive types and type parameters don't need resolution
            Type::Primitive(_) | Type::TypeParameter(_) => {
                Ok(self.type_interner.intern(ty.clone()))
            }
        }
    }

    /// Get the current scope.
    pub fn current_scope(&self) -> ScopeId {
        self.current_scope
    }

    /// Set the current scope.
    pub fn set_current_scope(&mut self, scope: ScopeId) {
        self.current_scope = scope;
    }

    /// Get the span from a node or return a default span.
    #[inline]
    fn get_span(&self, node: &AstNode<'_>) -> crate::parser::ast::Span {
        #[cfg(feature = "spans")]
        {
            node.span().unwrap_or_else(|| crate::parser::ast::Span::new(0, 0))
        }
        #[cfg(not(feature = "spans"))]
        {
            crate::parser::ast::Span::new(0, 0)
        }
    }

    /// Convert a TypeAnnotation to a Type.
    fn type_annotation_to_type(&mut self, annotation: &TypeAnnotation) -> Type {
        match annotation {
            TypeAnnotation::TypeReference { name, type_params } => {
                if let Some(params) = type_params {
                    let resolved_params: Vec<_> = params.iter()
                        .map(|p| self.type_annotation_to_type(p))
                        .collect();
                    Type::Reference {
                        name: name.clone(),
                        type_args: resolved_params,
                    }
                } else {
                    Type::Reference {
                        name: name.clone(),
                        type_args: vec![],
                    }
                }
            }
            TypeAnnotation::ArrayType(elem) => {
                Type::Array(Box::new(self.type_annotation_to_type(elem)))
            }
            TypeAnnotation::UnionType(types) => {
                let union_types: Vec<_> = types.iter()
                    .map(|t| self.type_annotation_to_type(t))
                    .collect();
                Type::Union(union_types)
            }
            TypeAnnotation::FunctionType { params, return_type } => {
                let param_types: Vec<_> = params.iter()
                    .map(|p| self.type_annotation_to_type(&p.type_annotation.clone()
                        .unwrap_or(TypeAnnotation::Unknown)))
                    .collect();
                let return_ty = self.type_annotation_to_type(return_type);
                Type::Function {
                    params: param_types,
                    return_type: Box::new(return_ty),
                    type_params: vec![],
                }
            }
            TypeAnnotation::Unknown => Type::Primitive(PrimitiveType::Unknown),
        }
    }
}

impl<'a, 'ast> Visitor<'ast> for TypeResolver<'a> {
    /// Default visitor implementation.
    fn default_visit_node(&mut self, node: &'ast AstNode<'ast>) {
        // Visit all children recursively
        for child in node.children() {
            self.visit_node(child);
        }
    }

    /// Visit a variable statement and resolve types for variable declarations.
    fn visit_variable_statement(&mut self, node: &'ast AstNode<'ast>) {
        if let NodeKind::VariableStatement { declarations } = node.kind() {
            for decl in declarations {
                // Resolve the type for this variable
                let type_id = if let Some(type_annotation) = &decl.type_annotation {
                    // Convert TypeAnnotation to Type and resolve it
                    let type_value = self.type_annotation_to_type(type_annotation);
                    match self.resolve_type(&type_value) {
                        Ok(id) => Some(id),
                        Err(_) => None, // TODO: Collect errors instead of silently failing
                    }
                } else {
                    // For now, use Unknown type if no annotation
                    Some(self.type_interner.intern(Type::Primitive(PrimitiveType::Unknown)))
                };

                // Update the symbol's type_id
                if let Some(symbol_id) = self.symbol_table.lookup_lexical(
                    &decl.name,
                    self.current_scope,
                    self.scope_table,
                ) {
                    if let Some(ty) = type_id {
                        if let Some(symbol) = self.symbol_table.lookup_mut(symbol_id) {
                            symbol.set_type_id(ty);
                        }
                    }
                }
            }
        }

        // Visit children (initializers)
        self.default_visit_node(node);
    }

    /// Visit a function declaration and resolve types for parameters and return type.
    fn visit_function_declaration(&mut self, node: &'ast AstNode<'ast>) {
        if let NodeKind::FunctionDeclaration { name, params, return_type, body: _ } = node.kind() {
            // Find the function symbol
            if let Some(symbol_id) = self.symbol_table.lookup_lexical(
                name,
                self.current_scope,
                self.scope_table,
            ) {
                // Build the function type
                let param_types: Vec<_> = params.iter()
                    .map(|param| {
                        let ty = if let Some(type_annotation) = &param.type_annotation {
                            let type_value = self.type_annotation_to_type(type_annotation);
                            self.resolve_type(&type_value).unwrap_or_else(|_| {
                                self.type_interner.intern(Type::Primitive(PrimitiveType::Unknown))
                            })
                        } else {
                            self.type_interner.intern(Type::Primitive(PrimitiveType::Unknown))
                        };
                        self.type_interner.get(ty).unwrap().clone()
                    })
                    .collect();

                let return_ty = if let Some(rt) = return_type {
                    let type_value = self.type_annotation_to_type(rt);
                    self.resolve_type(&type_value).ok()
                        .map(|id| Box::new(self.type_interner.get(id).unwrap().clone()))
                } else {
                    None
                };

                // Create the function type
                let function_type = Type::Function {
                    params: param_types,
                    return_type: return_ty.unwrap_or_else(|| {
                        Box::new(Type::Primitive(PrimitiveType::Unknown))
                    }),
                    type_params: vec![],
                };

                // Intern the function type
                let type_id = self.type_interner.intern(function_type);

                // Update the symbol's type_id
                if let Some(symbol) = self.symbol_table.lookup_mut(symbol_id) {
                    symbol.set_type_id(type_id);
                }
            }
        }

        // Visit function body
        self.default_visit_node(node);
    }
}

/// Result of a type resolution attempt.
pub type ResolutionResult<T = TypeId> = Result<T, ResolutionError>;

/// Error that occurs when type resolution fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResolutionError {
    /// Named type was not found in scope
    #[error("type `{0}` not found in scope")]
    TypeNotFound(String),

    /// Reference refers to a non-type symbol
    #[error("`{0}` is not a type")]
    NotAType(String),

    /// Generic reference has wrong number of arguments
    #[error("type `{name}` expects {expected} type arguments, got {actual}")]
    GenericArityMismatch {
        name: String,
        expected: usize,
        actual: usize,
    },

    /// Recursive type reference without proper indirection
    #[error("recursive type reference to `{0}`")]
    RecursiveReference(String),
}
