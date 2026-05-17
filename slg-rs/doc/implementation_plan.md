# slg-rs 总体实施计划

> 最后更新：2026-05-17
> 当前阶段：Step 13，World 核心系统进行中
> 说明：本文档是当前唯一进度源。历史执行稿见 [archive](./archive/README.md)。

## 现在进行到哪一步

当前推进到 **Step 13：World 核心系统**。

Step 1-12 已经完成基础落地或第一轮接线，但其中不少模块仍是“可编译、可联调骨架”，不是完整业务验收。当前重点是把 World 的地图、行军、到达事件和 Home 回调链路做成可继续接战斗/采集/侦查结算的基础。

## 全局进度

| Step | 内容 | 当前状态 | 说明 |
|------|------|----------|------|
| 1 | 协议兼容层 | 已完成 | `GameMessage` / `GameCodec` / FunctionClientBase 基础兼容已落地 |
| 2 | 静态配置系统 | 已完成 | `StaticConfig` 聚合与多模块加载已落地，真实表结构仍需持续校验 |
| 3 | 玩家数据持久化 | 已完成 | `p_lord` / `p_data` 读写、定时存盘、下线存盘已接入 |
| 4 | Auth 服务修复 | 已完成 | 基础登录和 token 验证可用，渠道适配仍是后续项 |
| 5 | Gateway 会话管理 | 已完成 | 状态机、重复登录、断线通知、Home/World 路由已接入 |
| 6 | 端到端联调 | 部分完成 | 已补关键链路测试，真实客户端联调仍待执行 |
| 7 | Home 核心系统骨架 | 已完成 | Hero/Bag/Building/Tech/Equip/Mission/Skin 接入 load/save/dispatch |
| 8 | 命令分发框架 | 已完成 | PlayerActor 已按 cmd 分发到系统模块 |
| 9 | gRPC Dispatch 接口 | 已完成 | Gateway -> Home / World Dispatch 已接通 |
| 10 | 事件系统完善 | 已完成基础 | PlayerActor 内事件、Mission/Activity 驱动已接入，全服事件仍需更多业务使用 |
| 11 | 核心系统命令填充 | 首轮完成 | 背包、建筑、任务、科技、装备、英雄均有首轮实现，但业务校验仍不完整 |
| 12 | 活动系统命令完善 | 首轮完成 | 签到、任务、积分、最强领主等基础逻辑已落地 |
| 13 | World 核心系统 | 进行中 | 地图/AOI/行军/Sector/outbound 框架已落地，采集返回回写已有基线，战斗/侦查和生产级结算仍未完成 |
| 14 | 战斗引擎 | 待执行 | 攻城/PvE/PvP 战斗计算与战报生成 |
| 15 | 运营系统 | 待执行 | 邮件、聊天、社交、商店、VIP、排行榜、GM 工具 |
| 16 | 安全与上线准备 | 待执行 | 加密、限流、压测、跨服、上线检查 |

## Step 13 已完成范围

| 范围 | 当前产出 |
|------|----------|
| World 命令路由 | `50001..=50040`、`5121..=5220` 路由到 World Service |
| 地图实体 | `MapGrid` 支持实体索引、坐标边界、按 area/block/type 查询、迁城移动 |
| AOI | 支持入图订阅、迁城更新、离图取消，测试覆盖基本链路 |
| 行军 | 支持派兵、侦查、召回、加速、玩家部队查询和非法参数校验 |
| SectorActor | Runtime sender registry 已接入，实体和部队可同步进 Sector |
| 到达分类 | 到达事件已按战斗、采集、侦查、驻防、返回分类生成 outbound 事件 |
| 驻防/采集状态 | Sector 到达后能恢复采集状态和驻防状态 |
| Home 回调链路 | `service.proto` 新增 `WorldOutbound` 与非战斗 typed payload，World outbound consumer 可按 troop owner 解析 role_id 并调用 HomeService |
| 采集返回回写 | 采集完成后 Sector 会生成返回行军，WAL 可恢复采集/返回状态，返回到家后发送 `CollectReturned`，Home 可应用资源奖励并把阵型置回空闲 |
| 测试 | 覆盖 World 查询、派兵、AOI、迁城、到达事件、Home outbound request 构造等 |

## Step 13 剩余 P0

| 优先级 | 问题 | 说明 |
|--------|------|------|
| P0 | 到达结算仍需生产化 | 采集完成/返回资源回写已有确定性和 WAL 恢复基线，但矿点配置、产量公式、幂等投递、兵力返回和真实战斗/侦查结算仍未完整接入 |
| P0 | 实体生命周期仍是最小实现 | 默认实体生成已可用，但真实地图初始化、刷新规则、持久化和配置驱动仍未完整接入 |
| P0 | 战斗入口只返回缺口 | fight/declare/join 能解码并明确返回 battle service 未接入，还没有战斗引擎 |
| P1 | World 查询仍偏兼容空响应 | 部分命令为了客户端兼容返回空结构，后续需要逐个补业务数据 |
| P1 | 全局事件使用不足 | EventBus 已建，但 World/Home 的真实业务事件还需要更多发布点 |

## 下一步建议

1. 将采集结算从当前确定性默认值升级为矿点配置、产量公式和幂等投递驱动。
2. 把侦查报告 payload 从“请求生成报告”扩展成真实侦查结果，接入邮件/报告存储。
3. 接入战斗引擎前，先把 `DeclareFight` / `JoinTheFight` 的请求校验、目标查找、占位错误码整理稳定。
4. 每补一个 World 命令，要求至少有服务层测试和 Sector 状态测试，避免只做到兼容空响应。

## 当前验收基线

最近一次已知通过：

```bash
rtk cargo test --manifest-path Cargo.toml -p world -p home@0.1.0 -p proto
```

提交前仍建议跑：

```bash
rtk git -c core.quotepath=false diff --check
rtk cargo test --manifest-path Cargo.toml -p world -p home@0.1.0 -p proto
```

## 关键代码索引

| 文件 | 说明 |
|------|------|
| `crates/shared/src/msg.rs` | GameMessage 编解码、FunctionClientBase 构造 |
| `crates/shared/src/cmd.rs` | GameCmd 路由规则 |
| `crates/shared/src/static_config/mod.rs` | StaticConfig 聚合与热加载 |
| `crates/shared/src/persistence.rs` | PlayerDao、p_lord/p_data 读写 |
| `crates/gateway/src/handler.rs` | Gateway 状态机、Home/World Dispatch |
| `crates/home/src/service.rs` | HomeService BeginGame/CreateRole/Dispatch/WorldOutbound |
| `crates/home/src/actors/player_actor.rs` | PlayerActor 主循环、GetRoleData、系统分发 |
| `crates/world/src/service.rs` | WorldService Dispatch、地图/行军查询与命令处理 |
| `crates/world/src/runtime.rs` | WorldRuntime 与 Sector sender registry |
| `crates/world/src/sector_actor.rs` | SectorActor、到达状态处理、WAL 恢复 |
| `crates/world/src/outbound.rs` | World outbound 事件分类与 Home request 构造 |
