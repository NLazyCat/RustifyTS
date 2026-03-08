# Phase 02: 解析层 - Research

**Researched:** 2026-03-01
**Domain:** TypeScript Parser with Deno Backend
**Confidence:** HIGH

## Summary

Phase 02 requires implementing a TypeScript parser layer that converts TypeScript source code into a unified AST-IR representation. The parser uses Deno as a subprocess backend (communicating via MessagePack), implements an arena allocator for efficient AST node storage, provides a visitor pattern for AST traversal, and uses a three-layer error strategy (thiserror + anyhow + miette).

The key technical decisions are already locked in CONTEXT.md: use typed structs per node with arena allocation, MessagePack for Deno subprocess communication, zero-based span tracking internally, and structured error handling with fancy diagnostics.

**Primary recommendation:** Use bumpalo for arena allocation, rmp-serde for MessagePack serialization, tokio::process for Deno subprocess management, and implement a typed AST structure with visitor pattern for efficient parsing and traversal.

## User Constraints (from CONTEXT.md)

### Locked Decisions

### AST node design
- **结构:** Typed structs per node - 每种节点类型有独立的结构体
- **Span 存储:** Optional with cfg - 生产环境可选
- **类型粒度:** Categorized enums - 按类别分组（Statement, Expression, Declaration 等）
- **子节点存储:** Arena allocator - 独立的 arena + 节点索引系统
- **源文本:** Reference source file - 只保存跨度信息，按需从源文件重构
- **TS 特性支持:** Minimal core types - 仅核心类型，按需扩展
- **注释处理:** Track location only - 只跟踪位置，不解析内容
- **可变性:** Builder pattern - 构建后只读
- **遍历机制:** Visitor pattern - 实现访问者模式
- **相等性:** Structural equality - 基于结构的 PartialEq

### Deno backend integration
- **通信方式:** Subprocess/CLI - 使用 Deno CLI 作为子进程，通过 stdio/JSON 通信
- **序列化格式:** MessagePack - 使用 MessagePack 格式
- **Deno 可用性:** Auto-download - 首次使用时自动下载
- **进程生命周期:** Configurable - 可配置的持久化 + 空闲超时

### Error handling strategy
- **错误处理方式:** thiserror（定义结构化错误类型）+ anyhow（上层上下文包装）+ miette（友好的终端错误展示）
- **解析错误处理:** Partial AST recovery - 尝试恢复，返回带有嵌入错误的有效语法树
- **错误详情级别:** Diagnostic format - 跨度 + 代码片段 + 建议修复
- **Deno 错误映射:** Mapped with debug info - 映射到内部错误类型，调试模式包含 Deno 详情

### Span and location tracking
- **跨度表示:** Complete location - 同时存储字节偏移和行列号
- **索引约定:** Zero-based internally - 内部零索引，显示时加 1
- **Span 结构设计:** Explicit struct - 命名字段，Copy + Clone traits
- **行列查找:** Separate line map - 解析时构建行列映射，存储在独立结构中
- **诊断格式:** Flexible - 灵活支持零基存储和显示格式化

### Claude's Discretion
- Arena allocator 的具体实现选择
- Visitor pattern 的具体 trait 设计
- MessagePack 的具体配置选项
- 线程安全策略（如果需要并发解析）

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1.0.228 | Serialization framework | De facto standard for Rust serialization |
| `serde_json` | 1.0.149 | JSON handling | Standard JSON library for Rust |
| `rmp-serde` | latest | MessagePack serialization | Official Serde integration for MessagePack |
| `tokio` | 1.49.0 | Async runtime for subprocess | Standard async runtime in Rust ecosystem |
| `thiserror` | 2.0.18 | Structured error definitions | Derive-based error implementation, idiomatic Rust |
| `anyhow` | 1.0.102 | Application-level error handling | Flexible error type with context attachment |
| `miette` | latest | Fancy diagnostic reporting | Rich error reporting with code snippets |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `bumpalo` | latest | Arena allocator for AST nodes | High-performance allocation for phase-oriented data |
| `clap` | 4.5.60 | CLI argument parsing | Standard for command-line tools |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| bumpalo | Custom Vec-based indices | Bumpalo provides safety guarantees and lifetime tracking |
| rmp-serde | Raw JSON | MessagePack is faster and more compact for subprocess IPC |
| tokio::process | std::process::Command blocking | Tokio enables non-blocking subprocess for better performance |
| miette | colored/fancy-println | Miette provides structured diagnostics with source code snippets |

**Installation:**
```toml
[dependencies]
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
rmp-serde = "1.3"
tokio = { version = "1.49.0", features = ["rt-multi-thread", "process", "io-util"] }
thiserror = "2.0.18"
anyhow = "1.0.102"
miette = { version = "7.2", features = ["fancy"] }
bumpalo = "3.16"
clap = { version = "4.5.60", features = ["derive"] }
```

## Architecture Patterns

### Recommended Project Structure

```
src/
├── parser/
│   ├── mod.rs              # Module interface
│   ├── backend/
│   │   ├── mod.rs          # Backend module
│   │   ├── trait.rs        # ParserBackend trait
│   │   └── deno.rs         # Deno subprocess implementation
│   ├── ast/
│   │   ├── mod.rs          # AST module
│   │   ├── types.rs        # AST type definitions
│   │   ├── node.rs         # AstNode and builder
│   │   ├── visitor.rs      # Visitor trait and implementations
│   │   └── span.rs         # Span and location tracking
│   └── error.rs            # Error type definitions
├── lib.rs                  # Library entry point
└── main.rs                 # CLI entry point
```

### Pattern 1: Arena Allocator for AST Nodes

**What:** Use bumpalo arena to allocate all AST nodes in a contiguous memory region, using references tied to the arena lifetime.

**When to use:** When parsing large TypeScript files where all nodes have the same lifetime and can be deallocated together.

**Example:**
```rust
// Source: https://github.com/fitzgen/bumpalo
use bumpalo::Bump;

struct Node<'a> {
    kind: NodeKind,
    children: Vec<&'a Node<'a>>,
    span: Span,
}

fn parse_ast(source: &str, arena: &Bump) -> &Node {
    // Allocate all nodes in the arena
    let node = arena.alloc(Node {
        kind: NodeKind::Function,
        children: Vec::new(),
        span: Span::new(0, 10),
    });
    node
}
```

### Pattern 2: ParserBackend Trait for Extensibility

**What:** Define a trait for parser backends, allowing future support for multiple TypeScript parsers (Deno, swc, etc.).

**When to use:** When you need abstraction over different parser implementations.

**Example:**
```rust
pub trait ParserBackend {
    fn parse(&self, source: &str) -> Result<ParsedAst>;
    fn parse_file(&self, path: &Path) -> Result<ParsedAst>;
}

pub struct DenoBackend {
    // Deno-specific configuration
}

impl ParserBackend for DenoBackend {
    fn parse(&self, source: &str) -> Result<ParsedAst> {
        // Spawn Deno subprocess, send source, receive AST
    }
}
```

### Pattern 3: Visitor Pattern for AST Traversal

**What:** Define a visitor trait that can walk the AST and perform operations on different node types.

**When to use:** When you need to traverse the AST for analysis, transformation, or code generation.

**Example:**
```rust
pub trait Visitor<'a> {
    fn visit_node(&mut self, node: &'a Node<'a>) {
        // Default implementation just visits children
    }
    fn visit_function(&mut self, func: &'a Function<'a>) {
        // Override for function-specific logic
    }
    fn visit_expression(&mut self, expr: &'a Expression<'a>) {
        // Override for expression-specific logic
    }
}
```

### Pattern 4: Span Tracking with Line Map

**What:** Store byte offsets in spans, maintain a separate line map for converting to line/column on demand.

**When to use:** When you need source location information but want to avoid storing redundant data.

**Example:**
```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,  // Byte offset (zero-based)
    pub end: usize,    // Byte offset (exclusive)
}

pub struct LineMap {
    lines: Vec<usize>,  // Starting byte offset of each line
}

impl LineMap {
    pub fn lookup(&self, offset: usize) -> (usize, usize) {
        // Returns (line, column) - display as 1-based
        let line = self.lines.partition_point(|&l| l <= offset) - 1;
        let col = offset - self.lines[line];
        (line + 1, col + 1)  // Convert to 1-based for display
    }
}
```

### Pattern 5: Three-Layer Error Handling

**What:** Use thiserror for structured library errors, anyhow for application-level error propagation, and miette for fancy diagnostics.

**When to use:** Throughout the codebase for comprehensive error handling.

**Example:**
```rust
// Library layer: thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid syntax at {span}")]
    InvalidSyntax { span: Span },
    #[error("Deno process failed: {0}")]
    DenoError(#[source] std::io::Error),
}

// Application layer: anyhow
use anyhow::{Context, Result};

fn parse_file(path: &Path) -> Result<Ast> {
    let source = std::fs::read_to_string(path)
        .context("Failed to read source file")?;
    let backend = DenoBackend::new();
    backend.parse(&source)
        .context("Failed to parse TypeScript source")
}

// Display layer: miette
use miette::{Diagnostic, SourceSpan};

#[derive(Debug, Diagnostic)]
#[error("Parse error")]
#[diagnostic(code(parse::invalid))]
pub struct ParseDiagnostic {
    #[source_code]
    src: String,
    #[label("here")]
    span: SourceSpan,
}
```

### Anti-Patterns to Avoid

- **Storing source text in AST nodes:** Store only spans and reference the original source file for text reconstruction
- **Clone-heavy AST:** Use arena allocator with references instead of cloning nodes
- **Blocking subprocess:** Use tokio::process for non-blocking Deno communication
- **Swallowing errors:** Always propagate errors with context using anyhow::Context
- **Manual span calculation:** Build line map during parsing for efficient line/column lookup
- **JSON for subprocess IPC:** Use MessagePack (rmp-serde) for faster, more compact serialization
- **Global mutable state:** Use arena lifetime to pass context instead of global state

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| AST memory allocation | Custom Vec-based pooling | bumpalo arena | Provides safety guarantees, lifetime tracking, and bulk deallocation |
| Message serialization | Custom binary protocol | rmp-serde | Well-tested, efficient, integrates with serde derive |
| Async subprocess | Manual process spawning | tokio::process | Handles async I/O, proper resource cleanup, and cross-platform concerns |
| Error types | Manual std::error::Error impl | thiserror derive | Reduces boilerplate, ensures correct implementation |
| Error reporting | Custom formatting | miette | Provides rich diagnostics with code snippets, source span highlighting |
| Type serialization | Manual JSON/string parsing | serde derive | Handles complex types automatically, reduces bugs |

**Key insight:** The Rust ecosystem provides mature, well-tested solutions for all the core challenges in this phase. Custom implementations would increase complexity, reduce reliability, and likely be less performant.

## Common Pitfalls

### Pitfall 1: Arena Lifetime Confusion

**What goes wrong:** Trying to return AST nodes from the arena to callers outside the arena's lifetime scope.

**Why it happens:** Arena-allocated references are tied to the arena's lifetime and cannot outlive it.

**How to avoid:**
- Keep parsing as a self-contained phase that uses the AST within the same scope
- Return analysis results derived from the AST, not the AST nodes themselves
- Store the arena in a struct that owns the analysis results

**Warning signs:**
- Compiler errors about lifetime parameters not living long enough
- Attempts to store `&Node` in structs without including the arena lifetime

### Pitfall 2: Deno Subprocess Deadlock

**What goes wrong:** The Deno subprocess hangs waiting for input or the Rust process hangs waiting for output.

**Why it happens:** Not properly closing stdin after writing, not reading both stdout and stderr, or not awaiting process completion.

**How to avoid:**
- Always drop stdin after writing to signal EOF to the child process
- Use `wait_with_output()` instead of manually handling pipes when possible
- Set timeouts for subprocess operations
- Kill child process on error

**Warning signs:**
- Application appears to hang without error messages
- CPU usage goes to 0 while waiting
- No output despite successful subprocess spawn

### Pitfall 3: Span Off-By-One Errors

**What goes wrong:** Error reporting shows incorrect source locations, often by one character or line.

**Why it happens:** Mixing zero-based internal representation with one-based display format without proper conversion.

**How to avoid:**
- Always use zero-based indexing internally (byte offsets, line/column)
- Only convert to one-based for display in error messages
- Include clear comments about index conventions
- Write tests that verify span-to-display conversion

**Warning signs:**
- Error diagnostics point to wrong locations
- Test failures related to span calculations

### Pitfall 4: MessagePack Serde Compatibility

**What goes wrong:** Deserialization fails because the Deno process sends data in a different format than Rust expects.

**Why it happens:** MessagePack has multiple representations for the same data (e.g., byte arrays as object arrays or binary), and Serde defaults may differ.

**How to avoid:**
- Use `with_bytes()` configuration for binary data
- Test serialization/deserialization roundtrip with the same data
- Consider using `rmp-serde::to_vec_named` for compatibility
- Document the exact MessagePack format in Deno bridge code

**Warning signs:**
- Deserialization errors with "invalid type" messages
- Inconsistent data between send and receive

### Pitfall 5: Error Context Loss

**What goes wrong:** Error messages lack useful context, making debugging difficult.

**Why it happens:** Using `?` without adding context or using `unwrap()` instead of proper error handling.

**How to avoid:**
- Always use `.context()` or `.with_context()` when propagating errors with anyhow
- Include relevant values in error messages (file paths, line numbers, etc.)
- Use `bail!()` and `ensure!()` macros for early error returns
- Never use `unwrap()` in production code

**Warning signs:**
- Generic error messages like "failed to parse"
- Stack traces without function names
- Missing file paths in error messages

### Pitfall 6: Partial AST Recovery Complexity

**What goes wrong:** Attempting to recover from parsing errors produces an invalid AST that causes cascading errors.

**Why it happens:** Error recovery strategies are complex and can introduce invalid structure.

**How to avoid:**
- Start with simple error recovery (abort on first error)
- Gradually add recovery for well-understood cases
- Validate AST structure after recovery
- Consider storing errors as nodes in the AST for later reporting

**Warning signs:**
- Panics in downstream analysis phases
- Inconsistent error counts
- Tests failing on malformed input

## Code Examples

Verified patterns from official sources:

### Arena Allocation for AST

```rust
// Source: https://github.com/fitzgen/bumpalo
use bumpalo::Bump;

struct AstArena<'a> {
    bump: Bump,
    // Store root node reference
    root: Option<&'a AstNode<'a>>,
}

impl<'a> AstArena<'a> {
    fn new() -> Self {
        Self {
            bump: Bump::new(),
            root: None,
        }
    }

    fn alloc_node(&'a self, kind: NodeKind, span: Span) -> &'a AstNode<'a> {
        self.bump.alloc(AstNode {
            kind,
            span,
            children: Vec::new_in(&self.bump),
        })
    }
}
```

### Deno Subprocess with MessagePack

```rust
// Source: tokio::process + rmp-serde docs
use tokio::process::Command;
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ParseRequest {
    source: String,
}

#[derive(Serialize, Deserialize)]
struct ParseResponse {
    ast: AstData,
    errors: Vec<ParseError>,
}

async fn parse_with_deno(source: &str) -> Result<ParseResponse> {
    let request = ParseRequest {
        source: source.to_string(),
    };

    // Serialize request as MessagePack
    let input = to_vec(&request)?;

    // Spawn Deno subprocess
    let mut child = Command::new("deno")
        .arg("run")
        .arg("--allow-read")
        .arg("bridge/deno_parser.ts")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write request to stdin
    let mut stdin = child.stdin.take().ok_or("Missing stdin")?;
    stdin.write_all(&input).await?;
    drop(stdin);  // Signal EOF

    // Wait for output
    let output = child.wait_with_output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Deno parser failed: {}", stderr);
    }

    // Deserialize response
    from_slice(&output.stdout).context("Failed to parse Deno response")
}
```

### Error Handling with thiserror

```rust
// Source: https://docs.rs/thiserror/latest/thiserror/
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Source file not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid TypeScript syntax at {line}:{column}")]
    InvalidSyntax { line: usize, column: usize },

    #[error("Deno process error: {0}")]
    DenoError(#[source] std::io::Error),

    #[error("MessagePack deserialization failed")]
    DeserializationError(#[from] rmp_serde::decode::Error),
}

// Use with anyhow for context
use anyhow::{Context, Result};

fn parse_source(source: &str) -> Result<Ast> {
    let backend = DenoBackend::new()?;
    backend.parse(source)
        .context("Failed to parse TypeScript source")
}
```

### Span and Location Tracking

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,  // Zero-based byte offset
    pub end: usize,    // Zero-based byte offset (exclusive)
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "Span start: {}, end: {}", start, end);
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

pub struct LineMap {
    line_starts: Vec<usize>,
}

impl LineMap {
    pub fn from_source(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_starts
            .partition_point(|&l| l <= offset)
            .saturating_sub(1);
        let line_start = self.line.line_starts.get(line).copied().unwrap_or(0);
        let col = offset - line_start;
        (line + 1, col + 1)  // 1-based for display
    }
}
```

### Visitor Pattern Implementation

```rust
pub trait Visitor<'a> {
    fn visit_node(&mut self, node: &'a AstNode<'a>) {
        match &node.kind {
            NodeKind::Function(f) => self.visit_function(f),
            NodeKind::Expression(e) => self.visit_expression(e),
            NodeKind::Statement(s) => self.visit_statement(s),
        }
    }

    fn visit_function(&mut self, func: &'a Function<'a>);
    fn visit_expression(&mut self, expr: &'a Expression<'a>);
    fn visit_statement(&mut self, stmt: &'a Statement<'a>);
}

// Example visitor: count nodes
pub struct NodeCounter {
    count: usize,
}

impl<'a> Visitor<'a> for NodeCounter {
    fn visit_node(&mut self, node: &'a AstNode<'a>) {
        self.count += 1;
        for child in &node.children {
            self.visit_node(child);
        }
    }

    fn visit_function(&mut self, _func: &'a Function<'a>) {}
    fn visit_expression(&mut self, _expr: &'a Expression<'a>) {}
    fn visit_statement(&mut self, _stmt: &'a Statement<'a>) {}
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual AST nodes with Box pointers | Arena allocator with references | 2020-2021 | Significant memory reduction and performance improvement for parsers |
| Custom binary serialization | Serde-based (serde_json / rmp-serde) | 2018-2019 | Standardized, type-safe serialization ecosystem |
| Blocking subprocess communication | tokio::process async | 2019-2020 | Better resource utilization and cancellation support |
| fmt-based error messages | Structured errors with miette | 2021-2023 | Rich diagnostics with source code snippets and suggestions |

**Deprecated/outdated:**
- std::sync::mpsc for async code: Use tokio channels instead
- serde_json for IPC when performance matters: rmp-serde is 2-3x faster
- Manual std::error::Error implementations: Use thiserror derive

## Open Questions

1. **Deno TypeScript AST format specification**
   - What we know: Deno uses TypeScript compiler internally, but standard AST module was not found in current docs
   - What's unclear: Exact structure of Deno's TypeScript AST output, whether JSON or MessagePack representation is available
   - Recommendation: Build a small Deno bridge script that uses TypeScript Compiler API to serialize AST to JSON/MessagePack, then define Rust types to match

2. **MessagePack format for AST data**
   - What we know: rmp-serde provides Serde integration for MessagePack
   - What's unclear: Whether Deno can natively output MessagePack or if we need JSON+convert
   - Recommendation: Start with JSON for simplicity (easier debugging), profile performance, switch to MessagePack if needed

3. **Error recovery granularity**
   - What we know: Context.md specifies "Partial AST recovery"
   - What's unclear: Which errors should be recoverable vs fatal
   - Recommendation: Implement basic recovery (skip tokens) for simple syntax errors, abort on structural errors for Phase 02

4. **Deno process pool management**
   - What we know: "Configurable process lifecycle" from Context.md
   - What's unclear: Optimal pool size, timeout values, startup cost
   - Recommendation: Start with single persistent process, add pooling if performance tests show benefit

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (cargo test) |
| Config file | None (using default) |
| Quick run command | `cargo test --package RustifyTS --lib parser:: -- -x` |
| Full suite command | `cargo test --package RustifyTS --lib` |

### Phase Requirements → Test Map

No specific requirement IDs defined for this phase. Testing will be based on deliverables:

| Behavior | Test Type | Automated Command | File Exists? |
|----------|-----------|-------------------|-------------|
| Arena allocation | unit | `cargo test parser::ast::tests::test_arena_allocation -- -x` | ❌ Wave 0 |
| Span calculation | unit | `cargo test parser::ast::tests::test_span_line_col -- -x` | ❌ Wave 0 |
| Deno subprocess communication | integration | `cargo test parser::backend::deno::tests::test_deno_parse -- -x` | ❌ Wave 0 |
| MessagePack serialization | unit | `cargo test parser::backend::deno::tests::test_msgpack_roundtrip -- -x` | ❌ Wave 0 |
| Visitor pattern traversal | unit | `cargo test parser::ast::tests::test_visitor -- -x` | ❌ Wave 0 |
| Error propagation | unit | `cargo test parser::error::tests::test_error_context -- -x` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --package RustifyTS --lib parser:: -- -x`
- **Per wave merge:** `cargo test --package RustifyTS --lib`
- **Phase gate:** All tests passing before proceeding to Phase 03

### Wave 0 Gaps

- [ ] `src/parser/mod.rs` - Parser module interface
- [ ] `src/parser/backend/mod.rs` - Backend module
- [ ] `src/parser/backend/trait.rs` - ParserBackend trait
- [ ] `src/parser/backend/denodeno.rs` - Deno subprocess implementation
- [ ] `src/parser/ast/mod.rs` - AST module
- [ ] `src/parser/ast/types.rs` - AST type definitions
- [ ] `src/parser/ast/node.rs` - AstNode and builder
- [ ] `src/parser/ast/visitor.rs` - Visitor trait
- [ ] `src/parser/ast/span.rs` - Span and LineMap
- [ ] `src/parser/error.rs` - Error type definitions
- [ ] Test fixtures: TypeScript files for integration tests
- [ ] Deno bridge script: TypeScript script that uses Compiler API to serialize AST

*(No existing test infrastructure - all files must be created in Phase 02)*

## Sources

### Primary (HIGH confidence)

- /websites/rs_bumpalo - Bumpalo arena allocator API
- https://github.com/fitzgen/bumpalo - Bumpalo GitHub repository
- /websites/rs_rmp-serde - rmp-serde serialization API
- /dtolnay/thiserror - thiserror derive macro API
- /zkat/miette - Miette diagnostic library features
- /websites/rs_anyhow - anyhow error handling API
- https://docs.rs/tokio/latest/tokio/process/ - Tokio async subprocess management
- https://docs.rs/rust/latest/std/process/ - Rust standard process API
- https://serde.rs/derive.html - Serde derive macro documentation

### Secondary (MEDIUM confidence)

- /websites/rs_serde - Serde serialization framework
- https://docs.rs/anyhow/latest/anyhow/ - anyhow API documentation
- C:/Users/16017/Documents/RustifyTS/.planning/research/SUMMARY.md - Domain research summary

### Tertiary (LOW confidence)

- WebSearch for "Deno TypeScript AST" - No authoritative source found for Deno AST format
- WebSearch for "TypeScript Compiler API" - No recent documentation retrieved

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries are well-documented with official sources
- Architecture: HIGH - Patterns are well-established in Rust ecosystem
- Pitfalls: HIGH - Based on common Rust parser implementation issues
- Deno AST format: LOW - Could not find official documentation for Deno's AST output format

**Research date:** 2026-03-01
**Valid until:** 2026-04-01 (30 days - stable Rust ecosystem)

---

*Phase: 02-parser*
*Research generated: 2026-03-01*
