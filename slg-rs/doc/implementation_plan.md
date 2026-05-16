# slg-rs 总体实施计划

> 最后更新：2026-05-16
> 当前分支：`feature/phase2-home-systems`（Step 7 起在此分支）

---

## 全局进度

```
Step 1  协议兼容层 .............. ✅ 已完成
Step 2  静态配置系统 ............ ✅ 已完成
Step 3  玩家数据持久化 .......... ✅ 已完成
Step 4  Auth 服务修复 ........... ✅ 已完成
Step 5  Gateway 会话管理 ........ ✅ 已完成
Step 6  端到端联调 .............. ⏸️ 暂跳过（家里电脑已做，待合并）
Step 7  Home 核心系统骨架 ....... ✅ 已完成
Step 8  命令分发框架 ............ ✅ 已完成
Step 9  gRPC Dispatch 接口 ...... ✅ 已完成
Step 10 事件系统完善 ............ ✅ 已完成
Step 11 核心系统命令填充 ........ ✅ 已完成
Step 12 活动系统命令完善 ........ ✅ 已完成
Step 13 World 核心系统 .......... 🚧 进行中
Step 14 战斗引擎 ................ 待执行
Step 15 运营系统 ................ 待执行
Step 16 安全与上线准备 .......... 待执行
```

---

## 已完成步骤

### Step 1：协议兼容层（Phase 7）✅

实现 proto2 extensions 的自定义 wire-format 编解码，确保与 Java 客户端二进制兼容。

| 产出 | 文件 |
|------|------|
| GameMessage 编解码 | `crates/shared/src/msg.rs` |
| GameCodec 帧编解码 | `crates/gateway/src/codec.rs` |
| FunctionClientBase 17 个模块 | `crates/shared/src/msg.rs` |
| cmd_generated.rs 652 个命令号 | `crates/proto/build.rs` 自动生成 |
| GameCmdExt 路由 trait | `crates/shared/src/cmd.rs` |

测试：shared 5 个 + gateway 7 个 = 12 个全通过。

### Step 2：静态配置系统（Phase 9）✅

从 MySQL `s_` 表并行加载 16 个配置模块到内存，支持热加载。

| 产出 | 文件 |
|------|------|
| 16 个配置模块 | `crates/shared/src/static_config/*.rs` |
| StaticConfig 聚合容器 | `crates/shared/src/static_config/mod.rs` |
| ConfigWatcher 热加载 | `crates/shared/src/static_config/mod.rs` |

覆盖表：s_activity_plan/form_plan/form_define/cycle/task/form_sign/form_score/score_gear/rank/award、s_hero/hero_lv/hero_star/skill、s_building、s_tech_lv、s_equip/equip_lv、s_prop_conf、s_task/task_chapter/task_daily、s_shop/shop_prop、s_vip、s_skin、s_lord_equip/lord_equip_set、s_lord_talent/lord_talent_stage、s_mail、s_map/npc/mine/wall、s_battle_skill/battle_type/buff/buff_effect、s_pay。

### Step 3：玩家数据持久化 ✅

对齐真实数据库 `imperial_sim_game_hqy` 的表结构。

| 产出 | 文件 |
|------|------|
| PlayerDao（p_account/p_lord/p_data/p_global/p_server_config） | `crates/shared/src/persistence.rs` |
| 列名常量 col::* / global_col::* | `crates/shared/src/persistence.rs` |
| 紧急存盘到本地文件 | `crates/shared/src/persistence.rs` |
| DDL 对齐真实表 | `doc/db/auth_schema.sql`、`doc/db/role_schema.sql` |

关键设计：p_data 是宽列（25 个 blob 列），不是 KV；p_lord 独立表存领主资源；p_global 按 server_id 分区。

### Step 4：Auth 服务修复 ✅

| 产出 | 文件 |
|------|------|
| session.rs 改用 AsyncCommands | `crates/auth/src/session.rs` |
| service.rs 对齐真实 p_account | `crates/auth/src/service.rs` |
| 补 tracing-subscriber 依赖 | `crates/auth/Cargo.toml` |
| 移除不存在的 EtcdRegistry::register | `crates/auth/src/main.rs` |

### Step 5：Gateway 会话管理 ✅

| 产出 | 文件 |
|------|------|
| Session 5 态状态机 + SessionStore | `crates/gateway/src/session.rs` |
| 完整登录流程 DoLogin→Verify→BeginGame | `crates/gateway/src/handler.rs` |
| 心跳回包、cmd 路由、断开通知 | `crates/gateway/src/handler.rs` |
| 断开通知后台任务 | `crates/gateway/src/main.rs` |

补充进展（2026-05-16）：已补 Gateway 重复登录集成测试，覆盖新连接登录同账号时旧 session 收到断开信号，并通过断开通知链路触发 Home `PlayerOffline`。

### Step 6：端到端联调 ⏸️ 暂跳过

家里电脑已做，待合并。

### Step 7：Home 核心系统骨架 ✅

| 产出 | 文件 | p_data 列 |
|------|------|-----------|
| HeroSystem | `crates/home/src/systems/hero.rs` | hero_func |
| BackpackSystem | `crates/home/src/systems/backpack.rs` | backpack_func |
| TechSystem | `crates/home/src/systems/tech.rs` | technology_func |
| EquipSystem | `crates/home/src/systems/equip.rs` | equip_func |
| MissionSystem | `crates/home/src/systems/mission.rs` | mission_func |
| build_function_base_bytes_pub | `crates/shared/src/msg.rs` | — |

每个系统已实现：PlayerSystem trait（load/save/dirty/column_name）、ToFunctionClientBaseBytes、handle_command 骨架。全部接入 PlayerActor 的 load/save/tick/dispatch 流程。

---

## 待执行步骤

### Step 8：命令分发框架

**目标**：让 PlayerActor 能接收 Gateway 转发的任意业务命令，按 cmd 范围路由到对应系统。

**修改文件**：
- `crates/home/src/actors/player_actor.rs`
  - `PlayerMessage` 枚举新增 `GameCommand { cmd: u32, payload: Vec<u8>, reply: oneshot::Sender }` 变体
  - `run()` 循环中添加 `GameCommand` 分支
  - 新增 `handle_game_command(cmd, payload) -> Result<Vec<u8>>` 方法，按 cmd 范围路由：
    - 2001-2500 → hero_system
    - 背包命令范围 → backpack_system（需查 Bag.proto 确认 cmd 号）
    - 4201-4300 → tech_system
    - 4801-5000 → equip_system
    - 建筑命令范围 → building_system（需查 Simulate.proto 确认 cmd 号）
    - 任务命令范围 → mission_system（需查 Game.proto 确认 cmd 号）
    - 8001-8100 → activity_system

**参考文件**：
- `crates/proto/proto/Hero.proto`（cmd 2001-2500）
- `crates/proto/proto/Technology.proto`（cmd 4201-4206）
- `crates/proto/proto/Equip.proto`（cmd 4801-5000）
- `crates/proto/proto/Bag.proto`（查 extend Base 获取 cmd 号）
- `crates/proto/proto/Simulate.proto`（查 extend Base 获取 cmd 号）
- `crates/proto/proto/Game.proto`（任务相关 cmd 号）

**验证**：`cargo check -p home@0.1.0` 无 error。

---

### Step 9：gRPC Dispatch 接口

**目标**：Gateway 通过 gRPC 将业务消息转发到 Home Service，Home Service 路由到 PlayerActor 处理后返回响应。

**修改文件**：

1. `crates/proto/proto/service.proto` — 新增消息和 RPC：
   ```protobuf
   rpc Dispatch(DispatchRq) returns (DispatchRs);
   rpc PlayerOffline(PlayerOfflineRq) returns (PlayerOfflineRs);

   message DispatchRq { int64 role_id = 1; int32 cmd = 2; bytes payload = 3; }
   message DispatchRs { int32 code = 1; bytes payload = 2; }
   message PlayerOfflineRq { int64 role_id = 1; }
   message PlayerOfflineRs { int32 code = 1; }
   ```

2. `crates/home/src/service.rs` — 实现 `dispatch()` 和 `player_offline()`：
   - dispatch：从 PlayerManager 获取 Actor sender → 发送 GameCommand → 等待 reply → 返回
   - player_offline：发送 Shutdown 消息 → 等待 Actor 存盘退出

3. `crates/gateway/src/handler.rs` — 实现 `forward_to_home()`：
   - 调用 `HomeServiceClient::dispatch(DispatchRq)` → 将响应 payload 封装为 GamePacket 发回客户端

**验证**：`cargo check -p proto -p home@0.1.0 -p gateway` 无 error。

---

### Step 10：事件系统完善

**目标**：补全事件类型，让各系统操作能驱动任务/活动进度。

**修改文件**：

1. `crates/shared/src/event.rs`：
   - MissionType 枚举扩充（从 Java 版 MissionType 和 s_task.mission_type 提取，约 50 种）
   - GameEvent 新增变体：HeroLevelUp、HeroTierUp、BuildingUpgrade、TechResearchComplete、EquipStrengthen、ItemConsume、DiamondConsume、GoldConsume、TroopTrain 等

2. 各系统在关键操作后触发事件：
   - `hero.rs` level_up/tier_up 后 → `GameEvent::Mission(MissionEvent { mission_type: HeroLevelUp, ... })`
   - `tech.rs` 研究完成后 → `GameEvent::Mission(MissionEvent { mission_type: TechResearch, ... })`
   - 通过 PlayerActor.dispatch_event() 分发

**验证**：`cargo check -p shared -p home@0.1.0` 无 error。

---

### Step 11：核心系统命令填充

**目标**：为 6 个核心系统的 handle_command() 填充具体业务逻辑。

**架构决策（先确认再动手）**：
- 跨系统资源扣除：在 PlayerActor 层协调（推荐），而非系统内直接跨调
- StaticConfig 传递：handle_command 签名改为 `(&mut self, cmd, payload, config: &StaticConfig)`
- 奖励发放：PlayerActor 层统一 grant_award() 方法

**子步骤**：

11.1 HeroSystem（`hero.rs`）— 将领合成/升级/升星/技能/招募/编队/兵营/医院
11.2 BackpackSystem（`backpack.rs`）— 道具使用/合成
11.3 TechSystem（`tech.rs`）— 科技研究/加速/取消 + tick 研究完成检测
11.4 EquipSystem（`equip.rs`）— 装备穿戴/脱下/强化
11.5 BuildingSystem（`building.rs`）— 建筑升级/加速
11.6 MissionSystem（`mission.rs`）— 事件驱动进度更新 + 领取奖励

**参考**：各 proto 文件中的消息定义 + `crates/shared/src/static_config/` 中的配置结构。

**验证**：每个子步骤完成后 `cargo check`，建议为核心路径添加单元测试。

**当前进展（2026-05-16）**：
- `BackpackSystem` 已补道具增加、堆叠、消耗和 `PropUseRq/PropUseRs` 最小主链，支持 `s_prop_conf.rewardList` 发放背包类奖励，并补单元测试。
- `BuildingSystem` 已补 `SimDataFunction` load/save，`BuildStart`、`BuildLvUp`、`BuildSpeed` 会维护最小 `BaseBuildData` / `BuildQueue` 状态，并补单元测试。
- `PlayerActor` 已将 Dispatch `1109` 直接接到 `GetRoleDataRs` 组装逻辑，覆盖 BeginGame/RoleLogin 后获取全量功能数据的闭环测试。

---

### Step 12：活动系统命令完善 ✅

**目标**：完善 4 种已有玩法骨架（签到/任务/积分/最强领主）的具体逻辑。

**修改文件**：
- `crates/home/src/systems/activity/mod.rs` — handle_command 分发
- `crates/home/src/systems/activity/forms/sign.rs` — 签到逻辑
- `crates/home/src/systems/activity/forms/task.rs` — 任务进度更新
- `crates/home/src/systems/activity/forms/score.rs` — 积分累计 + 档位领取
- `crates/home/src/systems/activity/forms/supreme_lord.rs` — 阶段排行

**验证**：`cargo check -p home@0.1.0` + 单元测试。

---

### Step 13：World 核心系统

**目标**：完善大地图、行军、战斗触发。

**修改文件**：`crates/world/src/` 下的 map/march/sector_actor 等。

**需求文档**：`doc/requirements/world-systems.md`

**当前进展（2026-05-16）**：
- 已修正 `50001..=50040`、`5121..=5220` World 命令路由归属，避免大地图/行军命令落到 Home。
- 已补 `MapGrid` 实体索引、坐标边界校验、按类型查询、跨 Grid 移动。
- 已补 AOI 订阅/取消/移动更新逻辑，并覆盖入图、迁城、离图链路。
- 已补确定性行军 API：起止时间计算、状态更新、重复 key/非法速度/非法坐标校验、到达动作分类。
- `MapSectorActor` 已接入行军开始/到达 AOI 事件，到达后按战斗、采集、侦查、驻防、返回分类触发占位逻辑。
- 已修复 `TimerWheel` seconds/overflow 下沉到当前 tick 时延后一轮的问题，并补回归测试。
- 已补 `WorldOutboundDispatcher`，Home 目标事件可进入 channel consumer，Battle 目标暂保留占位 sink。

**本轮 P0 进展（2026-05-16）**：
- `WorldService::Dispatch` 已经把第一批 World 查询命令和派兵命令接通，查询仍然走 `MapGrid` / `MarchingManager` 内存视图。
- World 派兵命令已从孤立的 `MarchingManager` 写入，改为同时投递到最小 `WorldRuntime` / `SectorActor` sender registry。
- 创角链路已修复并保持可用，World 侧不再阻断后续业务命令分发。
- 已补 `EnterWorldMap -> MovePosition -> LeaveWorldMap` 集成链路测试。
- 已补 `BeginGame -> RoleLogin -> Dispatch 1109 GetRoleData` 集成链路测试。

**剩余 P0 / 未使用项**：
- `SectorActor` 的 WAL 到达处理仍是骨架，未做真实到达结算。
- 真实实体生成与实体生命周期管理未接入。
- 战斗、采集、侦查等到达业务仍是占位逻辑，未实现结算与结果回写。
- Home outbound channel 已接入，但仍缺少明确的 Home RPC/协议消息承载 World 到达事件。
- 其他 World 业务命令仍待分批接入，当前只覆盖第一批查询与派兵链路。

---

### Step 14：战斗引擎

**目标**：实现回合制战斗计算、战报生成。

**需求文档**：`doc/requirements/battle-engine.md`

---

### Step 15：运营系统

**目标**：邮件、聊天、社交、商店、VIP、排行榜、GM 工具。

**需求文档**：`doc/requirements/home-systems.md`（P2/P3 部分）

---

### Step 16：安全与上线准备

**目标**：协议加密、限流、压力测试、跨服系统。

**需求文档**：`doc/requirements/security.md`、`doc/requirements/cross-server.md`

---

## 附录：关键文件索引

### 核心代码

| 文件 | 说明 |
|------|------|
| `crates/shared/src/msg.rs` | GameMessage 编解码 + FunctionClientBase |
| `crates/shared/src/cmd.rs` | GameCmd re-export + GameCmdExt 路由 trait |
| `crates/shared/src/persistence.rs` | PlayerDao + 列名常量 |
| `crates/shared/src/event.rs` | GameEvent / MissionType / EventDispatcher |
| `crates/shared/src/static_config/mod.rs` | StaticConfig 聚合 + ConfigWatcher |
| `crates/home/src/actors/player_actor.rs` | PlayerActor 主循环 |
| `crates/home/src/managers/player_manager.rs` | 在线玩家管理 |
| `crates/home/src/service.rs` | HomeService gRPC 实现 |
| `crates/home/src/systems/mod.rs` | PlayerSystem trait 定义 |
| `crates/gateway/src/handler.rs` | 连接处理 + 状态机 |
| `crates/gateway/src/session.rs` | Session + SessionStore |
| `crates/gateway/src/codec.rs` | GameCodec + GamePacket |
| `crates/auth/src/service.rs` | AuthService gRPC 实现 |
| `crates/auth/src/session.rs` | Redis SessionManager |

### Proto 文件

| 文件 | 系统 | cmd 范围 |
|------|------|---------|
| `Hero.proto` | 将领 | 2001-2500 |
| `Bag.proto` | 背包 | 查 extend Base |
| `Technology.proto` | 科技 | 4201-4206 |
| `Equip.proto` | 装备 | 4801-5000 |
| `Simulate.proto` | 模拟经营/建筑 | 查 extend Base |
| `Game.proto` | 登录/创角/任务 | 1101-1200 |
| `Activity.proto` | 活动 | 8001-8100 |
| `Http.proto` | DoLogin/Verify | 103-140 |

### 数据库表（MCP mysql_ini 可查）

| 表 | 用途 |
|----|------|
| `s_hero` / `s_hero_lv` / `s_hero_star` | 将领配置 |
| `s_tech_lv` | 科技配置 |
| `s_equip` / `s_equip_lv` | 装备配置 |
| `s_prop_conf` | 道具配置 |
| `s_task` / `s_task_chapter` / `s_task_daily` | 任务配置 |
| `s_building` / `s_sim_building_conf` | 建筑配置 |
| `s_activity_*` | 活动配置（10 张表） |
| `p_account` / `p_lord` / `p_data` / `p_global` | 玩家数据（MCP mysql_game） |
