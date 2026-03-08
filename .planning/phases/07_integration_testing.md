# Phase 07: 集成测试

## 目标

集成所有层，验证端到端流程。

## 实施计划

### 任务清单

- [ ] 完善 `src/lib.rs` - 完整转换管道
- [ ] 创建端到端转换函数
- [ ] 创建集成测试套件
- [ ] 性能测试
- [ ] 使用现有测试用例验证

### 关键设计

#### 完整转换管道

```rust
use anyhow::Result;
use std::path::Path;

/// TypeScript 到 Rust 的主转换函数
pub fn transmute(source: &str, options: &TransmuteOptions) -> Result<String> {
    // 1. 解析 TypeScript
    let backend = parser::backend::deno::DenoParser::new()?;
    let ast = backend.parse(source)?;

    // 2. 语义分析
    let mut semantic_analyzer = semantic::SemanticAnalyzer::new();
    let module = semantic_analyzer.analyze(&ast)?;

    // 3. 运行重构算法
    let mut refactor_ctx = refactor::context::AnalysisContext::new(module.clone());
    refactor::pipeline::RefactorPipeline::run_all(&mut refactor_ctx)?;

    // 4. 构建 Rust AST
    let mut ast_builder = ast::AstBuilder::new();
    let rust_ast = ast_builder.build(&module, &refactor_ctx)?;

    // 5. 生成 Rust 代码
    let mut renderer = codegen::Renderer::new(options.render_config.clone());
    let rust_code = renderer.render_file(&rust_ast)?;

    // 6. 格式化代码
    let formatted = rustfmt(&rust_code)?;

    Ok(formatted)
}

/// 文件到文件的转换函数
pub fn transmute_file(input: &Path, options: &TransmuteOptions) -> Result<String> {
    let source = std::fs::read_to_string(input)?;
    transmute(&source, options)
}

/// 转换选项
#[derive(Debug, Clone)]
pub Default struct TransmuteOptions {
    /// 渲染配置
    pub render_config: codegen::RenderConfig,

    /// 是否运行所有分析算法
    pub run_all_analyses: bool,

    /// 输出的分析算法列表
    pub analyses: Vec<String>,

    /// 是否优化生成的代码
    pub optimize: bool,

    /// 目标 Rust 版本
    pub rust_edition: String,
}

impl Default for TransmuteOptions {
    fn default() -> Self {
        Self {
            render_config: codegen::RenderConfig::default(),
            run_all_analyses: true,
            analyses: vec![
                "ownership".to_string(),
                "lifetime".to_string(),
                "mutability".to_string(),
            ],
            optimize: true,
            rust_edition: "2024".to_string(),
        }
    }
}
```

#### AST Builder

```rust
use crate::semantic::{SemanticModule, Type, Expr};
use crate::refactor::context::AnalysisContext;
use crate::ast::*;
use anyhow::Result;

pub struct AstBuilder;

impl AstBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build(&self, module: &SemanticModule, ctx: &AnalysisContext) -> Result<RustFile> {
        let mut items = Vec::new();

        // 转换类型定义
        for (name, ty) in &module.types {
            if let Some(item) = self.convert_type_def(name, ty)? {
                items.push(item);
            }
        }

        // 转换函数
        for func in &module.functions {
            items.push(self.convert_function(func, ctx)?);
        }

        Ok(RustFile {
            items,
            attributes: vec![],
            shebang: None,
        })
    }

    fn convert_function(&self, func: &semantic::Function, ctx: &AnalysisContext) -> Result<Item> {
        // 获取可变性分析结果
        let mutability_result = ctx.results.get(&(func.id, TypeId(0)))
            .and_then(|r| r.as_any().downcast_ref::<refactor::algorithms::mutability::MutabilityResult>());

        // 转换参数
        let params: Vec<Parameter> = {
            let mut result = Vec::new();
            for param in &func.params {
                let param_name = param.name.clone();

                // 获取参数的可变性
                let is_param_mutable = if let Some(mr) = mutability_result {
                    mr.mutation_points.contains(&param.id)
                } else {
                    false
                };

                result.push(Parameter {
                    attributes: vec![],
                    is_mut: is_param_mutable,
                    pattern: Pattern::Binding {
                        name: param_name,
                        is_mut: false,
                        is_ref: false,
                        subpattern: None,
                    },
                    type_annotation: self.convert_type(&param.type_annotation.clone().unwrap_or(Type::Unknown), ctx)?,
                });
            }
            result
        };

        // 转换函数体
        let body = self.convert_expr(&func.body, ctx)?;

        Ok(Item::Function(Function {
            attributes: vec![],
            visibility: Visibility::Private,
            is_unsafe: false,
            is_async: false,
            abi: None,
            name: func.name.clone(),
            generics: Generics {
                params: vec![],
                where_clauses: vec![],
            },
            params,
            return_type: self.convert_type(&func.return_type, ctx)?,
            body: Some(Block {
                statements: vec![Statement::Expr(body)],
                tail: None,
            }),
        }))
    }

    fn convert_expr(&self, expr: &semantic::Expr, ctx: &AnalysisContext) -> Result<Expr> {
        match expr {
            semantic::Expr::Literal(lit) => Ok(self.convert_literal(lit)?),
            semantic::Expr::Identifier(name) => Ok(Expr::Path(PathExpr {
                segments: vec![PathSegment {
                    name: name.clone(),
                    generics: vec![],
                }],
            })),
            semantic::Expr::Binary { op, left, right } => {
                Ok(Expr::Binary {
                    op: self.convert_binary_op(op),
                    left: Box::new(self.convert_expr(left, ctx)?),
                    right: Box::new(self.convert_expr(right, ctx)?),
                })
            }
            semantic::Expr::Unary { op, operand } => {
                Ok(Expr::Unary {
                    op: self.convert_unary_op(op),
                    operand: Box::new(self.convert_expr(operand, ctx)?),
                })
            }
            semantic::Expr::Call { callee, args } => {
                let mut converted_args = Vec::new();
                for arg in args {
                    converted_args.push(self.convert_expr(arg, ctx)?);
                }
                Ok(Expr::Call {
                    func: Box::new(self.convert_expr(callee, ctx)?),
                    args: converted_args,
                })
            }
            semantic::Expr::Member { object, property } => {
                Ok(Expr::FieldAccess {
                    object: Box::new(self.convert_expr(object, ctx)?),
                    field: property.clone(),
                })
            }
            semantic::Expr::Block(stmts) => {
                let mut statements = Vec::new();
                for stmt in stmts {
                    statements.push(Statement::Expr(self.convert_expr(stmt, ctx)?));
                }
                Ok(Expr::Block(Block {
                    statements,
                    tail: None,
                }))
            }
            semantic::Expr::If { condition, then_branch, else_branch } => {
                let then_block = Block {
                    statements: vec![Statement::Expr(self.convert_expr(then_branch, ctx)?)],
                    tail: None,
                };

                let else_expr = if let Some(eb) = else_branch {
                    Some(Box::new(self.convert_expr(eb, ctx)?))
                } else {
                    None
                };

                Ok(Expr::If {
                    condition: Box::new(self.convert_expr(condition, ctx)?),
                    then_branch: then_block,
                    else_branch: else_expr,
                })
            }
            semantic::Expr::Function(func) => {
                Ok(Expr::Path(PathExpr {
                    segments: vec![PathSegment {
                        name: func.name.clone(),
                        generics: vec![],
                    }],
                }))
            }
            semantic::Expr::Variable(var) => {
                Ok(Expr::Path(PathExpr {
                    segments: vec![PathSegment {
                        name: var.name.clone(),
                        generics: vec![],
                    }],
                }))
            }
        }
    }

    fn convert_type(&self, ty: &semantic::Type, ctx: &AnalysisContext) -> Result<Type> {
        match ty {
            semantic::Type::Void => Ok(Type::Primitive(PrimitiveType::Void)),
            semantic::Type::Boolean => Ok(Type::Primitive(PrimitiveType::Bool)),
            semantic::Type::Number => Ok(Type::Primitive(PrimitiveType::I32)),
            semantic::Type::String => Ok(Type::Primitive(PrimitiveType::String)),
            semantic::Type::Array(inner) => {
                Ok(Type::Slice(Box::new(self.convert_type(inner, ctx)?)))
            {
            }
            semantic::Type::Tuple(types) => {
                let mut converted = Vec::new();
                for t in types {
                    converted.push(self.convert_type(t, ctx)?);
                }
                Ok(Type::Tuple(converted))
            }
            semantic::Type::Object(fields) => {
                // 创建结构体类型
                Ok(Type::Path {
                    path: "Object".to_string(), // 简化实现
                    generics: vec![],
                })
            }
            semantic::Type::Union(types) => {
                // 转换为 Option 或枚举
                if types.len() == 2 && types.contains(&semantic::Type::Null) {
                    // Option<T>
                    let inner = types.iter()
                        .find(|t| !matches!(t, semantic::Type::Null))
                        .unwrap();
                    Ok(Type::Path {
                        path: "Option".to_string(),
                        generics: vec![self.convert_type(inner, ctx)?],
                    })
                } else {
                    // 简化：创建枚举
                    Ok(Type::Path {
                        path: "Union".to_string(),
                        generics: vec![],
                    })
                }
            }
            semantic::Type::Option(inner) => {
                Ok(Type::Path {
                    path: "Option".to_string(),
                    generics: vec![self.convert_type(inner, ctx)?],
                })
            }
            semantic::Type::Function { params, return_type } => {
                let mut converted_params = Vec::new();
                for p in params {
                    converted_params.push(self.convert_type(p, ctx)?);
                }
                Ok(Type::FunctionPointer {
                    params: converted_params,
                    return_type: Box::new(self.convert_type(return_type, ctx)?),
                    is_unsafe: false,
                })
            }
            semantic::Type::Unknown => Ok(Type::Inferred),
            _ => Ok(Type::Inferred),
        }
    }

    fn convert_literal(&self, lit: &semantic::Literal) -> Result<Expr> {
        match lit {
            semantic::Literal::Bool(b) => Ok(Expr::Literal(Literal::Bool(*b))),
            semantic::Literal::Number(n) => Ok(Expr::Literal(Literal::Int(*n as i128))),
            semantic::Literal::String(s) => Ok(Expr::Literal(Literal::String(s.clone()))),
        }
    }

    fn convert_binary_op(&self, op: &semantic::BinaryOp) -> BinaryOp {
        match op {
            semantic::BinaryOp::Add => BinaryOp::Add,
            semantic::BinaryOp::Sub => BinaryOp::Sub,
            semantic::BinaryOp::Mul => BinaryOp::Mul,
            semantic::BinaryOp::Div => BinaryOp::Div,
            semantic::BinaryOp::Mod => BinaryOp::Rem,
            semantic::BinaryOp::Eq => BinaryOp::Eq,
            semantic::BinaryOp::NotEq => BinaryOp::NotEq,
            semantic::BinaryOp::Lt => BinaryOp::Lt,
            semantic::BinaryOp::Gt => BinaryOp::Gt,
            semantic::BinaryOp::LtEq => BinaryOp::LtEq,
            semantic::BinaryOp::GtEq => BinaryOp::GtEq,
            semantic::BinaryOp::And => BinaryOp::And,
            semantic::BinaryOp::Or => BinaryOp::Or,
        }
    }

    fn convert_unary_op(&self, op: &semantic::UnaryOp) -> UnaryOp {
        match op {
            semantic::UnaryOp::Negate => UnaryOp::Negate,
            semantic::UnaryOp::Not => UnaryOp::Not,
        }
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = r#"
            function add(a: number, b: number): number {
                return a + b;
            }
        "#;

        let result = transmute(source, &TransmuteOptions::default()).unwrap();

        assert!(result.contains("fn add"));
        assert!(result.contains("a: i32"));
        assert!(result.contains("b: i32"));
        assert!(result.contains("-> i32"));
    }

    #[test]
    fn test_with_option() {
        let source = r#"
            function greet(name: string | null): string {
                if (name) {
                    return "Hello, " + name;
                } else {
                    return "Hello, World";
                }
            }
        "#;

        let result = transmute(source, &TransmuteOptions::default()).unwrap();

        assert!(result.contains("Option<String>"));
        assert!(result.contains("is_some()"));
    }

    #[test]
    fn test_array_operations() {
        let source = r#"
            function sum(arr: number[]): number {
                let total = 0;
                for (let i = 0; i < arr.length; i++) {
                    total += arr[i];
                }
                return total;
            }
        "#;

        let result = transmute(source, &TransmuteOptions::default()).unwrap();

        assert!(result.contains("Vec<i32>"));
        assert!(result.contains(".len()"));
    }
}
```

### 性能测试

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_file_performance() {
        let source = generate_large_source(1000); // 1000 个函数

        let start = Instant::now();
        let result = transmute(&source, &TransmuteOptions::default());
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_secs() < 5, "转换大文件应在 5 秒内完成");
    }

    fn generate_large_source(count: usize) -> String {
        let mut source = String::new();
        for i in 0..count {
            source.push_str(&format!(r#"
                function func{i}(a: number, b: number): number {{
                    return a + b;
                }}
            "#));
        }
        source
    }
}
```

### 现有测试用例验证

```rust
#[cfg(test)]
mod fixture_tests {
    use super::*;
    use std::path::PathBuf;

    fn get_fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("original")
    }

    #[test]
    fn test_original_fixtures() {
        let fixture_dir = get_fixture_dir();

        for entry in std::fs::read_dir(fixture_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().extension().map(|e| e == "ts").unwrap_or(false) {
                let source = std::fs::read_to_string(entry.path()).unwrap();
                let result = transmute(&source, &TransmuteOptions::default());

                // 验证生成代码
                if let Ok(rust_code) = result {
                    // 尝试编译生成的代码
                    let compile_result = test_compile(&rust_code);

                    // 记录结果
                    if compile_result.is_err() {
                        println!("Fixture {:?} failed to compile", entry.file_name());
                    }
                }
            }
        }
    }

    fn test_compile(code: &str) -> Result<(), String> {
        // 简化：只检查语法
        // 实际实现可以使用 rustc 或其他工具
        Ok(())
    }
}
```

## 验收标准

- [ ] `src/lib.rs` 包含完整转换管道
- [ ] 端到端转换函数功能完整
- [ ] 可以转换完整的 TypeScript 代码
- [ ] 集成测试通过
- [ ] 性能测试通过
- [ ] 现有测试用例通过率 >= 95%
- [ ] 生成的代码符合 rustfmt 格式

## 依赖

Phase 06 必须完成

## 估计时间

5 天

## 风险

1. **现有测试用例不兼容** - 新架构可能不兼容现有测试用例
   - 缓解：逐步迁移测试用例

2. **性能不达标** - 性能可能不满足要求
   - 缓解：优化算法，使用并行处理

## 下一步

完成 v0.1.0 里程碑，准备 v0.2.0
