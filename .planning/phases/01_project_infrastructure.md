# Phase 01: 项目基础设施

## 目标

搭建 GSD 结构，基础项目骨架。

## 实施计划

### 任务清单

- [ ] 创建 `.planning/` 目录结构
- [ ] 创建 `.planning/PROJECT.md` - 项目愿景
- [ ] 创建 `.planning/ROADMAP.md` - 路线图
- [ ] 创建 `.planning/research/SUMMARY.md` - 研究总结占位符
- [ ] 创建/更新 `HELP.md` - 项目帮助
- [ ] 创建/更新 `FIX.md` - 待修复列表
- [ ] 创建/更新 `FIXED.md` - 已完成列表
- [ ] 更新 `Cargo.toml` - 依赖配置
- [ ] 更新 `README.md` - 项目说明
- [ ] 创建新 `src/lib.rs` - 基础库入口

### 实施步骤

#### 1. 创建 .planning 目录结构

```bash
mkdir -p .planning/research
mkdir -p .planning/phases
```

#### 2. 创建 PROJECT.md

包含项目愿景、目标、架构设计、15个推导算法说明。

#### 3. 创建 ROADMAP.md

包含 7 个阶段的详细规划和里程碑。

#### 4. 更新 HELP.md

添加新架构的工具使用指南。

#### 5. 更新 Cargo.toml

使用最新版本的依赖：
- `serde` = "1.0.228"
- `serde_json` = "1.0.149"
- `thiserror` = "2.0.18"
- `anyhow` = "1.0.102"
- `clap` = "4.5.60"
- `tokio` = "1.49.0"

#### 6. 更新 README.md

添加新架构说明和快速开始指南。

#### 7. 创建新 src/lib.rs

基础库入口，预留模块接口：

```rust
//! # Transmute
//!
//! TypeScript 到 Rust 语义转换器

pub mod parser;
pub mod semantic;
pub mod refactor;
pub mod ast;
pub mod codegen;

use anyhow::Result;

/// TypeScript 到 Rust 的主转换函数
pub fn transmute(source: &str) -> Result<String> {
    // TODO: 实现 5 层转换管道
    Ok(String::new())
}

/// 文件到文件的转换函数
pub fn transmute_file(input: &std::path::Path) -> Result<String> {
    // TODO: 读取文件并转换
    Ok(String::new())
}
```

## 验收标准

- [ ] `.planning/PROJECT.md` 文件存在且内容完整
- [ ] `.planning/ROADMAP.md` 文件存在且内容完整
- [ ] `HELP.md` 包含新架构说明
- [ ] `FIX.md` 重置为空或包含待开始项
- [ ] `FIXED.md` 重置为空
- [ ] `Cargo.toml` 使用最新依赖版本
- [ ] `README.md` 包含新架构说明
- [ ] `src/lib.rs` 包含基础模块接口
- [ ] `cargo check` 通过编译

## 依赖

无前置依赖

## 估计时间

1 天

## 风险

无重大风险

## 下一步

完成后进入 Phase 02: 解析层
