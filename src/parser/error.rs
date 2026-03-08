//! Parser error types
//!
//! This module defines all error types that can occur during TypeScript parsing,
//! using `thiserror` for structured error definitions.

use std::path::PathBuf;
use thiserror::Error;

/// Parser error type
///
/// Errors that can occur during parsing TypeScript source code.
/// This uses `thiserror` for automatic implementation of `std::error::Error`,
/// `Display`, and `From` conversions where applicable.
#[derive(Debug, Error)]
pub enum ParseError {
    /// File I/O error when reading source files
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// File not found error
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Deno subprocess failed to start
    #[error("Failed to start Deno subprocess: {0}")]
    DenoStartFailed(String),

    /// Deno subprocess execution error
    #[error("Deno subprocess error: {0}")]
    DenoExecutionError(String),

    /// Deno returned non-zero exit code
    #[error("Deno exited with code {code}: {message}")]
    DenoNonZeroExit { code: i32, message: String },

    /// Deno binary not found
    #[error("Deno binary not found. Please install Deno: https://deno.land/install")]
    DenoNotFound,

    /// MessagePack serialization error
    #[error("MessagePack serialization error: {0}")]
    SerializationError(String),

    /// MessagePack deserialization error
    #[error("MessagePack deserialization error: {0}")]
    DeserializationError(String),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Syntax error in TypeScript source
    #[error("Syntax error at {line}:{column}: {message}")]
    SyntaxError { line: usize, column: usize, message: String },

    /// Invalid AST structure received from Deno
    #[error("Invalid AST structure: {0}")]
    InvalidAst(String),

    /// Timeout waiting for Deno process
    #[error("Deno process timeout after {0} seconds")]
    Timeout(u64),

    /// Feature not yet implemented
    #[error("Feature not implemented: {feature}")]
    Unimplemented { feature: String },

    /// Generic parse error with context
    #[error("Parse error: {0}")]
    Generic(String),

    /// Runtime error (e.g., failed to create tokio runtime)
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

impl ParseError {
    /// Create a serialization error from any error that implements Display
    pub fn serialization_error<E: std::fmt::Display>(e: E) -> Self {
        ParseError::SerializationError(e.to_string())
    }

    /// Create a deserialization error from any error that implements Display
    pub fn deserialization_error<E: std::fmt::Display>(e: E) -> Self {
        ParseError::DeserializationError(e.to_string())
    }

    /// Create an invalid AST error
    pub fn invalid_ast<S: Into<String>>(msg: S) -> Self {
        ParseError::InvalidAst(msg.into())
    }

    /// Create a generic parse error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        ParseError::Generic(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ParseError::FileNotFound(PathBuf::from("test.ts"));
        assert_eq!(err.to_string(), "File not found: test.ts");

        let err = ParseError::DenoNotFound;
        assert!(err.to_string().contains("Deno binary not found"));

        let err = ParseError::SyntaxError {
            line: 10,
            column: 5,
            message: "Unexpected token".to_string(),
        };
        assert_eq!(err.to_string(), "Syntax error at 10:5: Unexpected token");
    }

    #[test]
    fn test_error_constructors() {
        let err = ParseError::invalid_ast("missing field");
        assert!(matches!(err, ParseError::InvalidAst(_)));

        let err = ParseError::serialization_error("test error");
        assert!(matches!(err, ParseError::SerializationError(_)));

        let err = ParseError::deserialization_error("test error");
        assert!(matches!(err, ParseError::DeserializationError(_)));

        let err = ParseError::generic("something went wrong");
        assert!(matches!(err, ParseError::Generic(_)));
    }

    #[test]
    fn test_io_error_conversion() {
        // std::io::Error is automatically converted
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let parse_err = ParseError::from(io_err);
        assert!(matches!(parse_err, ParseError::Io(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        // serde_json::Error is automatically converted
        // Use serde_json to parse invalid JSON to create an error
        let json_err: Result<serde_json::Value, serde_json::Error> = serde_json::from_str("invalid json");
        if let Err(json_err) = json_err {
            let parse_err = ParseError::from(json_err);
            assert!(matches!(parse_err, ParseError::JsonError(_)));
        }
    }
}
