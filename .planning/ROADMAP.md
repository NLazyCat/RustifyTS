# Transmute 开发路线图

## 当前里程碑: v0.1.0 - 基础转换

### 里程碑目标
构建基础转换管道，实现 TypeScript 到 Rust 的基础语法转换。

## 阶段划分

### Phase 01: 项目基础设施
**状态:** 🔴 未开始
**目标:** 搭建 GSD 结构，基础项目骨架

**交付物:**
- `.planning/` 目录结构
- `PROJECT.md` - 项目愿景
- `ROADMAP.md` - 路线图
- `HELP.md` - 项目帮助
- `FIX.md` - 待修复列表
- `FIXED.md` - 已完成列表
- `Cargo.toml` - 依赖配置（更新）
- `README.md` - 项目说明（更新）
- `src/lib.rs` - 基础库入口

**依赖:** 无
**估计时间:** 1 天

---

### Phase 02: 解析层
**状态:** ✅ 完成 (6/6 waves 完成)
**目标:** 实现基础解析层，支持 Deno 后端

**Waves Progress:**
- [x] Wave 1: Project Configuration and Error Types (Complete - f81d3c2)
- [x] Wave 2: Span and Location Tracking (Complete - 82c4844)
- [x] Wave 3: AST Types and Node Infrastructure (Complete - aa4b9a3)
- [x] Wave 4: Visitor Pattern (Complete - 2a80756)
- [x] Wave 5: Deno Backend Implementation (Complete - e23b98b)
- [x] Wave 6: Integration and Public API (Complete - 2026-03-07)

**交付物:**
- `src/parser/mod.rs` - 模块接口
- `src/parser/backend/trait.rs` - ParserBackend trait
- `src/parser/backend/deno.rs` - Deno 后端实现
- `src/parser/ast.rs` - 统一 AST 表示
- `src/parser/types.rs` - AST 类型定义

**关键接口:**
```rust
pub trait ParserBackend {
    fn parse(&self, source: &str) -> Result<AstNode>;
    fn parse_file(&self, path: &Path) -> Result<AstNode>;
}

pub struct AstNode {
    pub kind: AstKind,
    pub span: Span,
    pub children: Vec<AstNode>,
    pub properties: HashMap<String, serde_json::Value>,
}
```

**测试:**
- 单元测试：AST 节点类型验证
- 集成测试：解析示例 TypeScript 文件

**依赖:** Phase 01
**估计时间:** 3 天

---

### Phase 03: 语义分析层
**状态:** 🟡 Gap Closure (8 gap plans created, awaiting execution)
**目标:** 实现基础 IR 构建

**Waves Progress:**
- [x] Wave 0: Test Infrastructure (03-00)
- [x] Wave 1: Core Data Structures (03-01a)
- [x] Wave 1: Core Data Structures (03-02a)
- [x] Wave 2: IR & CFG Construction (03-03a)
- [x] Wave 2: Analysis Implementations (03-02b)
- [x] Wave 3: Main Analyzer & Integration (03-03b)

**Plans:** 7/7 implementation plans complete, 8/8 gap closure plans created
- [x] 03-00-PLAN.md — Test infrastructure and skeleton files (8c7096b)
- [x] 03-01a-PLAN.md — Scope and Symbol data structures (7820cc5)
- [x] 03-01b-PLAN.md — Symbol Table and Scope Analyzer (e49917f)
- [x] 03-02a-PLAN.md — Type representation and interner
- [x] 03-02b-PLAN.md — Type compatibility checking and resolution (8c6c17b)
- [x] 03-03a-PLAN.md — IR and CFG implementation (5a0f3d1)
- [x] 03-03b-PLAN.md — Main analyzer coordinator and integration (220fa29)

**Gap Closure Plans (Awaiting Execution):**
- [x] 03-GAP-01-PLAN.md — Type Unification Implementation (Complete - 31bf788)
- [x] 03-GAP-02-PLAN.md — Type Assignability Check Implementation (Complete - b2c901e)
- [ ] 03-GAP-03-PLAN.md — CFG Integration into Main Analyzer
- [ ] 03-GAP-04-PLAN.md — Function Parameter Handling
- [ ] 03-GAP-05-PLAN.md — Exception Parameter Handling
- [ ] 03-GAP-06-PLAN.md — Class Type Information Extraction
- [ ] 03-GAP-07-PLAN.md — Generic Type Variance Support
- [ ] 03-GAP-08-PLAN.md — Type Resolution Error Collection

**交付物:**
- `src/semantic/mod.rs` - 模块接口
- `src/semantic/analyzer.rs` - 主分析器
- `src/semantic/scope/analyzer.rs` - 作用域分析器
- `src/semantic/scope/scope.rs` - 作用域数据结构
- `src/semantic/symbol/mod.rs` - 符号表
- `src/semantic/symbol/table.rs` - 符号表实现
- `src/semantic/types/mod.rs` - 类型系统
- `src/semantic/types/interner.rs` - 类型 Interner
- `src/semantic/types/unify.rs` - 类型统一和兼容性检查
- `src/semantic/flow/mod.rs` - 控制流
- `src/semantic/flow/cfg.rs` - 控制流图
- `src/semantic/flow/builder.rs` - CFG 构建器
- `src/semantic/ir/mod.rs` - 中间表示

**关键类型:**
```rust
pub struct SemanticModule {
    pub name: String,
    pub functions: Vec<Function>,
    pub types: HashMap<String, Type>,
}

pub struct Function {
    pub id: NodeId,
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Expr,
}

pub enum Type {
    Primitive(PrimitiveType),
    Struct(StructType),
    Union(Vec<Type>),
    Option(Box<Type>),
    Function(Box<FunctionType>),
}
```

**测试:**
- 作用域嵌套测试
- 类型推断测试
- 控制流分析测试

**依赖:** Phase 02
**估计时间:** 5 天

---

### Phase 04: 语义重构核心层
**状态:** 🟡 规划完成 (3/3 waves planned)
**目标:** 实现 RRA、OA、SR 和部分推导算法

**Waves Progress:**
- [ ] Wave 1: AnalysisContext and Analyzer Traits (04-01-PLAN.md) - 2 days
- [ ] Wave 2: RRA and Dataflow Framework (04-02-PLAN.md) - 3 days
- [ ] Wave 3: OA, SR, and Derivation Algorithms (04-03-PLAN.md) - 3 days

**Plans:** 3/3 waves planned
- [x] 04-01-PLAN.md — AnalysisContext, SemanticAnalyzer trait, module structure
- [x] 04-02-PLAN.md — Reference graph, dataflow framework, RRA analyzer
- [x] 04-03-PLAN.md — Ownership annotator, semantic rewriter, derivation algorithms

**交付物:**
- `src/semantic/refactor/mod.rs` - 模块接口
- `src/semantic/refactor/context/mod.rs` - 分析上下文
- `src/semantic/refactor/context/context.rs` - AnalysisContext
- `src/semantic/refactor/traits/mod.rs` - 算法接口
- `src/semantic/refactor/traits/analyzer.rs` - SemanticAnalyzer trait
- `src/semantic/refactor/dataflow/mod.rs` - 数据流框架
- `src/semantic/refactor/dataflow/framework.rs` - DataflowEngine, Analysis trait
- `src/semantic/refactor/rra/mod.rs` - 引用关系分析器
- `src/semantic/refactor/rra/graph.rs` - ReferenceGraph, RefType, UsageInfo
- `src/semantic/refactor/rra/analyzer.rs` - RRA 实现
- `src/semantic/refactor/rra/summary.rs` - Inter-procedural summaries
- `src/semantic/refactor/oa/mod.rs` - 所有权标注器
- `src/semantic/refactor/oa/categories.rs` - OwnershipCategory, OwnershipAnnotation
- `src/semantic/refactor/oa/annotator.rs` - OA 实现
- `src/semantic/refactor/sr/mod.rs` - 语义重写器
- `src/semantic/refactor/sr/rewriter.rs` - Rewriter trait, SemanticRewriter
- `src/semantic/refactor/sr/rules.rs` - RewriteRule, RuleRegistry
- `src/semantic/refactor/sr/history.rs` - TransformRecord, transformation history
- `src/semantic/refactor/derive/mod.rs` - 推导算法
- `src/semantic/refactor/derive/ownership.rs` - 所有权推导
- `src/semantic/refactor/derive/lifetime.rs` - 生命周期推导
- `src/semantic/refactor/derive/mutability.rs` - 可变性推导

**核心 Trait:**
```rust
pub trait AnalysisResult: Clone + Debug + Send + Sync {}

pub trait SemanticAnalyzer<R: AnalysisResult> {
    fn analyze(&self, ctx: &mut AnalysisContext) -> Result<()>;
    fn name(&self) -> &'static str;
}

pub struct AnalysisContext {
    pub module: SemanticModule,
    pub results: FxHashMap<(NodeId, TypeId), Box<dyn AnalysisResult>>,
    pub ir_version: u64,
    pub cache_enabled: bool,
}
```

**测试:**
- AnalysisContext 和 analyzer traits 测试
- Reference graph 和 dataflow framework 测试
- RRA analyzer 和 summaries 测试
- Ownership annotator 测试
- Semantic rewriter 测试
- Derivation algorithms 测试

**依赖:** Phase 03
**估计时间:** 8 天

---

### Phase 05: Rust AST层
**状态:** 🔴 未开始
**目标:** 实现 Rust 语义表示

**交付物:**
- `src/ast/mod.rs` - 模块接口
- `src/ast/types.rs` - Rust 类型
- `src/ast/items.rs` - 顶层项
- `src/ast/expr.rs` - 表达式
- `src/ast/stmt.rs` - 语句
- `src/ast/pattern.rs` - 模式
- `src/ast/attributes.rs` - 属性

**关键类型:**
```rust
pub struct RustFile {
    pub items: Vec<Item>,
    pub attributes: Vec<Attribute>,
}

pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Impl(Impl),
    Use(Use),
}

pub enum Type {
    Simple(SimpleType),
    Ref(bool, Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, usize),
    Tuple(Vec<Type>),
    Path(String, Vec<Type>),
}
```

**测试:**
- AST 构建测试
- 类型转换测试

**依赖:** Phase 04
**估计时间:** 3 天

---

### Phase 06: 代码生成层
**状态:** 🔴 未开始
**目标:** 实现代码渲染

**交付物:**
- `src/codegen/mod.rs` - 模块接口
- `src/codegen/renderer.rs` - 代码渲染器
- `src/codegen/formatter.rs` - 代码格式化
- `src/codegen/optimizer.rs` - AST 优化

**关键功能:**
```rust
pub struct Renderer {
    // 渲染配置
}

impl Renderer {
    pub fn render_file(&self, file: &RustFile) -> String;
    pub fn render_item(&self, item: &Item) -> String;
    pub fn render_expr(&self, expr: &Expr) -> String;
}
```

**测试:**
- 代码渲染测试
- 格式化测试

**依赖:** Phase 05
**估计时间:** 3 天

---

### Phase 07: 集成测试
**状态:** 🔴 未开始
**目标:** 集成所有层，验证端到端流程

**交付物:**
- 完善 `src/lib.rs` 的完整转换管道
- 集成测试套件

**验证计划:**
- 使用现有测试用例 `tests/fixtures/original/` (60+ 个测试用例)
- 验证编译通过率 >= 95%
- 性能测试：单个文件转换 < 0.5 秒

**关键功能:**
```rust
pub fn transmute(source: &str) -> Result<String> {
    // 1. Parse TypeScript
    // 2. Analyze semantics
    // 3. Run refactoring algorithms
    // 4. Build Rust AST
    // 5. Generate Rust code
}
```

**测试:**
- 端到端转换测试
- 性能基准测试
- 内存使用测试

**依赖:** Phase 06
**估计时间:** 5 天

---

## 未来里程碑

### v0.2.0 - 语义增强
**预计开始日期:** Phase 07 完成后
**预计持续时间:** 3 周

**目标:**
- 实现剩余的 12 个推导算法
- 完善所有权标注
- 完善生命周期推导

### v0.3.0 - 高级特性
**预计开始日期:** v0.2.0 完成后
**预计持续时间:** 4 周

**目标:**
- 异步模型转换
- 泛型支持
- 高级类型系统
- 闭包转换

### v1.0.0 - 稳定发布
**预计开始日期:** v0.3.0 完成后
**预计持续时间:** 4 周

**目标:**
- 完整 TypeScript 支持
- 生产级性能
- 完整文档和示例
- 发布到 crates.io

## 进度跟踪

### 总体进度
- 完成: 2/7 阶段 (29%)
- 预计总时间: 27 天

### 当前阶段
- 当前阶段: Phase 03 - 语义分析层
- 阶段状态: 🟡 规划完成
- 下一阶段目标: 实现基础 IR 构建
