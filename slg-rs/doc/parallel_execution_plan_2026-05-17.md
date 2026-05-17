# Step 13+ 并行执行文档

> 日期：2026-05-17
> 当前基线：Step 13/14 主链路已合入，Step 15 Home system registry 已合入，下一步进入 Shop/VIP、Chat、Rank/GM 等 Home 运营系统。
> 目的：把后续大功能切成可多 agent / 多 worktree 并行推进的工作流，同时避免核心文件互相覆盖。

## 0. 当前集成基线

当前可继续派生工作的基线是：

- 本地/远端分支：`integration/step13-collect-return-base`
- 当前提交：`c2f6329b merge step15 home system registry`
- 已合入：
  - World outbound idempotency
  - World map lifecycle
  - MailSystem baseline
  - Collect production settlement
  - ScoutReport mail flow
  - Battle engine core hardening
  - World battle integration
  - Home system registry
- 已验证测试：
  - `rtk cargo test --manifest-path Cargo.toml -p home@0.1.0 -p proto`
  - `rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto`

Linux 本机同步这个基线：

```bash
rtk git -c core.quotepath=false fetch origin
rtk git -c core.quotepath=false switch integration/step13-collect-return-base
rtk git -c core.quotepath=false pull --ff-only
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto
```

如果本地还没有这个分支，用：

```bash
rtk git -c core.quotepath=false fetch origin
rtk git -c core.quotepath=false switch -c integration/step13-collect-return-base origin/integration/step13-collect-return-base
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto
```

之后所有并行 worktree 都从 `integration/step13-collect-return-base` 分支切出，最后统一合回这个集成分支。

## 0.1 Mac 接管流程

Mac 上的主路径使用：

```bash
cd ~/DEV/javaCode/OnePunch/slg-rs
```

切到 Mac 后先同步最新集成分支：

```bash
rtk git -c core.quotepath=false fetch origin
rtk git -c core.quotepath=false switch integration/step13-collect-return-base
rtk git -c core.quotepath=false pull --ff-only
rtk git -c core.quotepath=false status --short --branch
rtk git -c core.quotepath=false show --no-patch --oneline --decorate HEAD
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto
```

Mac worktree 根目录建议使用：

```bash
rtk mkdir -p ~/DEV/javaCode/OnePunch-worktrees
```

新任务统一从最新 integration 派生：

```bash
rtk git -c core.quotepath=false worktree add \
  ~/DEV/javaCode/OnePunch-worktrees/<worktree-name> \
  -b agent/<branch-name> origin/integration/step13-collect-return-base
```

Mac 成为主工作机后，当前 Linux 窗口只作为参考，不再同时修改热点文件。每个 agent 完成后必须 push 自己的 `agent/*` 分支，再由主工作机合入 `integration/step13-collect-return-base`。

集成分支合并规则：

```bash
rtk git -c core.quotepath=false fetch origin
rtk git -c core.quotepath=false switch integration/step13-collect-return-base
rtk git -c core.quotepath=false pull --ff-only
rtk git -c core.quotepath=false merge --no-ff origin/agent/<branch-name>
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto
rtk git -c core.quotepath=false push origin integration/step13-collect-return-base
```

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
rtk mkdir -p /home/matt/dev/javaCode/OnePunch-worktrees

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

## 7. 当前待办队列

已完成：

1. Step 13 World outbound/idempotency
2. Step 13 world map lifecycle
3. Step 13 MailSystem baseline
4. Step 13 collect production settlement
5. Step 13 ScoutReport mail flow
6. Step 14 shared battle engine core
7. Step 14 World battle integration
8. Step 15 Home system registry

当前建议顺序：

1. `agent/step15-shop-vip`：Shop/VIP 最小闭环，优先做。
2. `agent/step15-chat`：聊天系统，Shop/VIP 完成后可并行。
3. `agent/step15-rank-gm`：排行/GM 基线，Shop/VIP 完成后可并行。
4. `agent/step15-world-query-completion`：World 查询补全，避开 Home 热点文件时可并行。
5. `agent/step15-battle-report-polish`：战报展示和邮件字段补强，只在没有其他 agent 修改 `mail.rs`/`player_actor.rs` 时做。

当前热点文件：

- `crates/home/src/actors/player_actor.rs`
- `crates/home/src/systems/registry.rs`
- `crates/home/src/systems/mod.rs`
- `crates/proto/proto/service.proto`
- `crates/world/src/outbound.rs`
- `crates/world/src/sector_actor.rs`

同一时间只允许一个实现型 agent 修改这些文件。其他 agent 可以做只读分析或写不冲突的新模块。

## 8. 下一轮自治任务：Shop/VIP

Mac 上开新对话后，把下面整段给 agent：

```text
你在 ~/DEV/javaCode/OnePunch/slg-rs 工作。先同步最新 integration，并创建独立 worktree：

rtk git -c core.quotepath=false fetch origin
rtk git -c core.quotepath=false switch integration/step13-collect-return-base
rtk git -c core.quotepath=false pull --ff-only

rtk mkdir -p ~/DEV/javaCode/OnePunch-worktrees
rtk git -c core.quotepath=false worktree add \
  ~/DEV/javaCode/OnePunch-worktrees/slg-shop-vip \
  -b agent/step15-shop-vip origin/integration/step13-collect-return-base

cd ~/DEV/javaCode/OnePunch-worktrees/slg-shop-vip

任务：实现 Step 15 Shop/VIP 最小闭环。

背景：
- 最新基线是 origin/integration/step13-collect-return-base。
- 已合入 Home system registry：crates/home/src/systems/registry.rs。
- 新 Home 系统命令应该通过 registry 接入，尽量减少 player_actor.rs 改动。
- 已合入 MailSystem、ScoutReport、World battle integration。

目标：
1. 新增或补全 ShopSystem 和 VipSystem。
2. 支持最小商品购买流程：校验配置、扣资源/道具、发奖励。
3. 支持 VIP 经验/等级基础数据：load/save、登录下发、基础查询。
4. 通过 Home registry 注册命令路由，不要把大量 match 逻辑加回 player_actor.rs。
5. 增加测试覆盖购买成功、资源不足、VIP 数据保存/下发。

文件归属：
- 可以新增/修改：crates/home/src/systems/shop.rs
- 可以新增/修改：crates/home/src/systems/vip.rs
- 可以修改：crates/home/src/systems/mod.rs
- 可以修改：crates/home/src/systems/registry.rs
- 可以小改：crates/home/src/actors/player_actor.rs，仅限字段初始化/注册所必需
- 可以使用：crates/shared/src/static_config/shop.rs、crates/shared/src/static_config/vip.rs
- 尽量不要修改 proto、world、shared battle

限制：
- 不要实现 Chat/Rank/GM。
- 不要改 ScoutReport/BattleResult/MailSystem 语义。
- 不要重构 registry 架构，只按现有模式接入。
- 不要做无关格式化。
- 你不是唯一 agent，不要回退别人改动。

验收：
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p home@0.1.0 -p proto
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto

完成后提交并推送：
rtk git -c core.quotepath=false add -A
rtk git -c core.quotepath=false commit -m "step15 add shop vip systems"
rtk git -c core.quotepath=false push -u origin HEAD:refs/heads/agent/step15-shop-vip

最终回答列出：
- 改动文件
- 测试结果
- 后续 Chat/Rank/GM 如何接入 registry
- 剩余风险
```

Shop/VIP 进行期间，不要让其他实现 agent 修改 `player_actor.rs`、`systems/registry.rs`、`systems/mod.rs`。

## 9. 后续并行提示词草案

### Chat

```text
你在 ~/DEV/javaCode/OnePunch/slg-rs 工作。从 origin/integration/step13-collect-return-base 创建 worktree：
~/DEV/javaCode/OnePunch-worktrees/slg-chat
分支：agent/step15-chat

任务：实现 Step 15 Chat 最小闭环。

目标：
1. 新增 chat actor/system 或最小 Home/Gateway 可路由聊天模块。
2. 支持私聊/频道/系统消息中的最小可测试子集。
3. 如果需要 Home 命令，必须通过 systems::registry 接入。
4. 增加测试覆盖消息发送、非法目标、基础路由。

限制：
- 不要改 Shop/VIP。
- 不要改 World battle/ScoutReport/MailSystem 语义。
- 如果必须修改 proto 或 gateway 路由，先保持改动最小并补测试。

验收：
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p gateway -p home@0.1.0 -p proto
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto

提交：
rtk git -c core.quotepath=false commit -m "step15 add chat baseline"
rtk git -c core.quotepath=false push -u origin HEAD:refs/heads/agent/step15-chat
```

### Rank/GM

```text
你在 ~/DEV/javaCode/OnePunch/slg-rs 工作。从 origin/integration/step13-collect-return-base 创建 worktree：
~/DEV/javaCode/OnePunch-worktrees/slg-rank-gm
分支：agent/step15-rank-gm

任务：实现 Step 15 Rank/GM 基线。

目标：
1. 建立排行榜快照的最小数据结构和查询入口。
2. 建立 GM 命令的最小入口，支持只读/受限修改的基础命令。
3. 如果接入 Home 命令，必须通过 systems::registry。
4. 补测试覆盖排行榜排序、空数据、非法 GM 命令。

限制：
- 不要改 Shop/VIP。
- 不要改 Chat。
- 不要改 World battle/ScoutReport/MailSystem 语义。
- 不要引入真实高权限破坏性命令。

验收：
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p home@0.1.0 -p shared -p proto
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto

提交：
rtk git -c core.quotepath=false commit -m "step15 add rank gm baseline"
rtk git -c core.quotepath=false push -u origin HEAD:refs/heads/agent/step15-rank-gm
```

### World 查询补全

```text
你在 ~/DEV/javaCode/OnePunch/slg-rs 工作。从 origin/integration/step13-collect-return-base 创建 worktree：
~/DEV/javaCode/OnePunch-worktrees/slg-world-query-completion
分支：agent/step15-world-query-completion

任务：补全 World 查询命令，逐步替换兼容空响应。

目标：
1. 梳理 crates/world/src/service.rs 中仍返回占位/空数据的查询命令。
2. 优先补全只读查询，不改 battle/scout/collect 语义。
3. 增加服务层测试，覆盖每个补全命令。

限制：
- 不要修改 Home registry/Shop/VIP/Chat/Rank/GM。
- 不要重构 sector actor。
- 如果必须碰 service.rs，当前任务独占该文件。

验收：
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p world -p proto
rtk cargo test --manifest-path Cargo.toml -p shared -p world -p home@0.1.0 -p proto

提交：
rtk git -c core.quotepath=false commit -m "step15 complete world query responses"
rtk git -c core.quotepath=false push -u origin HEAD:refs/heads/agent/step15-world-query-completion
```
