# Phase 02: 解析层 - Context

**Gathered:** 2026-02-28
**Status:** Ready for planning

<domain>
## Phase Boundary

实现基础解析层，支持 Deno 后端。将 TypeScript 源代码转换为统一的 AST-IR 表示，供后续语义分析、重构和代码生成阶段使用。

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<specifics>
## Specific Ideas

- 使用 Arena allocator 提高大文件解析的内存效率
- Builder pattern 确保 AST 节点在构建后不可变
- Visitor pattern 便于后续遍历和转换操作
- 零基索引符合 Rust 惯例，显示时转换为人类可读的 1 基索引
- 错误处理使用三层策略：结构化定义 + 上下文包装 + 友好展示

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-parser*
*Context gathered: 2026-02-28*
