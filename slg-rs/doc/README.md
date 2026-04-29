# Rust SLG 项目文档中心

> 本目录包含 Rust SLG 游戏服务端重构项目的所有设计文档、需求文档和技术方案。

---

## 文档总览

### 一、项目总纲

| 文档 | 说明 | 状态 |
|------|------|------|
| [../rust-rewrite/README.md](../rust-rewrite/README.md) | 整体架构设计与迁移路线图 | ✅ 已完成 |
| [项目现状与差距分析](./project-gap-analysis.md) | 当前实现 vs 目标的完整差距分析 | ✅ 新增 |

### 二、基础设施层

| 文档 | 对应 Phase | 说明 | 状态 |
|------|-----------|------|------|
| [网关服务需求文档](./requirements/gateway-service.md) | Phase 1 | Gateway 连接管理、编解码、路由 | ✅ 新增 |
| [协议兼容层需求文档](./requirements/protocol-compat.md) | Phase 7 | proto2 extensions 兼容、帧编解码 | ✅ 新增 |
| [静态配置系统需求文档](./requirements/static-config.md) | Phase 9 | MySQL s_ 表加载、热加载 | ✅ 新增 |
| [数据持久化需求文档](./requirements/persistence.md) | 补充 | 玩家数据存取、异步存盘、dirty 标记 | ✅ 新增 |

### 三、核心服务层

| 文档 | 对应 Phase | 说明 | 状态 |
|------|-----------|------|------|
| [认证服务需求文档](./requirements/auth-service.md) | Phase 2 | 登录验证、渠道适配、Session 管理 | ✅ 新增 |
| [Home 服务需求文档](./requirements/home-service.md) | Phase 3 | PlayerActor、功能系统框架 | ✅ 新增 |
| [World 服务需求文档](./requirements/world-service.md) | Phase 4 | 大地图、分区 Actor、行军、战斗 | ✅ 新增 |

### 四、业务系统层

| 文档 | 对应 Phase | 说明 | 状态 |
|------|-----------|------|------|
| [事件总线需求文档](./requirements/event-system.md) | Phase 8 | 事件分发、跨 Actor 通信 | ✅ 新增 |
| [活动框架需求文档](./requirements/activity-framework.md) | Phase 6 | 活动生命周期、18 种玩法 | ✅ 新增 |
| [Home 功能系统需求文档](./requirements/home-systems.md) | Phase 10 | 将领、背包、科技、装备等 | ✅ 新增 |
| [World 功能系统需求文档](./requirements/world-systems.md) | Phase 10 | 地图实体、采集、NPC、阵营 | ✅ 新增 |

### 五、高级系统层

| 文档 | 对应 Phase | 说明 | 状态 |
|------|-----------|------|------|
| [战斗引擎需求文档](./requirements/battle-engine.md) | Phase 10 | 回合制战斗、战报、伤害计算 | ✅ 新增 |
| [跨服系统需求文档](./requirements/cross-server.md) | Phase 10 | 跨服匹配、跨服战斗 | ✅ 新增 |
| [运维与可观测性需求文档](./requirements/observability.md) | Phase 4 P2 | Metrics、健康检查、日志 | ✅ 新增 |
| [安全与反作弊需求文档](./requirements/security.md) | 补充 | 协议加密、频率限制、数据校验 | ✅ 新增 |

### 六、已有设计文档（rust-rewrite 目录）

| 文档 | 说明 |
|------|------|
| [phase4-world-robustness.md](../rust-rewrite/phase4-world-robustness.md) | World 大地图改进 + 健壮性设计 |
| [phase4-p2-implementation-plan.md](../rust-rewrite/phase4-p2-implementation-plan.md) | P2 阶段实现步骤（WAL、熔断、Metrics、分片） |
| [phase6-activity-framework.md](../rust-rewrite/phase6-activity-framework.md) | 活动大框架设计 |
| [phase6-implementation-plan.md](../rust-rewrite/phase6-implementation-plan.md) | 活动框架实现步骤拆解 |
| [phase7-protocol-compat.md](../rust-rewrite/phase7-protocol-compat.md) | 协议兼容层设计 |
| [phase8-event-system.md](../rust-rewrite/phase8-event-system.md) | 事件总线设计 |
| [phase9-static-config.md](../rust-rewrite/phase9-static-config.md) | 静态配置加载设计 |
| [phase10-remaining-systems.md](../rust-rewrite/phase10-remaining-systems.md) | 剩余功能系统清单 |

---

## 建议阅读顺序

1. 先看 **项目现状与差距分析** 了解全局
2. 按 Phase 顺序阅读各需求文档
3. 对照 rust-rewrite 目录的设计文档理解技术方案
