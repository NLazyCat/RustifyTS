//! Main semantic analyzer module.
//!
//! Orchestrates the entire semantic analysis process, coordinating
//! all sub-components (scope, symbol, types, flow, IR) to produce
//! a fully analyzed program representation.

use crate::parser::ast::{AstNode, Span, visitor::Visitor, NodeKind};
use crate::semantic::flow::builder::CFGBuilder;
use crate::semantic::ir::module::SemanticModule;
use crate::semantic::ir::Function;
use crate::semantic::scope::analyzer::ScopeAnalyzer;
use crate::semantic::types::resolver::TypeResolver;
use crate::semantic::types::{Type, TypeId, TypeInterner, PrimitiveType};
use crate::semantic::symbol::SymbolKind;
use bumpalo::Bump;
use thiserror::Error;

/// Error that can occur during semantic analysis.
#[derive(Debug, Error)]
pub enum SemanticError {
    /// Type resolution error
    #[error("type error: {0}")]
    TypeError(#[from] crate::semantic::types::resolver::ResolutionError),

    /// Multiple type resolution errors
    #[error("{0} type resolution error(s) occurred")]
    MultipleTypeErrors(usize),
}

/// Main semantic analyzer that coordinates all analysis passes.
///
/// The SemanticAnalyzer runs a series of analysis passes on the AST:
/// 1. Scope analysis - builds the scope hierarchy and symbol table
/// 2. Type resolution - resolves all type references
/// 3. CFG construction - builds control flow graphs for functions
///
/// The result is a complete SemanticModule containing all semantic information.
pub struct SemanticAnalyzer<'a> {
    /// Bump allocator for AST nodes
    _arena: &'a Bump,
    /// Type interner for type management
    type_interner: TypeInterner,
    /// Counter for generating anonymous function names
    anon_function_counter: u32,
}

/// Information about a function found during AST traversal.
struct FunctionInfo<'a> {
    /// Function name (or generated name for anonymous functions)
    name: String,
    /// Function symbol ID (if found in symbol table)
    symbol_id: Option<crate::semantic::symbol::SymbolId>,
    /// Function kind (declaration, expression, arrow)
    kind: FunctionKind,
    /// Reference to the AST node containing the function
    node: &'a AstNode<'a>,
    /// Function parameters
    params: Vec<(String, TypeId)>,
    /// Return type
    return_type: TypeId,
}

/// The kind of function definition.
enum FunctionKind {
    /// Function declaration: function name() { ... }
    Declaration,
    /// Function expression: const x = function() { ... }
    Expression,
    /// Arrow function: const x = () => { ... }
    Arrow,
}

/// Visitor that collects function definitions from the AST.
struct FunctionCollector<'a> {
    /// Collected functions
    functions: Vec<FunctionInfo<'a>>,
    /// Counter for generating anonymous function names
    anon_counter: u32,
}

impl<'a> FunctionCollector<'a> {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            anon_counter: 0,
        }
    }

    fn get_unknown_type_id(&self) -> TypeId {
        // Create a new TypeInterner just for interning the unknown type
        // This is a workaround - in production, we'd handle this better
        let mut temp_interner = TypeInterner::new();
        temp_interner.intern(Type::Primitive(PrimitiveType::Unknown))
    }

    fn extract_params(&mut self, node: &'a AstNode<'a>) -> Vec<(String, TypeId)> {
        let mut params = Vec::new();

        if let NodeKind::FunctionDeclaration { params: decl_params, .. } = node.kind() {
            for param in decl_params {
                let type_id = param.type_annotation.as_ref()
                    .map(|_| self.get_unknown_type_id())
                    .unwrap_or_else(|| self.get_unknown_type_id());
                params.push((param.name.clone(), type_id));
            }
        } else if let NodeKind::FunctionExpression { params: decl_params, .. } = node.kind() {
            for param in decl_params {
                let type_id = param.type_annotation.as_ref()
                    .map(|_| self.get_unknown_type_id())
                    .unwrap_or_else(|| self.get_unknown_type_id());
                params.push((param.name.clone(), type_id));
            }
        } else if let NodeKind::ArrowFunction { params: decl_params, .. } = node.kind() {
            for param in decl_params {
                let type_id = param.type_annotation.as_ref()
                    .map(|_| self.get_unknown_type_id())
                    .unwrap_or_else(|| self.get_unknown_type_id());
                params.push((param.name.clone(), type_id));
            }
        }

        params
    }
}

impl<'a> Visitor<'a> for FunctionCollector<'a> {
    fn visit_function_declaration(&mut self, node: &'a AstNode<'a>) {
        if let NodeKind::FunctionDeclaration { name, .. } = node.kind() {
            let params = self.extract_params(node);
            let return_type = self.get_unknown_type_id();

            self.functions.push(FunctionInfo {
                name: name.clone(),
                symbol_id: None,
                kind: FunctionKind::Declaration,
                node,
                params,
                return_type,
            });
        }
        self.default_visit_node(node);
    }

    fn visit_function_expression(&mut self, node: &'a AstNode<'a>) {
        let name = if let NodeKind::FunctionExpression { name, .. } = node.kind() {
            name.clone().unwrap_or_else(|| {
                let name = format!("anon_{}", self.anon_counter);
                self.anon_counter += 1;
                name
            })
        } else {
            let name = format!("anon_{}", self.anon_counter);
            self.anon_counter += 1;
            name
        };

        let params = self.extract_params(node);
        let return_type = self.get_unknown_type_id();

        self.functions.push(FunctionInfo {
            name,
            symbol_id: None,
            kind: FunctionKind::Expression,
            node,
            params,
            return_type,
        });

        self.default_visit_node(node);
    }

    fn visit_arrow_function(&mut self, node: &'a AstNode<'a>) {
        let name = format!("anon_{}", self.anon_counter);
        self.anon_counter += 1;

        let params = self.extract_params(node);
        let return_type = self.get_unknown_type_id();

        self.functions.push(FunctionInfo {
            name,
            symbol_id: None,
            kind: FunctionKind::Arrow,
            node,
            params,
            return_type,
        });

        self.default_visit_node(node);
    }
}

impl<'a> SemanticAnalyzer<'a> {
    /// Create a new SemanticAnalyzer with the given arena.
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            _arena: arena,
            type_interner: TypeInterner::new(),
            anon_function_counter: 0,
        }
    }

    /// Analyze the given AST and produce a complete semantic module.
    ///
    /// This runs all analysis passes in sequence:
    /// 1. Scope analysis - builds scope tree and symbol table
    /// 2. Type resolution - resolves all type references to concrete types
    /// 3. CFG construction - builds control flow graphs for functions
    pub fn analyze(&mut self, ast: &'a AstNode<'a>) -> Result<SemanticModule, SemanticError> {
        // Get root span for module
        let root_span = Self::get_span(ast);

        // Create a module to hold all analysis results
        let mut module = SemanticModule::new("main".to_string());

        // Pass 1: Scope analysis
        // Build the scope hierarchy and populate the symbol table
        let mut scope_analyzer = ScopeAnalyzer::new(
            self._arena,
            &mut self.type_interner,
            root_span,
        );
        scope_analyzer.visit_node(ast);

        // Transfer scope table and symbol table to module
        module.scopes = scope_analyzer.scope_table;
        module.symbols = scope_analyzer.symbol_table;

        // Transfer the type interner as well (it contains the interned types)
        // We need to replace the analyzer's type_interner with a new one
        // so that the module gets the types created during scope analysis
        module.types = std::mem::replace(&mut self.type_interner, TypeInterner::new());

        // Pass 2: Type resolution
        // Resolve all type references in the AST
        let mut type_resolver = TypeResolver::new(
            &mut module.symbols,
            &module.scopes,
            &mut self.type_interner,
            module.scopes.root(),
        );
        type_resolver.visit_node(ast);

        // Check for type resolution errors
        if type_resolver.has_errors() {
            let errors = type_resolver.take_errors();
            let error_count = errors.len();
            // Store errors in the module for later reporting
            // For now, just return an error indicating the count
            return Err(SemanticError::MultipleTypeErrors(error_count));
        }

        // Pass 3: CFG construction
        // Build control flow graphs for all functions
        // For now, we'll create a simple CFG for each function in the symbol table
        self.build_cfgs_for_functions(&mut module, ast)?;

        Ok(module)
    }

    /// Build CFGs for all functions in the module.
    fn build_cfgs_for_functions(
        &mut self,
        module: &mut SemanticModule,
        ast: &'a AstNode<'a>,
    ) -> Result<(), SemanticError> {
        // Task 1: Find all function definitions in the AST
        let mut collector = FunctionCollector::new();
        collector.visit_node(ast);

        // Clone the functions to release the borrow on module
        let functions_to_build: Vec<_> = collector.functions.into_iter().collect();

        // Task 2: Create CFG for each function
        for func_info in functions_to_build {
            self.build_cfg_for_function(module, func_info)?;
        }

        Ok(())
    }

    /// Build a CFG for a single function.
    fn build_cfg_for_function(
        &mut self,
        module: &mut SemanticModule,
        func_info: FunctionInfo<'a>,
    ) -> Result<(), SemanticError> {
        use crate::semantic::ir::Instruction;

        // Look up the function symbol in the symbol table
        let symbol_id = if let Some(sym_id) = func_info.symbol_id {
            sym_id
        } else {
            // Try to find the symbol by name in the module's symbol table
            let root_scope = module.scopes.root();
            if let Some(sym) = module.symbols.lookup_lexical(&func_info.name, root_scope, &module.scopes) {
                sym
            } else {
                // Create a new symbol ID for this function
                // In a real implementation, this would be handled during scope analysis
                let span = Self::get_span(func_info.node);
                let new_id = module.symbols.insert(
                    func_info.name.clone(),
                    SymbolKind::Function,
                    span,
                    root_scope,
                    Some(func_info.return_type),
                );
                new_id
            }
        };

        // Task 2: Create Function object
        let mut function = Function::new(
            symbol_id,
            func_info.name.clone(),
            func_info.params.clone(),
            func_info.return_type,
        );

        // Task 3: Wire CFGBuilder visitor to function body
        {
            let mut cfg_builder = CFGBuilder::new(&mut function);

            // Find and visit the function body
            let function_body = self.extract_function_body(func_info.node);
            if let Some(body_node) = function_body {
                cfg_builder.build(body_node);
            }
        }

        // Task 4: Add function parameters to CFG entry block
        self.add_parameters_to_entry_block(&mut function, &func_info.params);

        // Add the function to the module
        module.add_function(function);

        Ok(())
    }

    /// Extract the function body node from a function definition AST node.
    fn extract_function_body(&self, node: &'a AstNode<'a>) -> Option<&'a AstNode<'a>> {
        match node.kind() {
            NodeKind::FunctionDeclaration { body, .. } => {
                // The body is a NodeId, so we need to find the corresponding child node
                node.children().get(body.index() as usize).copied()
            }
            NodeKind::FunctionExpression { body, .. } => {
                node.children().get(body.index() as usize).copied()
            }
            NodeKind::ArrowFunction { body, .. } => {
                node.children().get(body.index() as usize).copied()
            }
            _ => None,
        }
    }

    /// Add function parameters to the entry block of the CFG.
    fn add_parameters_to_entry_block(
        &self,
        function: &mut Function,
        params: &[(String, TypeId)],
    ) {
        use crate::semantic::ir::Instruction;

        let entry_block = function.entry_block();

        for (_param_name, param_type) in params {
            // Create an alloca instruction for the parameter
            let alloca_inst = Instruction::Alloca {
                ty: *param_type,
            };

            // Add the instruction to the entry block
            function.add_instruction(entry_block, alloca_inst);

            // TODO: Store the parameter value into the allocated slot
            // This would require knowing the parameter value ID, which
            // will be implemented in a future iteration
        }
    }

    /// Get the span from a node or return a default span.
    #[inline]
    fn get_span(node: &'a AstNode<'a>) -> Span {
        #[cfg(feature = "spans")]
        {
            node.span().unwrap_or_else(|| Span::new(0, 0))
        }
        #[cfg(not(feature = "spans"))]
        {
            Span::new(0, 0)
        }
    }
}

#[cfg(test)]
mod tests;