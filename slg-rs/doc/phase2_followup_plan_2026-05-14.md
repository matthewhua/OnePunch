# Phase 2 后续路线与执行清单

> 日期：2026-05-14
> 当前分支：`feature/phase2-home-systems`
> 范围：Step 14 收口、Phase 2 关键债务修正、Step 15/16 后续拆分执行

---

## 一、当前状态快照

### 1. 分支与工作区

- 当前工作区：`C:\dev\Company\OnePunch\slg-rs`
- 当前分支：`feature/phase2-home-systems`
- 最近 Step 13 相关提交：`39d2c7ee feat(world): start step13 core systems`
- 当前未提交边界：
  - staged：`crates/shared/src/battle/` 下 7 个新增战斗引擎文件
  - unstaged：`crates/shared/src/lib.rs`
  - unstaged：`doc/implementation_plan.md`

### 2. Step 状态

| Step | 当前状态 | 说明 |
| --- | --- | --- |
| Step 13 World 核心系统 | 代码已提交，计划文档仍显示进行中 | 需要后续单独核对 World 主链路闭环，不在 Step 14 提交里扩大范围 |
| Step 14 战斗引擎 | 代码已实现，待验证和提交 | `shared::battle` 新模块已 staged，`lib.rs` 导出与计划文档状态待一并 staging |
| Step 15 运营系统 | 未开始 | 邮件、聊天、社交、商店、VIP、排行榜、GM 工具 |
| Step 16 安全与上线准备 | 未开始 | 协议加密、限流、压力测试、跨服系统 |

---

## 二、Step 14 收口清单

目标：把当前 `shared::battle` 战斗引擎实现收成一个干净提交，不混入 Step 15/16 或业务闭环重构。

### 必做项

- 确认 `crates/shared/src/lib.rs` 导出 `pub mod battle;`
- 确认 `doc/implementation_plan.md` 将 Step 14 标记为已完成，并记录完成内容
- 确认新增模块边界只在 `shared::battle`：
  - `model.rs`：战斗输入模型、兵种、选项
  - `calculator.rs`：伤害计算与兵种克制
  - `skill.rs`：主动/被动技能触发
  - `round.rs`：回合执行与胜负判定
  - `report.rs`：战报动作与 protobuf 编码
  - `result.rs`：结算摘要
  - `mod.rs`：公开导出与定向单测

### 验证

- 只跑定向测试：`cargo test -p shared battle::`
- 跑 diff 空白检查：`git diff --check`
- 不跑宽泛 workspace 编译；除非公共接口改动扩散到其他 crate，再单独升级验证范围。

### 提交建议

- 提交范围：
  - `crates/shared/src/battle/*`
  - `crates/shared/src/lib.rs`
  - `doc/implementation_plan.md`
- 建议提交信息：`feat(shared): implement step14 battle engine`

---

## 三、遗留债务清单

这些债务不是 Step 14 的收口范围，但会影响后续 Phase 2 是否能真实联调。

### P0：先补核心业务闭环

1. Step 11 核心系统命令填充真实完成度不足
   - `HeroSystem`、`EquipSystem`、`TechSystem`、`MissionSystem` 仍有大量简化逻辑和 TODO。
   - `BackpackSystem`、`BuildingSystem` 更接近占位或空响应。
   - 后续应按系统拆分验收，而不是继续把 Step 11 视为业务完成。

2. `GetRoleDataRs` 组装不完整
   - 当前登录后功能树数据主要集中在活动模块。
   - 需要补齐 Hero、Backpack、Building、Tech、Equip、Mission 等系统的 `FunctionClientBase` 组装。
   - 验收标准应覆盖客户端登录后能拿到完整功能数据。

3. 命令路由与响应 payload 精度不足
   - `shared::cmd::route()` 仍是经验区间路由。
   - Gateway 的 World 路由仍是 TODO。
   - Home dispatch 会拆 Base payload，但仍需要按真实命令表核对请求/响应封包。

### P1：补完主链路可观测问题

- `BeginGame -> RoleLogin/GetRoleData -> Dispatch` 主链路要形成明确闭环。
- `PlayerOffline` 清理索引需要确认 account/role 双索引都正确释放。
- 全局事件总线已存在，但核心业务实际 publish 覆盖不足。

---

## 四、后续路线

### 阶段 A：核心业务闭环修正

优先把已标记完成但实际偏骨架的部分补到可联调：

1. 重评 Step 11，并把每个核心系统拆成单独验收项。
2. 补齐 `GetRoleDataRs` 的全模块功能数据组装。
3. 校准命令路由：从粗区间路由升级到基于真实命令表的路由清单。
4. 打通 Gateway/Home/World 的请求转发和错误响应封包。

### 阶段 B：进入 Step 15 运营系统

建议拆分顺序：

1. 邮件与附件奖励
2. GM/运维入口
3. 聊天与社交
4. VIP 与商店
5. 排行榜

拆分原则：先做能承载战报、补偿、活动奖励的邮件系统，再做 GM 与运营入口；聊天/社交和商业化系统可以在邮件闭环后并行推进。

### 阶段 C：进入 Step 16 安全与跨服

Step 16 应放在核心业务与运营闭环之后：

1. 协议加密与签名校验
2. 网关限流与频控
3. 压测与容量基线
4. 跨服匹配、跨服战斗、跨服排行榜

---

## 五、子代理执行策略

父代理职责：

- 维护路线、任务边界和 review。
- 统一检查跨 crate 接口变化。
- 控制验证范围，避免无关宽泛编译。

worker 子代理职责：

- 按互不重叠的文件范围实际改代码。
- 每个 worker 只拥有自己的模块目录，不回退或覆盖其他 worker 的改动。
- 每个 worker 只跑自己范围内的定向测试，并在最终报告中列出变更文件。

建议拆分：

| Worker | 负责范围 | 主要目标 |
| --- | --- | --- |
| Worker A | `crates/home/src/actors/player_actor.rs`、系统导出接口 | 补齐 `GetRoleDataRs` 组装 |
| Worker B | `crates/shared/src/cmd.rs`、`crates/gateway/src/handler.rs` | 精确命令路由与 World 转发边界 |
| Worker C | `crates/home/src/systems/{backpack,building,hero,equip,tech,mission}.rs` | Step 11 核心业务分系统补齐 |
| Worker D | `crates/home/src/systems/mail*`、邮件 proto 使用点 | Step 15 邮件与附件奖励 |
| Worker E | `crates/home/src/systems/{chat,social,vip,shop,rank}*` | Step 15 运营系统后半段 |

---

## 六、验证策略

- 文档整理：静态核对 `git status`、`doc/implementation_plan.md`、需求文档和 TODO 搜索结果。
- Step 14 收口：`cargo test -p shared battle::` + `git diff --check`。
- 核心业务闭环：优先补系统级单测和 actor/dispatch 定向测试。
- Step 15/16：每个 worker 只跑自己范围内的定向测试；公共接口变更时再扩大到相关 crate。
