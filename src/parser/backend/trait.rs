//! Parser backend trait definition
//!
//! This module defines the ParserBackend trait that all parser backends must implement.
//! It provides a common interface for parsing TypeScript source code into AST.

use std::path::Path;
use crate::parser::error::ParseError;
use serde_json::Value;

/// Parser backend interface
///
/// A ParserBackend is responsible for parsing TypeScript source code into
/// an AST representation. Different backends can implement this trait to
/// provide alternative parsing implementations (e.g., Deno, swc, etc.).
#[async_trait::async_trait]
pub trait ParserBackend {
    /// Parse TypeScript source code from a string
    ///
    /// # Arguments
    ///
    /// * `source` - The TypeScript source code to parse
    ///
    /// # Returns
    ///
    /// A Result containing either the parsed AST as JSON or a ParseError
    async fn parse_raw(&self, source: &str) -> Result<Value, ParseError>;

    /// Parse TypeScript source code from a file
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the TypeScript file to parse
    ///
    /// # Returns
    ///
    /// A Result containing either the parsed AST as JSON or a ParseError
    async fn parse_file_raw(&self, path: &Path) -> Result<Value, ParseError> {
        let source = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| if e.kind() == std::io::ErrorKind::NotFound {
                ParseError::FileNotFound(path.to_path_buf())
            } else {
                ParseError::Io(e)
            })?;
        self.parse_raw(&source).await
    }
}


