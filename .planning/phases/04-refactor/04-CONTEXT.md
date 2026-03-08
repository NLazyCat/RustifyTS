# Phase 04: 语义重构核心层 - Context

**Gathered:** 2026-03-01
**Status:** Ready for planning

<domain>
## Phase Boundary

实现 RRA、OA、SR 和部分推导算法。包括引用关系分析器、所有权标注器、语义重写器和第一批算法（所有权推导、生命周期推导、可变性推导）。

</domain>

<decisions>
## Implementation Decisions

### RRA (Reference Relation Analysis) design
- **引用关系表示:** Adjacency list - 边列表（from_id, to_id, type）表示稀疏图
- **引用类型跟踪:** Typed edges -借入、拥有、读取、写入等作为枚举类型
- **分析范围:** Inter-procedural with summaries - 跨函数调用分析，使用摘要数据
- **引用使用跟踪:** Full usage tracking - 同时跟踪使用计数和活跃区间

### OA (Ownership Annotator) design
- **注解存储位置:** Inline in IR nodes - 直接附加到 IR 节点上
- **所有权确定:** Mixed approach - 自动推断 + 手动提示的混合方式
- **冲突处理:** Track ambiguity - 标记为模糊供后续解析
- **注解格式:** Rust-style categories - Owned, Ref, RefMut 等 Rust 风格分类

### SR (Semantic Rewriter) design
- **IR 修改方式:** Immutable copy - 复制 IR 并应用转换
- **重写规则应用:** Fixed-point iteration - 迭代直到固定点
- **重写规则结构:** Visitor-based - 基于 IR 的访问者模式
- **转换跟踪:** Full history - 跟踪所有转换用于调试/回滚

### Derivation algorithms design
- **算法顺序:** Sequential pipeline - RRA → OA → 推导按顺序执行
- **结果存储:** Context results map - 存储在 AnalysisContext results map 中，键为 (NodeId, TypeId)
- **增量分析:** Full incremental - IR 变化时支持增量更新
- **缓存策略:** Lazy cache - 懒缓存，IR 变化时失效

### Claude's Discretion
- 邻接表的具体数据结构选择
- 类型边枚举的详细定义
- 跨过程摘要的具体格式
- 活跃区间分析的具体算法
- 固定点迭代的收敛条件
- 转换历史的持久化策略

</decisions>

<specifics>
## Specific Ideas

- 邻接表适合稀疏引用关系图，节省内存
- 内联注解简化访问，避免额外查找
- 混合所有权确定平衡了自动化和灵活性
- 不可变 IR 复制确保可追溯和可回滚
- 访问者模式便于扩展新的重写规则
- 固定点迭代确保转换稳定
- 完整历史支持调试和诊断问题

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-refactor*
*Context gathered: 2026-03-01*
