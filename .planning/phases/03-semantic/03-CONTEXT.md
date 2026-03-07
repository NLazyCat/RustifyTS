# Phase 03: 语义分析层 - Context

**Gathered:** 2026-02-28
**Status:** Ready for planning

<domain>
## Phase Boundary

实现基础 IR 构建，包括作用域分析、符号表、类型系统和控制流分析。将解析的 AST 转换为语义 IR，供下游重构和代码生成使用。

</domain>

<decisions>
## Implementation Decisions

### Type system design
- **类型表示:** Interned types - 不可变类型节点，内部引用以支持共享
- **类型推断:** No inference (TS types) - 直接使用 TypeScript 类型，IR 中不需要推断
- **类型统一:** TS compatibility rules - 使用 TypeScript 兼容性规则
- **泛型支持:** Full generic support - 完整支持泛型，包括类型参数和替换

### Scope analysis rules
- **作用域规则:** ES6 standard - 函数、块、循环、catch 块，遵循 ES6 语义
- **块级作用域:** Always block scope - 块语句创建新作用域（let/const）
- **变量提升:** Function + var hoisting - 提升函数声明和 var 声明
- **动态作用域:** Lexical + edge cases - 词法作用域，特殊情况特殊处理

### Symbol table design
- **符号存储:** Hash-based with parent links - 每个作用域使用 HashMap<Name, Symbol>，链接到父作用域
- **名称解析:** Lexical search chain - 沿作用域链向上查找最近的匹配符号
- **标识符遮蔽:** Allow shadowing - 内部作用域遮蔽外部作用域（标准 TS 行为）
- **可见性跟踪:** Export tracking only - 只跟踪导出 vs 内部

### Control flow representation
- **CFG 粒度:** Basic blocks - 基本块，带有终止符（br, condbr, ret 等）
- **支配关系:** Immediate dominators - 仅计算直接支配者（IDoms）
- **边表示:** Labeled condition edges - true/false 边带条件表达式
- **CFG 范围:** Multi-level CFG - 每个作用域一个 CFG（块、函数、模块）

### Claude's Discretion
- Interned types 的具体实现细节
- 作用域链的遍历优化
- 符号类型的具体分类方式
- CFG 节点的具体类型定义
- 边缘情况处理

</decisions>

<specifics>
## Specific Ideas

- 使用 Interned types 避免重复类型对象，提高内存效率
- ES6 作用域规则确保与 TypeScript 行为一致
- 词法搜索链是标准且易于理解的方式
- 多层 CFG 允许不同粒度的控制流分析
- 基本块粒度平衡了复杂度和分析能力

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-semantic*
*Context gathered: 2026-02-28*
