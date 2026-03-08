# Phase 02: 解析层

## 目标

实现基础解析层，支持 Deno 后端。

## 实施计划

### 任务清单

- [ ] 创建 `src/parser/mod.rs` - 模块接口
- [ ] 创建 `src/parser/backend/mod.rs` - 后端模块
- [ ] 创建 `src/parser/backend/trait.rs` - ParserBackend trait
- [ ] 创建 `src/parser/backend/deno.rs` - Deno 后端实现
- [ ] 创建 `src/parser/ast.rs` - 统一 AST 表示
- [ ] 创建 `src/parser/types.rs` - AST 类型定义
- [ ] 创建 `src/parser/error.rs` - 解析错误处理

### 关键设计

#### ParserBackend Trait

```rust
use anyhow::Result;
use std::path::Path;
use super::ast::{AstNode, AstKind};

pub trait ParserBackend: Send + Sync {
    /// 解析 TypeScript 源代码字符串
    fn parse(&self, source: &str) -> Result<AstNode>;

    /// 解析 TypeScript 文件
    fn parse_file(&self, path: &Path) -> Result<AstNode>;

    /// 获取后端名称
    fn name(&self) -> &'static str;
}
```

#### 统一 AST 表示

```rust
use serde::{Deserialize, Serialize};

/// AST 节点类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstKind {
    // 顶层节点
    SourceFile,
    ModuleDeclaration,
    ExportDeclaration,

    // 语句
    VariableStatement,
    FunctionDeclaration,
    IfStatement,
    ForStatement,
    WhileStatement,
    ReturnStatement,
    ExpressionStatement,
    BlockStatement,

    // 表达式
    Identifier,
    Literal,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrayLiteral,
    ObjectLiteral,
    ArrowFunction,
    AsExpression,
    TypeAssertion,

    // 类型
    TypeAnnotation,
    TypeAliasDeclaration,
    InterfaceDeclaration,
    InterfaceMethod,
    InterfaceProperty,
    UnionType,
    IntersectionType,

    // 其他
    Parameter,
    NamedImports,
}

/// AST 节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub kind: AstKind,
    pub span: Span,
    pub children: Vec<AstNode>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 源码位置
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}
```

#### Deno 后端实现

```rust
use std::process::Command;
use super::trait::ParserBackend;
use super::ast::AstNode;
use anyhow::{Result, Context};

pub struct DenoParser {
    deno_path: String,
    script_path: String,
}

impl DenoParser {
    pub fn new() -> Result<Self> {
        // 查找 deno 可执行文件
        let deno_path = Self::find_deno()?;

        // 保存或创建 TypeScript 解析脚本
        let script_path = "parser-server.deno.ts".to_string();

        Ok(Self {
            deno_path,
            script_path,
        })
    }

    fn find_deno() -> Result<String> {
        // 检查 PATH 中的 deno
        let output = Command::new("deno")
            .arg("--version")
            .output();

        match output {
            Ok(_) => Ok("deno".to_string()),
            Err(_) => anyhow::bail!("Deno not found. Please install Deno: https://deno.land/")
        }
    }
}

impl ParserBackend for DenoParser {
    fn parse(&self, source: &str) -> Result<AstNode> {
        // 调用 deno 脚本进行解析
        let output = Command::new(&self.deno_path)
            .arg("run")
            .arg(&self.script_path)
            .arg("--source")
            .arg(source)
            .output()
            .context("Failed to run Deno parser")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Deno parser failed: {}", stderr);
        }

        let json = String::from_utf8_lossy(&output.stdout);
        let ast: AstNode = serde_json::from_str(&json)
            .context("Failed to parse Deno output")?;

        Ok(ast)
    }

    fn parse_file(&self, path: &std::path::Path) -> Result<AstNode> {
        let source = std::fs::read_to_string(path)
            .context("Failed to read TypeScript file")?;

        self.parse(&source)
    }

    fn name(&self) -> &'static str {
        "deno"
    }
}
```

### 测试计划

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_node() {
        let node = AstNode {
            kind: AstKind::Literal,
            span: Span { start: 0, end: 5, line: 1, column: 1 },
            children: vec![],
            properties: HashMap::new(),
        };

        assert!(matches!(node.kind, AstKind::Literal));
    }
}
```

#### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_expression() {
        let parser = DenoParser::new().unwrap();
        let source = "const x = 42;";

        let ast = parser.parse(source).unwrap();
        assert!(matches!(ast.kind, AstKind::SourceFile));
    }

    #[tokio::test]
    async fn test_parse_function() {
        let parser = DenoParser::new().unwrap();
        let source = r#"
            function add(a: number, b: number): number {
                return a + b;
            }
        "#;

        let ast = parser.parse(source).unwrap();
        assert!(matches!(ast.kind, AstKind::SourceFile));
    }
}
```

## 验收标准

- [ ] `src/parser/` 模块编译通过
- [ ] `ParserBackend` trait 定义完整
- [ ] `DenoParser` 实现功能完整
- [ ] 可以解析简单的 TypeScript 代码
- [ ] `AstNode` 包含所有必要的 AST 节点类型
- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] 错误处理完善

## 依赖

Phase 01 必须完成

## 估计时间

3 天

## 风险

1. **Deno 可用性** - 用户可能没有安装 Deno
   - 缓解：提供清晰的错误消息和安装指南

2. **AST 结构差异** - 不同 TypeScript 解析器的 AST 结构可能不同
   - 缓解：使用统一 AST 表示，在 backend 层进行转换

## 下一步

完成后进入 Phase 03: 语义分析层
