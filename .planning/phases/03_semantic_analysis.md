# Phase 03: 语义分析层

## 目标

实现基础 IR 构建，包括作用域分析、符号表构建、类型推断和控制流分析。

## 实施计划

### 任务清单

- [ ] 创建 `src/semantic/mod.rs` - 模块接口
- [ ] 创建 `src/semantic/analyzer.rs` - 主分析器
- [ ] 创建 `src/semantic/scope/mod.rs` - 作用域模块
- [ ] 创建 `src/semantic/scope/analyzer.rs` - 作用域分析器
- [ ] 创建 `src/semantic/scope/table.rs` - 作用域表
- [ ] 创建 `src/semantic/symbol/mod.rs` - 符号表模块
- [ ] 创建 `src/semantic/symbol/table.rs` - 符号表实现
- [ ] 创建 `src/semantic/type/mod.rs` - 类型系统模块
- [ ] 创建 `src/semantic/type/infer.rs` - 类型推断
- [ ] 创建 `src/semantic/type/unify.rs` - 类型统一
- [ ] 创建 `src/semantic/flow/mod.rs` - 控制流模块
- [ ] 创建 `src/semantic/flow/cfg.rs` - 控制流图
- [ ] 创建 `src/semantic/flow/dominance.rs` - 支配关系

### 关键设计

#### 主分析器

```rust
use crate::parser::ast::AstNode;
use anyhow::Result;

pub struct SemanticAnalyzer {
    scope_analyzer: scope::ScopeAnalyzer,
    symbol_table: symbol::SymbolTable,
    type_inferer: type::TypeInferer,
    flow_analyzer: flow::FlowAnalyzer,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scope_analyzer: scope::ScopeAnalyzer::new(),
            symbol_table: symbol::SymbolTable::new(),
            type_inferer: type::TypeInferer::new(),
            flow_analyzer: flow::FlowAnalyzer::new(),
        }
    }

    pub fn analyze(&mut self, ast: &AstNode) -> Result<SemanticModule> {
        // 1. 作用域分析
        self.scope_analyzer.analyze(ast)?;

        // 2. 符号表构建
        self.symbol_table.build(ast)?;

        // 3. 类型推断
        self.type_inferer.infer(ast)?;

        // 4. 控制流分析
        self.flow_analyzer.analyze(ast)?;

        Ok(SemanticModule {
            name: "module".to_string(),
            functions: self.extract_functions(),
            types: self.extract_types(),
        })
    }
}
```

#### 语义 IR 类型

```rust
/// 语义模块
#[derive(Debug, Clone)]
pub struct SemanticModule {
    pub name: String,
    pub functions: Vec<Function>,
    pub types: HashMap<String, Type>,
}

/// 函数
#[derive(Debug, Clone)]
pub struct Function {
    pub id: NodeId,
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Expr,
    pub span: Span,
}

/// 参数
#[derive(Debug, Clone)]
pub struct Parameter {
    pub id: NodeId,
    pub name: String,
    pub type_annotation: Option<Type>,
    pub default_value: Option<Expr>,
}

/// 表达式
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Member {
        object: Box<Expr>,
        property: String,
    },
    Block(Vec<Expr>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Function(Function),
    Variable(Variable),
}

/// 类型系统
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Never,
    Null,
    Undefined,

    // 原始类型
    Boolean,
    Number,
    String,

    // 复合类型
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Object(HashMap<String, Type>),

    // 高级类型
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Option(Box<Type>),

    // 函数类型
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    // 泛型
    Generic(String),
    TypeParameter(String),
}

/// 作用域
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub symbols: HashMap<String, Symbol>,
}

/// 符号
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub type_annotation: Option<Type>,
    pub scope: ScopeId,
}
```

#### 类型推断器

```rust
pub struct TypeInferer {
    environment: HashMap<String, Type>,
    constraints: Vec<TypeConstraint>,
}

impl TypeInferer {
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    pub fn infer(&mut self, ast: &AstNode) -> Result<()> {
        // 推断整个 AST 的类型
        self.infer_node(ast)
    }

    fn infer_node(&mut self, node: &AstNode) -> Result<Type> {
        match &node.kind {
            AstKind::Literal => self.infer_literal(node),
            AstKind::Identifier => self.infer_identifier(node),
            AstKind::BinaryExpression => self.infer_binary(node),
            AstKind::CallExpression => self.infer_call(node),
            _ => Ok(Type::Unknown),
        }
    }

    fn infer_literal(&mut self, node: &AstNode) -> Result<Type> {
        // 根据字面量值推断类型
        if let Some(value) = node.properties.get("value") {
            if value.is_boolean() {
                return Ok(Type::Boolean);
            } else if value.is_number() {
                return Ok(Type::Number);
            } else if value.is_string() {
                return Ok(Type::String);
            }
        }
        Ok(Type::Unknown)
    }

    fn infer_identifier(&mut self, node: &AstNode) -> Result<Type> {
        // 从环境查找变量类型
        if let Some(name) = node.properties.get("name") {
            if let Some(name) = name.as_str() {
                if let Some(ty) = self.environment.get(name) {
                    return Ok(ty.clone());
                }
            }
        }
        Ok(Type::Unknown)
    }

    fn infer_binary(&mut self, node: &AstNode) -> Result<Type> {
        // 推断二元表达式类型
        // 算术运算符返回 Number
        // 比较运算符返回 Boolean
        // 逻辑运算符返回 Boolean
        Ok(Type::Number)
    }
}
```

### 测试计划

#### 作用域测试

```rust
#[cfg(test)]
mod scope_tests {
    use super::*;

    #[test]
    fn test_nested_scopes() {
        let mut analyzer = ScopeAnalyzer::new();
        // 创建嵌套作用域
        let parent_id = analyzer.create_scope(None);
        let child_id = analyzer.create_scope(Some(parent_id));

        assert_eq!(analyzer.get_scope(child_id).parent, Some(parent_id));
    }
}
```

#### 类型推断测试

```rust
#[cfg(test)]
mod type_inference_tests {
    use super::*;

    #[test]
    fn test_literal_inference() {
        let mut inferer = TypeInferer::new();

        let bool_type = inferer.infer_literal(true).unwrap();
        assert_eq!(bool_type, Type::Boolean);

        let number_type = inferer.infer_literal(42).unwrap();
        assert_eq!(number_type, Type::Number);

        let string_type = inferer.infer_literal("hello").unwrap();
        assert_eq!(string_type, Type::String);
    }
}
```

## 验收标准

- [ ] `src/semantic/` 模块编译通过
- [ ] 作用域分析器功能完整
- [ ] 符号表构建功能完整
- [ ] 类型推断功能完整
- [ ] 控制流分析功能完整
- [ ] 可以从 AST 构建完整的 SemanticModule
- [ ] 单元测试通过
- [ ] 集成测试通过

## 依赖

Phase 02 必须完成

## 估计时间

5 天

## 风险

1. **类型系统复杂性** - TypeScript 的类型系统非常复杂
   - 缓解：先实现基础类型，逐步扩展

2. **类型推断准确性** - 某些情况下类型推断可能不准确
   - 缓解：提供类型注解提示

## 下一步

完成后进入 Phase 04: 语义重构核心层
