# Phase 04: 语义重构核心层

## 目标

实现 RRA、OA、SR 和前 3 个推导算法（所有权、生命周期、可变性）。

## 实施计划

### 任务清单

- [ ] 创建 `src/refactor/mod.rs` - 模块接口
- [ ] 创建 `src/refactor/context/mod.rs` - 分析上下文模块
- [ ] 创建 `src/refactor/context/context.rs` - AnalysisContext
- [ ] 创建 `src/refactor/rra/mod.rs` - 引用关系分析器模块
- [ ] 创建 `src/refactor/rra/analyzer.rs` - RRA 实现
- [ ] 创建 `src/refactor/oa/mod.rs` - 所有权标注器模块
- [ ] 创建 `src/refactor/oa/annotator.rs` - OA 实现
- [ ] 创建 `src/refactor/sr/mod.rs` - 语义重写器模块
- [ ] 创建 `src/refactor/sr/rewriter.rs` - SR 实现
- [ ] 创建 `src/refactor/traits/mod.rs` - 算法接口模块
- [ ] 创建 `src/refactor/traits/analyzer.rs` - SemanticAnalyzer trait
- [ ] 创建 `src/refactor/algorithms/01_ownership.rs` - 所有权推导
- [ ] 创建 `src/refactor/algorithms/02_lifetime.rs` - 生命周期推导
- [ ] 创建 `src/refactor/algorithms/03_mutability.rs` - 可变性推导

### 关键设计

#### 分析上下文

```rust
use crate::semantic::{SemanticModule, Type, Expr};
use std::collections::HashMap;

/// 分析上下文，包含所有分析结果
pub struct AnalysisContext {
    /// 语义模块
    pub module: SemanticModule,

    /// 引用关系分析结果
    pub references: ReferenceAnalysis,

    /// 所有分析算法的结果
    pub results: HashMap<(NodeId, TypeId), Box<dyn AnalysisResult>>,
}

impl AnalysisContext {
    pub fn new(module: SemanticModule) -> Self {
        Self {
            module,
            references: ReferenceAnalysis::new(),
            results: HashMap::new(),
        }
    }
}

/// 引用关系分析结果
#[derive(Debug, Clone)]
pub struct ReferenceAnalysis {
    /// 每个节点的引用目标
    pub references: HashMap<NodeId, Vec<NodeId>>,

    /// 每个节点被引用的次数
    pub ref_count: HashMap<NodeId, usize>,

    /// 值的借用关系
    pub borrows: HashMap<NodeId, Vec<Borrow>>,

    /// 值的移动关系
    pub moves: HashMap<NodeId, Vec<NodeId>>,
}

/// 借用信息
#[derive(Debug, Clone)]
pub struct Borrow {
    pub borrower: NodeId,
    pub kind: BorrowKind,
    pub lifetime: Lifetime,
}

/// 借用类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowKind {
    Shared,  // &T
    Mutable, // &mut T
}
```

#### 分析结果 Trait

```rust
use std::fmt::Debug;

/// 分析结果 trait
pub trait AnalysisResult: Clone + Debug + Send + Sync {}

/// 所有权分析结果
#[derive(Debug, Clone)]
pub struct OwnershipResult {
    pub kind: OwnershipKind,
    pub borrowed_by: Vec<NodeId>,
    pub moved_to: Vec<NodeId>,
}

impl AnalysisResult for OwnershipResult {}

/// 所有权类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipKind {
    Owned,      // 拥有值
    Borrowed,   // 借用值
    Moved,      // 值已移动
}

/// 生命周期分析结果
#[derive(Debug, Clone)]
pub struct LifetimeResult {
    pub lifetimes: Vec<Lifetime>,
    pub constraints: Vec<LifetimeConstraint>,
}

impl AnalysisResult for LifetimeResult {}

/// 生命周期
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime {
    pub name: String,
    pub is_static: bool,
}

/// 生命周期约束
#[derive(Debug, Clone)]
pub struct LifetimeConstraint {
    pub left: Lifetime,
    pub right: Lifetime,
    pub relation: LifetimeRelation,
}

/// 生命周期关系
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeRelation {
    Outlives,  // 'a: 'b
    Equals,    // 'a = 'b
}

/// 可变性分析结果
#[derive(Debug, Clone)]
pub struct MutabilityResult {
    pub is_mutable: bool,
    pub mutation_points: Vec<NodeId>,
}

impl AnalysisResult for MutabilityResult {}
```

#### 语义分析器 Trait

```rust
use anyhow::Result;

/// 语义分析器 trait
pub trait SemanticAnalyzer<R: AnalysisResult> {
    /// 执行分析
    fn analyze(&self, ctx: &mut AnalysisContext) -> Result<()>;

    /// 获取分析器名称
    fn name(&self) -> &'static str;

    /// 获取分析结果
    fn get_result(&self, ctx: &AnalysisContext, node_id: NodeId) -> Option<&R>;
}
```

#### 引用关系分析器 (RRA)

```rust
use super::traits::SemanticAnalyzer;
use super::context::AnalysisContext;
use super::rra::ReferenceAnalysis;
use anyhow::Result;

pub struct ReferenceRelationshipAnalyzer;

impl ReferenceRelationshipAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn analyze_expr(&self, expr: &Expr, refs: &mut ReferenceAnalysis) {
        match expr {
            Expr::Identifier(name) => {
                // 记录标识符引用
                // TODO: 查找标识符定义并记录引用
            }
            Expr::Binary { left, right, .. } => {
                self.analyze_expr(left, refs);
                self.analyze_expr(right, refs);
            }
            Expr::Call { callee, args, .. } => {
                self.analyze_expr(callee, refs);
                for arg in args {
                    self.analyze_expr(arg, refs);
                }
            }
            _ => {}
        }
    }
}

impl SemanticAnalyzer<ReferenceAnalysis> for ReferenceRelationshipAnalyzer {
    fn analyze(&self, ctx: &mut AnalysisContext) -> Result<()> {
        let mut refs = ReferenceAnalysis::new();

        // 分析所有函数
        for func in &ctx.module.functions {
            self.analyze_expr(&func.body, &mut refs);
        }

        ctx.references = refs;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ReferenceRelationshipAnalyzer"
    }

    fn get_result(&self, ctx: &AnalysisContext, _node_id: NodeId)
        -> Option<&ReferenceAnalysis>
    {
        Some(&ctx.references)
    }
}
```

#### 所有权标注器 (OA)

```rust
use super::traits::SemanticAnalyzer;
use super::context::AnalysisContext;
use super::algorithms::ownership::{OwnershipResult, OwnershipKind};
use anyhow::Result;

pub struct OwnershipAnnotator;

impl OwnershipAnnotator {
    pub fn new() -> Self {
        Self
    }

    fn analyze_expression(&self, expr: &Expr, ctx: &mut AnalysisContext) -> Result<()> {
        match expr {
            Expr::Identifier(name) => {
                // 确定标识符的所有权类型
                let ownership = self.determine_ownership(name, ctx);
                // 存储结果
            }
            Expr::Call { callee, args, .. } => {
                self.analyze_expression(callee, ctx)?;
                for arg in args {
                    // 检查参数是否需要移动
                    if self.needs_move(arg, ctx) {
                        // 标记为移动
                    }
                    self.analyze_expression(arg, ctx)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn determine_ownership(&self, _name: &str) -> OwnershipKind {
        // 简单实现：默认为拥有
        OwnershipKind::Owned
    }

    fn needs_move(&self, _expr: &Expr, _ctx: &AnalysisContext) -> bool {
        // TODO: 判断是否需要移动
        false
    }
}

impl SemanticAnalyzer<OwnershipResult> for OwnershipAnnotator {
    fn analyze(&self, ctx: &mut AnalysisContext) -> Result<()> {
        // 分析所有函数
        for func in &ctx.module.functions {
            self.analyze_expression(&func.body, ctx)?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "OwnershipAnnotator"
    }

    fn get_result(&self, _ctx: &AnalysisContext, _node_id: NodeId)
        -> Option<&OwnershipResult>
    {
        // TODO: 从上下文中获取结果
        None
    }
}
```

#### 算法 1: 所有权推导

```rust
use super::traits::{SemanticAnalyzer, AnalysisResult};
use super::context::AnalysisContext;
use anyhow::Result;

pub struct OwnershipAnalyzer;

impl OwnershipAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn analyze_node(&self, node_id: NodeId, ctx: &mut AnalysisContext) -> Result<OwnershipResult> {
        // 根据引用关系确定所有权
        let ref_count = ctx.references.ref_count.get(&node_id).copied().unwrap_or(0);
        let borrows = ctx.references.borrows.get(&node_id).cloned().unwrap_or_default();
        let moves_to = ctx.references.moves.get(&node_id).cloned().unwrap_or_default();

        let kind = if !moves_to.is_empty() {
            OwnershipKind::Moved
        } else if !borrows.is_empty() {
            OwnershipKind::Borrowed
        } else {
            OwnershipKind::Owned
        };

        let result = OwnershipResult {
            kind,
            borrowed_by: borrows.iter().map(|b| b.borrower).collect(),
            moved_to: moves_to,
        };

        // 存储结果
        ctx.results.insert((node_id, TypeId(0)), Box::new(result.clone()));

        Ok(result)
    }
}

impl SemanticAnalyzer<OwnershipResult> for OwnershipAnalyzer {
    fn analyze(&self, ctx: &mut AnalysisContext) -> Result<()> {
        // 分析所有节点
        for func in &ctx.module.functions {
            // 分析函数体中的所有节点
            self.analyze_node(func.id, ctx)?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "OwnershipAnalyzer"
    }

    fn get_result(&self, ctx: &AnalysisContext, node_id: NodeId) -> Option<&OwnershipResult> {
        ctx.results.get(&(node_id, TypeId(0)))?
            .as_any()
            .downcast_ref()
    }
}

// 让 dyn AnalysisResult 支持 downcast
trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: AnalysisResult + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

#### 算法 2: 生命周期推导

```rust
use super::traits::{SemanticAnalyzer, AnalysisResult};
use super::context::AnalysisContext;
use anyhow::Result;

pub struct LifetimeAnalyzer;

impl LifetimeAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn analyze_node(&self, node_id: NodeId, ctx: &mut AnalysisContext) -> Result<LifetimeResult> {
        // 推导生命周期约束
        let mut lifetimes = Vec::new();
        let mut constraints = Vec::new();

        // 分析借用关系
        if let Some(borrows) = ctx.references.borrows.get(&node_id) {
            for borrow in borrows {
                lifetimes.push(borrow.lifetime.clone());

                // 如果借用在函数参数中，添加到参数生命周期
                // TODO: 更复杂的生命周期分析
            }
        }

        let result = LifetimeResult {
            lifetimes,
            constraints,
        };

        ctx.results.insert((node_id, TypeId(0)), Box::new(result.clone()));

        Ok(result)
    }
}

impl SemanticAnalyzer<LifetimeResult> for LifetimeAnalyzer {
    fn analyze(&self, ctx: &mut AnalysisContext) -> Result<()> {
        for func in &ctx.module.functions {
            self.analyze_node(func.id, ctx)?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "LifetimeAnalyzer"
    }

    fn get_result(&self, ctx: &AnalysisContext, node_id: NodeId) -> Option<&LifetimeResult> {
        ctx.results.get(&(node_id, TypeId(0)))?
            .as_any()
            .downcast_ref()
    }
}
```

#### 算法 3: 可变性推导

```rust
use super::traits::{SemanticAnalyzer, AnalysisResult};
use super::context::AnalysisContext;
use anyhow::Result;

pub struct MutabilityAnalyzer;

impl MutabilityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn analyze_node(&self, node_id: NodeId, ctx: &mut AnalysisContext) -> Result<MutabilityResult> {
        // 检查变量是否被修改
        let mut mutation_points = Vec::new();
        let mut is_mutable = false;

        // 分析引用关系，查找赋值操作
        // TODO: 检测赋值表达式
        // TODO: 检测方法调用（如 push, insert 等）

        is_mutable = !mutation_points.is_empty();

        let result = MutabilityResult {
            is_mutable,
            mutation_points,
        };

        ctx.results.insert((node_id, TypeId(0)), Box::new(result.clone()));

        Ok(result)
    }
}

impl SemanticAnalyzer<MutabilityResult> for MutabilityAnalyzer {
    fn analyze(&self, ctx: &mut AnalysisContext) -> Result<()> {
        for func in &ctx.module.functions {
            self.analyze_node(func.id, ctx)?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "MutabilityAnalyzer"
    }

    fn get_result(&self, ctx: &AnalysisContext, node_id: NodeId) -> Option<&MutabilityResult> {
        ctx.results.get(&(node_id, TypeId(0)))?
            .as_any()
            .downcast_ref()
    }
}
```

### 并行执行

```rust
use rayon::prelude::*;

pub struct RefactorPipeline;

impl RefactorPipeline {
    pub fn run_all(ctx: &mut AnalysisContext) -> Result<()> {
        // 创建所有分析器
        let analyzers: Vec<Box<dyn SemanticAnalyzer<()>>> = vec![
            Box::new(refactor::rra::ReferenceRelationshipAnalyzer::new()),
            Box::new(refactor::oa::OwnershipAnnotator::new()),
            Box::new(refactor::algorithms::ownership::OwnershipAnalyzer::new()),
            Box::new(refactor::algorithms::lifetime::LifetimeAnalyzer::new()),
            Box::new(refactor::algorithms::mutability::MutabilityAnalyzer::new()),
        ];

        // 并行执行
        analyzers.par_iter().for_each(|analyzer| {
            // 注意：这里需要使用线程安全的上下文
            // 或者使用顺序执行避免数据竞争
            let _ = analyzer.analyze(ctx);
        });

        Ok(())
    }
}
```

### 测试计划

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_analyzer() {
        let module = SemanticModule::new();
        let mut ctx = AnalysisContext::new(module);

        let analyzer = OwnershipAnalyzer::new();
        analyzer.analyze(&mut ctx).unwrap();

        // 验证结果
    }

    #[test]
    fn test_lifetime_analyzer() {
        let module = SemanticModule::new();
        let mut ctx = AnalysisContext::new(module);

        let analyzer = LifetimeAnalyzer::new();
        analyzer.analyze(&mut ctx).unwrap();

        // 验证结果
    }

    #[test]
    fn test_mutability_analyzer() {
        let module = SemanticModule::new();
        let mut ctx = AnalysisContext::new(module);

        let analyzer = MutabilityAnalyzer::new();
        analyzer.analyze(&mut ctx).unwrap();

        // 验证结果
    }
}
```

## 验收标准

- [ ] `src/refactor/` 模块编译通过
- [ ] RRA 功能完整
- [ ] OA 功能完整
- [ ] SR 功能完整
- [ ] 3 个推导算法功能完整
- [ ] 可以并行执行算法
- [ ] 单元测试通过

## 依赖

Phase 03 必须完成

## 估计时间

7 天

## 风险

1. **所有权分析复杂性** - Rust 的所有权系统非常复杂
   - 缓解：先实现简单场景，逐步支持复杂情况

2. **生命周期推断** - 生命周期推断可能不准确
   - 缓解：提供生命周期标注提示

## 下一步

完成后进入 Phase 05: Rust AST 层
