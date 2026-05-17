# Claude 计划 / Gemini 代码执行评估

> 评估日期：2026-05-12
> 评估范围：`doc/implementation_plan.md` 中已标记完成的 Step 1-11
> 对照文档：`doc/implementation_plan.md`、`doc/implementation_plan7.md`、`doc/task8.md`、`doc/task9.md`
> 核验方式：静态代码审查 + `cargo check --workspace` + `cargo test --workspace`

---

## 一、结论先看

这批工作不是“写得很差”，但也绝对不能按“Step 1-11 已高质量完成”来认定。

- Claude 文档计划质量：**8/10**
- Gemini 代码落地质量：**6/10**
- 当前整体可交付度：**5.5/10**

结论：

1. **计划文档整体是合格的**，拆分顺序、阶段边界、文件定位都比较清楚，能指导执行。
2. **Gemini 产出的代码有明显推进**，尤其是协议层、静态配置、持久化骨架、Gateway 会话、Home 命令分发这些大框架已经搭起来了。
3. **但“文档已完成”与“代码真正完成”之间存在明显落差**。不少 Step 实际只是“能编译的骨架”或“半成品逻辑”，还没达到可联调、可验收、可长期维护的标准。
4. 当前最关键的问题不是代码风格，而是**关键链路没有真正闭环**，尤其是 `BeginGame -> PlayerActor 在线态 -> Dispatch` 这条主链。

---

## 二、评分

### 1. Claude 计划文档评分：8/10

优点：

- Step 划分清楚，基本符合真实开发顺序。
- 大部分步骤都明确到 crate / 文件级别，执行指向性强。
- 对协议兼容、静态配置、事件系统、Home 路由这些高风险点有前置识别。
- `implementation_plan.md` 比单纯需求文档更接近“执行蓝图”，这点是对的。

不足：

- 对“完成标准”的定义偏松，很多 Step 只写 `cargo check` 通过，没有要求链路级验收。
- 没把“骨架完成”和“业务完成”明确区分，导致后续容易把半成品标记为完成。
- 没有把关键联调断点单独列为 Gate，例如：
  - `BeginGame` 后是否保证在线 Actor 已存在
  - `GetRoleDataRs` 是否完整拼装
  - 错误码、请求/响应封包是否与客户端真实约定一致

建议：

- 后续 plan 每一步都补一条“最低验收标准”，不要只写编译通过。
- 对业务 step 增加三档状态：`骨架完成 / 可联调 / 可上线`。

### 2. Gemini 代码执行评分：6/10

优点：

- 能把大段计划快速落成代码，不是只写空文件。
- `shared::msg` 的 proto2 extension 兼容实现是有效推进，不是纯占位。
- `static_config`、`persistence`、`gateway session`、`home dispatch` 这些模块都已经形成可运行骨架。
- `cargo check --workspace` 通过，`shared/gateway/home` 的测试也都通过，说明基本工程纪律还在。

不足：

- 多个 Step 被“提前标记完成”，但代码仍停留在占位或简化实现。
- 关键链路存在真实断点，不是小瑕疵。
- 测试覆盖偏窄，更多是在测局部编码/解码，没覆盖核心业务闭环。
- 一些接口看似接上了，实际上没有被上游真正调用，属于“存在但未闭环”。

---

## 三、核验结果

### Step 1 协议兼容层：**8/10，基本完成**

证据：

- `GameMessage` 已实现 Base + extension 的手工解析与构造：[msg.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/shared/src/msg.rs:39)
- `GameCodec` 已实现长度头 + cmd + payload 的帧编解码：[codec.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/gateway/src/codec.rs:33)
- 对应单测通过：`shared` 5 个、`gateway` 7 个。

评价：

- 这是目前兑现度最高的一步。
- 至少从本地代码和单测看，Proto2 extension 兼容方向是对的。

不足：

- 目前更多证明“本项目内部编码解码自洽”，还没有证明“与 Java 客户端抓包完全一致”。

### Step 2 静态配置系统：**8/10，框架完成度较高**

证据：

- 16 类配置已聚合进 `StaticConfig` 并并行加载：[static_config/mod.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/shared/src/static_config/mod.rs:68)
- 各模块确实在从 MySQL `s_` 表读取，不只是空壳。

评价：

- 这一步不只是写了骨架，实际已经有批量加载和基础校验。

不足：

- 缺少针对真实表结构的集成验证。
- 热加载机制有 watch channel，但没有看到 GM/管理入口真正触发 reload。

### Step 3 玩家数据持久化：**7.5/10，核心骨架完成**

证据：

- `PlayerDao`、`p_lord/p_data/p_global/p_server_config` 访问接口已存在：[persistence.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/shared/src/persistence.rs:167)
- 定时存盘、下线存盘、紧急存盘逻辑已接入 `PlayerActor`：[player_actor.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/actors/player_actor.rs:262)

评价：

- 这一步从结构上看是成立的，不算假完成。

不足：

- 新角色创建流程没有把 `p_account` 写完整接上，链路仍有缺口。
- 缺少存盘一致性测试，尤其是 dirty 模块与失败回退。

### Step 4 Auth 服务修复：**7/10，能用但不完整**

证据：

- Redis session 已改用 `AsyncCommands`：[session.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/auth/src/session.rs:20)
- `AuthService` 已直接查 `p_account` 并生成 token：[service.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/auth/src/service.rs:24)

评价：

- 这一步解决了基础可用性问题。

不足：

- 仍然是简化版 auth，没有渠道适配、没有更完整封号/注册/续期流程。

### Step 5 Gateway 会话管理：**7/10，主状态机已经搭好**

证据：

- 会话状态机、SessionStore、断开通知都在：[session.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/gateway/src/session.rs:16)
- Gateway 的连接生命周期和消息分发主循环已经成型：[handler.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/gateway/src/handler.rs:134)

评价：

- 这一步属于“框架已成，细节未满”。

不足：

- `World` 路由还是 TODO：[handler.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/gateway/src/handler.rs:363)
- gRPC client 每次临时连接，没有复用。

### Step 7 Home 核心系统骨架：**6.5/10，名义完成，实际不均衡**

证据：

- `Hero/Backpack/Tech/Equip/Mission` 都有独立系统和 `PlayerSystem` 接口。
- `FunctionClientBase` trait 已有统一封装：[msg.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/shared/src/msg.rs:256)

不足：

- `BuildingSystem` 仍基本是空壳，load/save 都是空实现：[building.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/systems/building.rs:48)
- `ActivitySystem` 的持久化也还是空实现：[activity/mod.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/systems/activity/mod.rs:236)
- `GetRoleDataRs` 只拼了活动模块，其他系统没有真正组装：[player_actor.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/actors/player_actor.rs:375)

结论：

- 如果把“骨架”理解为接口和结构存在，可以算完成。
- 如果把“骨架”理解为所有核心模块都能基本装载/导出，则不能算完全完成。

### Step 8 命令分发框架：**7/10，已接通但路由策略仍粗**

证据：

- `PlayerMessage::GameCommand` 和 `handle_game_command` 已存在：[player_actor.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/actors/player_actor.rs:48)

不足：

- `shared::cmd::route()` 仍然是粗粒度区间路由，不是基于真实命令表：[cmd.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/shared/src/cmd.rs:27)
- 文档说要精确核对 `Bag/Simulate/Game` 的命令范围，代码里仍是经验区间，不是严格对齐。

### Step 9 gRPC Dispatch：**5/10，接口存在，但主链未闭环**

证据：

- `service.proto` 已新增 `Dispatch` / `PlayerOffline`。
- Home / Gateway 都有对应实现：[service.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/service.rs:176)、[handler.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/gateway/src/handler.rs:385)

P0 问题：

- Gateway `BeginGame` 只调用 `home.begin_game()`，没有接 `RoleLogin`，也没有确保 `PlayerActor` 已在线：[handler.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/gateway/src/handler.rs:289)
- 但后续 `Dispatch` 强依赖 `PlayerManager.get_by_role(role_id)`：[service.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/service.rs:188)

结论：

- 这意味着“协议上可以 BeginGame，逻辑上未必有在线 Actor 可以接业务命令”。
- 这是当前最严重的链路级问题。

### Step 10 事件系统完善：**6.5/10，框架完成，跨服事件未真正落地**

证据：

- `MissionType`、`GameEvent`、`GlobalEvent`、`EventDispatcher` 都有了：[event.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/shared/src/event.rs:13)
- `ActivitySystem` 和 `MissionSystem` 已接事件分发。

不足：

- `MissionType` 数量与计划中的“约 50 种”有明显差距，目前更像第一批核心类型。
- `GlobalEventBus` 存在，但 `PlayerActor` 中几乎没有真正发布 `global_event_bus.publish(...)` 的调用，属于“总线已建，业务没上车”：[global_event_bus.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/actors/global_event_bus.rs:10)

### Step 11 核心系统命令填充：**4.5/10，不能算真正完成**

这是当前文档和代码落差最大的步骤。

原因：

- `HeroSystem` 虽然有多个 cmd 分支，但大量关键校验仍是 TODO，逻辑明显简化：[hero.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/systems/hero.rs:71)
- `BackpackSystem` 基本只返回空字节，没有真正的协议响应和业务处理：[backpack.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/systems/backpack.rs:93)
- `BuildingSystem` 仍是占位实现：[building.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/systems/building.rs:26)
- `EquipSystem`、`TechSystem`、`MissionSystem` 虽有部分逻辑，但仍大量简化和 TODO。

结论：

- Step 11 最多只能写成“第一批命令接线完成，核心业务逻辑未完成”。
- 把它标为 `✅ 已完成`，评价偏乐观。

---

## 四、关键不足

### P0：必须先修

1. **`BeginGame -> RoleLogin -> Dispatch` 没真正闭环**
   - 现状：`BeginGame` 不保证 `PlayerActor` 已创建，`Dispatch` 却要求玩家在线 Actor 已存在。
   - 影响：游戏内命令可能直接 `player not online`。

2. **`GetRoleDataRs` 组装严重不完整**
   - 现状：当前只塞了 `activity`，其他系统都没装进去。
   - 证据：[player_actor.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/actors/player_actor.rs:375)
   - 影响：客户端登录后拿到的功能树数据不完整，真实联调会出问题。

3. **Step 11 被过度标记为完成**
   - 现状：背包、建筑、招募、资源扣除、奖励发放仍大量缺失。
   - 影响：计划状态失真，会误导后续评审和迭代排期。

### P1：应尽快修

1. **事件总线定义了，但全局事件实际使用率很低**
2. **命令路由仍靠粗区间，不够严谨**
3. **`PlayerOffline` 清理映射时用 `account_id = 0`，只会删 role 索引，不会删 account 索引**
   - 证据：[service.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/service.rs:227)
   - 证据：[player_manager.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/home/src/managers/player_manager.rs:88)
4. **`cargo test --workspace` 仍未全绿**
   - `world` 的 `timer_wheel` 单测失败：[timer_wheel.rs](/Users/huaqiyuan/DEV/javaCode/OnePunch/slg-rs/crates/world/src/timer_wheel.rs:112)

### P2：质量问题

1. warning 较多，说明代码收尾不够干净
2. 很多测试只测“编码解码自洽”，没测“服务链路闭环”
3. 不少实现仍是简化版逻辑，暂时没对齐真实 SLG 业务约束

---

## 五、我给的最终判断

如果这是一次“阶段性代码审阅”，我的判断是：

- **Claude 写的 plan 可以继续用，但要收紧完成标准。**
- **Gemini 生成的代码可以作为第一轮实现基础，但不能直接按“前 11 步已完成”验收。**

更准确的状态应该改成：

- Step 1-5：**大体完成**
- Step 7-10：**框架完成，联调未完全验证**
- Step 11：**仅第一轮落地，未完成**

---

## 六、建议的修正文案

建议把 `doc/implementation_plan.md` 中相关状态改得更诚实一些：

- Step 7：`✅ 核心系统骨架初版完成`
- Step 8：`✅ 命令分发框架完成`
- Step 9：`⚠️ Dispatch 接口完成，主链待闭环验证`
- Step 10：`✅ 事件系统基础完成`
- Step 11：`⚠️ 核心系统命令首轮填充完成，业务逻辑未完成`

---

## 七、下一步优先级

建议按这个顺序补：

1. 先修 `BeginGame/RoleLogin/Dispatch` 主链闭环。
2. 补全 `GetRoleDataRs` 的全模块组装。
3. 重新拆 Step 11，把 Hero / Backpack / Building / Equip / Tech / Mission 分开单独验收。
4. 把 `cargo test --workspace` 清到全绿。
5. 再进入 Step 12 活动系统深化。

