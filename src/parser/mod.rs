//! Parser module - TypeScript source code parsing
//!
//! This module provides functionality to parse TypeScript source code into a unified
//! AST representation using Deno as the backend parser.

pub mod error;
pub mod ast;
pub mod backend;

// Re-export key types
pub use error::ParseError;
pub use backend::{ParserBackend, DenoBackend, DenoBackendConfig};
pub use ast::AstArena;

use std::path::Path;
use tokio::runtime::Runtime;

/// Parse TypeScript source code from a string
///
/// This function uses the default Deno backend to parse TypeScript source code
/// into an AST representation. It creates a new AST arena for the parsed nodes.
///
/// # Arguments
///
/// * `source` - The TypeScript source code to parse
///
/// # Returns
///
/// A Result containing either the parsed AST arena or a ParseError
///
/// # Example
///
/// ```no_run
/// # use transmute::parse_source;
/// let ts_code = r#"
///     function hello(name: string): string {
///         return `Hello, ${name}!`;
///     }
/// "#;
///
/// match parse_source(ts_code) {
///     Ok(arena) => println!("Parsed successfully, root node: {:?}", arena.root()),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// ```
pub fn parse_source(source: &str) -> Result<AstArena<'_>, ParseError> {
    // Create a tokio runtime for async operations
    let rt = Runtime::new()
        .map_err(|e| ParseError::RuntimeError(format!("Failed to create tokio runtime: {}", e)))?;

    // Create Deno backend with default configuration
    let backend = DenoBackend::new()?;

    // Parse the source using the backend
    let ast_json = rt.block_on(backend.parse_raw(source))?;

    // Convert JSON AST to typed AST in arena
    let arena = AstArena::from_json(&ast_json)?;

    Ok(arena)
}

/// Parse TypeScript source code from a file
///
/// This function reads a TypeScript file and parses it into an AST representation
/// using the default Deno backend. It creates a new AST arena for the parsed nodes.
///
/// # Arguments
///
/// * `path` - The path to the TypeScript file to parse
///
/// # Returns
///
/// A Result containing either the parsed AST arena or a ParseError
///
/// # Example
///
/// ```no_run
/// # use transmute::parse_file;
/// # use std::path::Path;
/// match parse_file(Path::new("example.ts")) {
///     Ok(arena) => println!("Parsed successfully, root node: {:?}", arena.root()),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// ```
pub fn parse_file(path: &Path) -> Result<AstArena, ParseError> {
    // Create a tokio runtime for async operations
    let rt = Runtime::new()
        .map_err(|e| ParseError::RuntimeError(format!("Failed to create tokio runtime: {}", e)))?;

    // Create Deno backend with default configuration
    let backend = DenoBackend::new()?;

    // Parse the file using the backend
    let ast_json = rt.block_on(backend.parse_file_raw(path))?;

    // Convert JSON AST to typed AST in arena
    let arena = AstArena::from_json(&ast_json)?;

    Ok(arena)
}

/// Parse TypeScript source code asynchronously from a string
///
/// This async version of parse_source uses the default Deno backend to parse
/// TypeScript source code into an AST representation without blocking the current thread.
///
/// # Arguments
///
/// * `source` - The TypeScript source code to parse
///
/// # Returns
///
/// A Future that resolves to a Result containing either the parsed AST arena or a ParseError
///
/// # Example
///
/// ```no_run
/// # use transmute::parse_source_async;
/// # #[tokio::main]
/// # async fn main() {
/// let ts_code = r#"
///     function hello(name: string): string {
///         return `Hello, ${name}!`;
///     }
/// "#;
///
/// match parse_source_async(ts_code).await {
///     Ok(arena) => println!("Parsed successfully"),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// # }
/// ```
pub async fn parse_source_async(source: &str) -> Result<AstArena, ParseError> {
    // Create Deno backend with default configuration
    let backend = DenoBackend::new()?;

    // Parse the source using the backend
    let ast_json = backend.parse_raw(source).await?;

    // Convert JSON AST to typed AST in arena
    let arena = AstArena::from_json(&ast_json)?;

    Ok(arena)
}

/// Parse TypeScript source code asynchronously from a file
///
/// This async version of parse_file reads a TypeScript file and parses it into
/// an AST representation using the default Deno backend without blocking the current thread.
///
/// # Arguments
///
/// * `path` - The path to the TypeScript file to parse
///
/// # Returns
///
/// A Future that resolves to a Result containing either the parsed AST arena or a ParseError
///
/// # Example
///
/// ```no_run
/// # use transmute::parse_file_async;
/// # use std::path::Path;
/// # #[tokio::main]
/// # async fn main() {
/// match parse_file_async(Path::new("example.ts")).await {
///     Ok(arena) => println!("Parsed successfully"),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// # }
/// ```
pub async fn parse_file_async(path: &Path) -> Result<AstArena, ParseError> {
    // Create Deno backend with default configuration
    let backend = DenoBackend::new()?;

    // Parse the file using the backend
    let ast_json = backend.parse_file_raw(path).await?;

    // Convert JSON AST to typed AST in arena
    let arena = AstArena::from_json(&ast_json)?;

    Ok(arena)
}
