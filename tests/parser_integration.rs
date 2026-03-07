//! Parser integration tests
//!
//! Tests for the Deno backend parser integration.

use std::path::Path;
use RustifyTS::parser::{DenoBackend, ParserBackend};

/// Test that Deno backend can parse a simple TypeScript file
#[tokio::test]
async fn test_parse_simple_file() {
    let backend = match DenoBackend::new() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    // Check if Deno is available
    if backend.check_deno_available().await.is_err() {
        eprintln!("Skipping test: Deno not available");
        return;
    }

    let path = Path::new("tests/fixtures/parser/simple.ts");
    let result = backend.parse_file_raw(path).await;

    match result {
        Ok(ast) => {
            assert!(ast.is_object());
            println!("Successfully parsed simple.ts");
            println!("AST type: {}", ast["kind"].as_str().unwrap_or("unknown"));
        }
        Err(e) => {
            panic!("Failed to parse simple.ts: {}", e);
        }
    }
}

/// Test that Deno backend can parse a more complex TypeScript file
#[tokio::test]
async fn test_parse_complex_file() {
    let backend = match DenoBackend::new() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    // Check if Deno is available
    if backend.check_deno_available().await.is_err() {
        eprintln!("Skipping test: Deno not available");
        return;
    }

    let path = Path::new("tests/fixtures/parser/complex.ts");
    let result = backend.parse_file_raw(path).await;

    match result {
        Ok(ast) => {
            assert!(ast.is_object());
            println!("Successfully parsed complex.ts");
            println!("AST type: {}", ast["kind"].as_str().unwrap_or("unknown"));
        }
        Err(e) => {
            panic!("Failed to parse complex.ts: {}", e);
        }
    }
}

/// Test that Deno backend can parse TypeScript source from a string
#[tokio::test]
async fn test_parse_source_string() {
    let backend = match DenoBackend::new() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    // Check if Deno is available
    if backend.check_deno_available().await.is_err() {
        eprintln!("Skipping test: Deno not available");
        return;
    }

    let source = r#"
        function greet(name: string): string {
            return `Hello, ${name}!`;
        }

        const message = greet("World");
        console.log(message);
    "#;

    let result = backend.parse_raw(source).await;

    match result {
        Ok(ast) => {
            assert!(ast.is_object());
            println!("Successfully parsed source string");
            println!("AST type: {}", ast["kind"].as_str().unwrap_or("unknown"));
        }
        Err(e) => {
            panic!("Failed to parse source string: {}", e);
        }
    }
}

/// Test error handling for invalid TypeScript
#[tokio::test]
async fn test_parse_invalid_syntax() {
    let backend = match DenoBackend::new() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    // Check if Deno is available
    if backend.check_deno_available().await.is_err() {
        eprintln!("Skipping test: Deno not available");
        return;
    }

    // Invalid TypeScript syntax
    let source = r#"
        function broken( {
            return "this is invalid";
        }
    "#;

    let result = backend.parse_raw(source).await;

    // The parser should return an error for invalid syntax
    assert!(result.is_err());
    if let Err(e) = result {
        println!("Got expected error: {}", e);
    }
}

