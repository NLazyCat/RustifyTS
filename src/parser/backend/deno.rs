//! Deno backend implementation
//!
//! This module provides a ParserBackend implementation that uses Deno as a subprocess
//! to parse TypeScript source code using the TypeScript Compiler API.

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use super::r#trait::ParserBackend;
use crate::parser::error::ParseError;
use serde_json::Value;

/// Deno backend configuration
#[derive(Debug, Clone)]
pub struct DenoBackendConfig {
    /// Path to Deno binary (default: "deno")
    pub deno_path: String,
    /// Path to deno_parser.ts script
    pub script_path: String,
    /// Timeout in seconds for Deno subprocess (default: 30)
    pub timeout: u64,
    /// Additional Deno CLI arguments
    pub deno_args: Vec<String>,
}

impl Default for DenoBackendConfig {
    fn default() -> Self {
        Self {
            deno_path: "deno".to_string(),
            script_path: "deno-bridge/deno_parser.ts".to_string(),
            timeout: 30,
            deno_args: vec![
                "run".to_string(),
                "--allow-read".to_string(),
                "--allow-env".to_string(),
            ],
        }
    }
}

/// Deno parser backend
///
/// Uses Deno as a subprocess to parse TypeScript source code using the
/// TypeScript Compiler API. Communication happens via stdin/stdout using
/// MessagePack serialization.
#[derive(Debug, Clone)]
pub struct DenoBackend {
    config: DenoBackendConfig,
}

impl DenoBackend {
    /// Create a new DenoBackend with default configuration
    pub fn new() -> Result<Self, ParseError> {
        Self::with_config(DenoBackendConfig::default())
    }

    /// Create a new DenoBackend with custom configuration
    pub fn with_config(config: DenoBackendConfig) -> Result<Self, ParseError> {
        // Verify script exists
        let script_path = Path::new(&config.script_path);
        if !script_path.exists() {
            return Err(ParseError::FileNotFound(script_path.to_path_buf()));
        }

        Ok(Self { config })
    }

    /// Check if Deno is available in the system
    pub async fn check_deno_available(&self) -> Result<(), ParseError> {
        let output = Command::new(&self.config.deno_path)
            .arg("--version")
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => Ok(()),
            Ok(_) => Err(ParseError::DenoNotFound),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(ParseError::DenoNotFound)
            }
            Err(e) => Err(ParseError::DenoStartFailed(e.to_string())),
        }
    }
}

/// Request sent to Deno parser
#[derive(Serialize, Deserialize, Debug)]
struct ParseRequest {
    source: String,
}

/// Response received from Deno parser
#[derive(Serialize, Deserialize, Debug)]
struct ParseResponse {
    success: bool,
    #[serde(default)]
    ast: Option<serde_json::Value>,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default)]
    error: Option<String>,
}

#[async_trait]
impl ParserBackend for DenoBackend {
    async fn parse_raw(&self, source: &str) -> Result<Value, ParseError> {
        // Check if Deno is available
        self.check_deno_available().await?;

        // Prepare request
        let request = ParseRequest {
            source: source.to_string(),
        };

        // Serialize request to MessagePack
        let input = to_vec(&request)
            .map_err(ParseError::serialization_error)?;

        // Build Deno command
        let mut command = Command::new(&self.config.deno_path);
        command
            .args(&self.config.deno_args)
            .arg(&self.config.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn subprocess
        let mut child = command.spawn()
            .map_err(|e| ParseError::DenoStartFailed(e.to_string()))?;

        // Get stdin handle
        let mut stdin = child.stdin.take()
            .ok_or_else(|| ParseError::DenoStartFailed("Failed to open stdin".to_string()))?;

        // Write request to stdin
        tokio::io::AsyncWriteExt::write_all(&mut stdin, &input).await
            .map_err(|e| ParseError::DenoExecutionError(format!("Failed to write to stdin: {}", e)))?;

        // Drop stdin to signal EOF
        drop(stdin);

        // Wait for output with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout),
            child.wait_with_output()
        ).await
            .map_err(|_| ParseError::Timeout(self.config.timeout))?
            .map_err(|e| ParseError::DenoExecutionError(e.to_string()))?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return if let Some(exit_code) = output.status.code() {
                Err(ParseError::DenoNonZeroExit {
                    code: exit_code,
                    message: stderr.to_string(),
                })
            } else {
                Err(ParseError::DenoExecutionError(format!("Process terminated by signal: {}", stderr)))
            };
        }

        // Deserialize response
        let response: ParseResponse = from_slice(&output.stdout)
            .map_err(ParseError::deserialization_error)?;

        if !response.success {
            return Err(ParseError::DenoExecutionError(
                response.error.unwrap_or_else(|| "Unknown error from Deno parser".to_string())
            ));
        }

        if !response.errors.is_empty() {
            return Err(ParseError::SyntaxError {
                line: 1, // TODO: Extract line/column from error
                column: 1,
                message: response.errors.join("\n"),
            });
        }

        let ast_value = response.ast
            .ok_or_else(|| ParseError::InvalidAst("Missing AST in response".to_string()))?;

        Ok(ast_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deno_backend_creation() {
        let backend = DenoBackend::new();
        // This might fail if Deno is not installed, so we just check it doesn't panic
        assert!(backend.is_ok() || matches!(backend.err(), Some(ParseError::FileNotFound(_))));
    }

    #[tokio::test]
    async fn test_parse_simple_source() {
        let backend = match DenoBackend::new() {
            Ok(b) => b,
            Err(_) => {
                // Skip test if Deno or script not available
                return;
            }
        };

        // Check if Deno is available
        if backend.check_deno_available().await.is_err() {
            return;
        }

        let source = "const x: number = 42;";
        let result = backend.parse_raw(source).await;

        // For now, we just check it doesn't panic
        assert!(result.is_ok());
        if let Ok(ast) = result {
            assert!(ast.is_object());
        }
    }
}
