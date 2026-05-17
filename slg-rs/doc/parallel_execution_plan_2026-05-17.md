# Step 13+ 并行执行文档

> 日期：2026-05-17
> 当前基线：Step 13 World 核心系统进行中，采集返回回写基线已完成。
> 目的：把后续大功能切成可多 agent / 多 worktree 并行推进的工作流，同时避免核心文件互相覆盖。

## 0. 先固化当前基线

当前主 worktree 还有未提交代码变更，并且已有一组文档归档变更处于 staged 状态。不要直接从当前 `master` 派多个 worktree 开干，否则新 worktree 会从旧 HEAD 出发，拿不到采集返回回写基线。

建议先在主 worktree 完成两个基线提交：第一个提交当前 staged docs，第二个提交 Step 13 采集返回代码基线。

```bash
rtk git -c core.quotepath=false status --short --branch
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p world -p home@0.1.0 -p proto

rtk git -c core.quotepath=false switch -c integration/step13-collect-return-base
rtk git -c core.quotepath=false commit -m "docs: refresh step 13 plan"

rtk git -c core.quotepath=false add \
  crates/home/src/actors/player_actor.rs \
  crates/home/src/systems/backpack.rs \
  crates/home/src/systems/hero.rs \
  crates/proto/proto/service.proto \
  crates/world/src/collect.rs \
  crates/world/src/main.rs \
  crates/world/src/march/mod.rs \
  crates/world/src/message.rs \
  crates/world/src/outbound.rs \
  crates/world/src/runtime.rs \
  crates/world/src/sector_actor.rs \
  crates/world/src/service.rs \
  crates/world/src/wal.rs
rtk git -c core.quotepath=false commit -m "step13 collect return outbound baseline"
```

之后所有并行 worktree 都从 `integration/step13-collect-return-base` 分支切出，最后统一合回这个集成分支。

## 1. 并行原则

能并行：

- 新模块、新目录、新系统骨架：冲突少，适合独立 worktree。
- 只读探索、测试补充、文档补充：可随时并行。
- `shared/src/battle/*` 这类新战斗引擎内部实现：可以和 World/Home 业务并行。
- `world/src/map/*` 地图刷新/生命周期：大多可以独立于 Home 和战斗引擎推进。

不要并行改同一批热点文件：

- `crates/proto/proto/service.proto`
- `crates/world/src/sector_actor.rs`
- `crates/world/src/outbound.rs`
- `crates/world/src/service.rs`
- `crates/home/src/actors/player_actor.rs`
- `crates/world/src/wal.rs`

这些文件每次只给一个 owner。其他 agent 如需改接口，先在自己的文档/草案里写清楚，由当前 owner 集成。

## 2. Worktree 模板

```bash
mkdir -p /home/matt/dev/javaCode/OnePunch-worktrees

rtk git -c core.quotepath=false worktree add \
  /home/matt/dev/javaCode/OnePunch-worktrees/slg-outbound-idempotency \
  -b agent/step13-outbound-idempotency integration/step13-collect-return-base

rtk git -c core.quotepath=false worktree add \
  /home/matt/dev/javaCode/OnePunch-worktrees/slg-map-lifecycle \
  -b agent/step13-world-map-lifecycle integration/step13-collect-return-base

rtk git -c core.quotepath=false worktree add \
  /home/matt/dev/javaCode/OnePunch-worktrees/slg-battle-engine \
  -b agent/step14-battle-engine integration/step13-collect-return-base
```

每个 worktree 的根目录就是一份 `slg-rs` checkout。每个 worktree 内都先跑：

```bash
rtk git -c core.quotepath=false status --short --branch
rtk cargo test --manifest-path Cargo.toml -p world -p home@0.1.0 -p proto
```

## 3. 推荐执行波次

### Wave 1：Step 13 基础收口

这波优先做，因为它会影响后续侦查、战斗、邮件等所有回写链路。

| Lane | 是否可并行 | Owner 文件 | 目标 | 验收 |
|------|------------|------------|------|------|
| A. Outbound 幂等投递 | 单 owner | `service.proto`, `world/src/outbound.rs`, `world/src/main.rs`, `world/src/wal.rs`, `home/src/actors/player_actor.rs`, `shared/src/persistence.rs` | 给 WorldOutbound 增加 event_id / event_key，Home 侧做已处理记录或最小内存去重，World 侧保留可重试状态 | 重复投递 `CollectReturned` 不重复加资源；重启后不会丢待投递事件 |
| B. 采集生产化 | 可在 A 后独立 | `world/src/collect.rs`, `world/src/sector_actor.rs`, `shared/src/static_config/world.rs` | 用矿点配置和产量公式替换固定默认奖励，补容量/时间/资源类型 | 采集不同矿点返回不同资源和数量；配置缺失有明确错误 |
| C. 地图实体生命周期 | 可并行 | `world/src/map/lifecycle.rs`, `world/src/service.rs`, `world/src/runtime.rs`, `world/src/wal.rs` | 地图初始化、刷新、过期、持久化从最小实现推进到配置驱动 | refresh 测试覆盖 expired/spawned/WAL 恢复 |
| D. World 查询补全 | 可并行但避开 `service.rs` 热点 | `world/src/service.rs`, `world/src/map/*` | 把兼容空响应逐个替换成真实数据 | 每个命令有服务层测试 |

建议顺序：A 先做，B/C/D 可在 A 接口稳定后并行。B 和 C 都可能碰 `wal.rs`，如果同时做，`wal.rs` 归 A 或 C 单 owner。

### Wave 2：侦查报告链路

侦查跨 World、Home、Mail，拆成两个 worktree 比较稳。

| Lane | 是否可并行 | Owner 文件 | 目标 | 依赖 |
|------|------------|------------|------|------|
| E. MailSystem 最小骨架 | 可并行 | 新增 `home/src/systems/mail.rs`，小改 `systems/mod.rs`, `player_actor.rs` | 支持 `mail_func` load/save、邮件列表、读邮件、删除/锁定最小命令 | 无，尽量不碰 World |
| F. ScoutReport 生成 | E 后集成 | `service.proto`, `world/src/outbound.rs`, `sector_actor.rs`, `home/src/actors/player_actor.rs` | `ScoutReportRequested` 扩展为真实 `ScoutReport` payload，Home 写入 MailSystem | A 和 E |

不要让 E/F 同时大改 `player_actor.rs`。E 先把 MailSystem 接进去，F 只调用 MailSystem 的公开方法。

### Wave 3：战斗入口和战斗引擎

战斗可以拆成“引擎内部”和“World 接线”两段，前者适合并行，后者必须串行集成。

| Lane | 是否可并行 | Owner 文件 | 目标 | 依赖 |
|------|------------|------------|------|------|
| G. Battle engine skeleton | 可并行 | 新增 `shared/src/battle/*` 或独立 `crates/battle`，`shared/src/static_config/battle.rs` | 定义 Fighter/Skill/Round/Report/Result，先做确定性小战斗 | 无 |
| H. Battle report/mail | 可并行于 G 后半段 | `home/src/systems/mail.rs`, `proto/Mail.proto` 已有类型 | 能存战报邮件、读战报、清新状态 | E |
| I. World battle integration | 不可并行 | `world/src/service.rs`, `sector_actor.rs`, `outbound.rs`, `runtime.rs` | `DeclareFight` / `JoinTheFight` 从占位错误变成创建/加入战斗，战斗结束生成报告和返回部队 | A, G, H |

G 的验收不要依赖 World：只跑 `shared` 或 battle crate 单测。I 的验收必须跑 World + Home + proto。

### Wave 4：Step 15 运营系统

运营系统多数在 Home，容易同时抢 `player_actor.rs` 分发入口。正确做法是先一个 agent 建注册模式，再让多个 agent 写各自系统。

| Lane | 是否可并行 | Owner 文件 | 目标 |
|------|------------|------------|------|
| J. Home 系统注册整理 | 单 owner | `home/src/actors/player_actor.rs`, `home/src/systems/mod.rs` | 降低新增系统时对 PlayerActor 的改动面 |
| K. Shop/VIP | J 后可并行 | `home/src/systems/shop.rs`, `home/src/systems/vip.rs` | 商品购买、VIP 经验/等级 |
| L. Chat | J 后可并行 | 新增 chat actor/system，Gateway 路由可能要接 | 私聊/频道/系统消息最小闭环 |
| M. Rank/GM | J 后可并行 | shared persistence/global data, home service/admin path | 排行榜快照和 GM 命令基础 |

## 4. 推荐 agent 分工

一次最多开 3-4 个实现 agent，超过这个数量集成成本会反超收益。

第一轮建议：

1. Agent A：Outbound 幂等投递，单独 worktree，拥有 proto/outbound/Home writeback 热点。
2. Agent B：地图实体生命周期，独立 worktree，只碰 `world/src/map/*` 和必要 service 测试。
3. Agent C：Battle engine skeleton，独立 worktree，新建 battle 模块，不碰 World 接线。
4. Agent D：MailSystem 最小骨架，独立 worktree，但先约定 `player_actor.rs` 的改动窗口。

第二轮建议：

1. Agent E：采集生产化，基于 A 合入后的分支。
2. Agent F：ScoutReport 生成，基于 A + D 合入后的分支。
3. Agent G：World battle integration，基于 A + C + D/H 合入后的分支。

## 5. Agent 提示词模板

实现型 agent：

```text
在 /home/matt/dev/javaCode/OnePunch/slg-rs 的当前 worktree 工作。
只负责 <lane name>。
你不是唯一 agent，不要回退别人改动。
写权限限定在：<file/module list>。
遵守 RTK：shell 命令必须以 rtk 开头。
实现完成后运行：<test commands>。
最终回答列出改动文件、测试结果、剩余风险。
```

探索型 agent：

```text
只读分析 <domain>，不要编辑文件。
输出：任务切分、文件归属、依赖、风险、建议测试。
不要重复分析其他 agent 已覆盖的领域。
```

## 6. 集成规则

每个 worktree 完成后：

```bash
rtk git -c core.quotepath=false status --short --branch
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p world -p home@0.1.0 -p proto
rtk git -c core.quotepath=false add -A
rtk git -c core.quotepath=false commit -m "<lane summary>"
```

回到 `integration/step13-collect-return-base` 集成：

```bash
rtk git -c core.quotepath=false switch integration/step13-collect-return-base
rtk git -c core.quotepath=false merge --no-ff agent/<lane-branch>
rtk cargo test --manifest-path Cargo.toml -p world -p home@0.1.0 -p proto
```

如果合并冲突发生在热点文件，只让该 lane owner 处理，不要让多个 agent 同时修同一个冲突。

## 7. 下一步选择

如果目标是最快把 Step 13 收口，下一步做：

1. Outbound 幂等投递
2. 地图实体生命周期
3. 采集生产化
4. 侦查报告 + MailSystem

如果目标是尽快启动 Step 14，下一步并行做：

1. Battle engine skeleton
2. MailSystem 最小骨架
3. Outbound 幂等投递

真正的 World battle integration 必须等这三项至少有可用接口后再做。
