---
name: git-commit-message-from-staged
description: 从 staged changes 生成符合仓库规范的 git commit message（slg-rs Rust 工程）
---

# 从 staged changes 生成 commit message

> 适用：slg-rs Rust 工程，不依赖额外工具，照步骤操作即可。

## 前置条件
- 你已经把要提交的改动 `git add` 到暂存区（staged changes）
- 当前仓库是 `slg-rs`

## Step 1：确认暂存区
运行：
```bash
git diff --cached --name-only
```
如果输出为空，先 `git add` 后再继续。

## Step 2：查看暂存内容摘要（用于确定 scope/type）
运行：
```bash
git diff --cached --stat
```

## Step 3：基于暂存内容写 commit message 草稿
按仓库规则生成：

1. 推荐格式（必选）：
`<type>(<scope>): <subject>`

2. `type` 选择建议：
- `feat`：新增功能/新增模块/新增协议处理
- `fix`：Bug 修复
- `chore`：构建/工具/工程性改动/Cargo.toml 依赖变更
- `refactor`：纯重构（不改变行为）
- `perf`：性能优化
- `docs`：文档/注释
- `style`：格式化（不影响逻辑，如 `cargo fmt`）
- `test`：测试相关

3. `<scope>` 建议（对应 crates/ 下的模块）：
- `auth` - 认证相关
- `gateway` - 网关
- `home` - 主城/家园
- `world` - 世界地图
- `proto` - 协议层
- `shared` - 公共库/工具
- `workspace` - 工程级改动（Cargo.toml/workspace）
- 不确定就用最相关的 crate 名

4. `<subject>` 建议：
- 中文
- 动词开头
- 不超过 50 字
- 不要句号结尾

## Step 4（可选）：写 body（最多 3 条）
body 用 `- ` bullet，说明"为什么/带来什么效果"，不要复刻文件名列表。

## Step 5：最终执行提交（如果你确认无误）
```bash
git commit -m "<type>(<scope>): <subject>"
```
