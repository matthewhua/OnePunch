# Rust SLG 文档入口

> 最后整理：2026-05-17
> 当前唯一进度源：[总体实施计划](./implementation_plan.md)

本目录只保留当前仍需要维护的计划、需求和设计补充。已完成阶段的执行稿、旧评估和过期现状分析已经移动到 [archive](./archive/README.md)。

## 当前进度

目前推进到 **Step 13：World 核心系统**。

已经落地的是 World 查询、地图实体索引、AOI 订阅、行军基础命令、SectorActor 到达事件框架、World 到 Home 的 `WorldOutbound` RPC 投递链路、非战斗 outbound typed payload 基线，以及采集完成后的返回行军、WAL 恢复和 Home 资源/阵型回写基线。还没有完成的是战斗引擎接入、真实侦查报告、以及配置/幂等投递驱动的生产级到达结算。

## 维护中的文档

| 文档 | 用途 | 状态 |
|------|------|------|
| [implementation_plan.md](./implementation_plan.md) | 当前阶段、已完成范围、下一步优先级 | 当前维护 |
| [login_and_character_creation_design.md](./login_and_character_creation_design.md) | 登录与创角链路设计 | 参考中 |
| [single_world_server_guide.md](./single_world_server_guide.md) | 早期单 World 节点部署说明 | 参考中 |
| [phase8_eventbus_optimization.md](./phase8_eventbus_optimization.md) | EventBus 后续优化建议 | 待排期 |

## 需求基线

这些文档描述目标能力，不代表代码已经全部实现。

| 文档 | 范围 |
|------|------|
| [requirements/gateway-service.md](./requirements/gateway-service.md) | Gateway 连接、编解码、路由 |
| [requirements/auth-service.md](./requirements/auth-service.md) | Auth 登录、Token、渠道适配 |
| [requirements/home-service.md](./requirements/home-service.md) | Home / PlayerActor 主框架 |
| [requirements/world-service.md](./requirements/world-service.md) | World 服务架构 |
| [requirements/protocol-compat.md](./requirements/protocol-compat.md) | proto2 extension 兼容 |
| [requirements/static-config.md](./requirements/static-config.md) | 静态配置加载与热更 |
| [requirements/persistence.md](./requirements/persistence.md) | 玩家数据持久化 |
| [requirements/event-system.md](./requirements/event-system.md) | 事件系统 |
| [requirements/activity-framework.md](./requirements/activity-framework.md) | 活动框架 |
| [requirements/home-systems.md](./requirements/home-systems.md) | Home 功能系统 |
| [requirements/world-systems.md](./requirements/world-systems.md) | World 功能系统 |
| [requirements/battle-engine.md](./requirements/battle-engine.md) | 战斗引擎 |
| [requirements/cross-server.md](./requirements/cross-server.md) | 跨服系统 |
| [requirements/observability.md](./requirements/observability.md) | 可观测性 |
| [requirements/security.md](./requirements/security.md) | 安全与反作弊 |

## 历史设计

`../rust-rewrite/` 是早期 Rust 重构设计资料。它仍可用于理解背景和设计取舍，但不再作为当前进度或验收状态的来源。

## 归档规则

- 已经实现完的分步执行稿进入 `doc/archive/`。
- 与当前代码状态冲突的现状分析进入 `doc/archive/`。
- 新增计划只更新 `implementation_plan.md`，避免多个文档同时声明“当前进度”。
