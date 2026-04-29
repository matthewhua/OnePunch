# Rust SLG 项目现状与差距分析

> 更新日期：2026-04-29
> 目的：全面梳理当前实现进度，识别可优化点和待补充功能

---

## 一、当前实现进度总览

### 1.1 各 Crate 实现状态

| Crate | 模块 | 实现程度 | 说明 |
|-------|------|---------|------|
| **proto** | proto 文件 | ✅ 完整 | 30+ 个 proto 文件已从 Java 迁移 |
| **proto** | build.rs 代码生成 | ✅ 可用 | prost 编译通过 |
| **shared** | config 配置加载 | ⚠️ 骨架 | 仅有 TOML 配置，缺少环境变量覆盖 |
| **shared** | db 数据库连接 | ⚠️ 骨架 | sqlx 连接池基础实现 |
| **shared** | error 错误码 | ⚠️ 骨架 | 缺少完整的游戏业务错误码 |
| **shared** | cmd 命令号 | ⚠️ 部分 | 仅注册了约 15 个命令号，Java 版有 200+ |
| **shared** | event 事件系统 | ✅ 基础 | EventDispatcher + GameEvent 枚举已实现 |
| **shared** | msg 消息封装 | ⚠️ 骨架 | GameMessage 结构定义，extension 解码未完成 |
| **shared** | static_config | ⚠️ 骨架 | 仅有 ActivityConfig 框架，其他配置未实现 |
| **shared** | registry | ❓ 未知 | 需要检查实现程度 |
| **gateway** | TCP 监听 | ✅ 基础 | tokio TcpListener 已实现 |
| **gateway** | 编解码 | ⚠️ 骨架 | 帧格式基础实现，proto2 extension 未处理 |
| **gateway** | 消息路由 | ⚠️ 骨架 | 基础路由逻辑，缺少完整命令号映射 |
| **gateway** | 会话管理 | ❌ 缺失 | 无 Session 结构、无状态机 |
| **gateway** | TLS 加密 | ❌ 缺失 | 未实现 TLS |
| **auth** | 登录验证 | ✅ 基础 | gRPC Login + ValidateToken |
| **auth** | Session 管理 | ⚠️ 内存版 | 内存 HashMap，未接 Redis |
| **auth** | 渠道适配 | ❌ 缺失 | 无 PlatformVerifier trait 实现 |
| **home** | PlayerActor | ✅ 基础 | 消息循环 + RoleLogin + GetRoleData |
| **home** | PlayerManager | ✅ 基础 | spawn_actor + 在线管理 |
| **home** | PlayerSystem trait | ✅ 定义 | load_from_bin / save_to_bin |
| **home** | BuildingSystem | ⚠️ 空壳 | 仅有结构定义 |
| **home** | SkinSystem | ⚠️ 空壳 | 仅有结构定义 |
| **home** | ActivitySystem | ⚠️ 骨架 | types/model/forms 框架已搭建 |
| **home** | ActivityActor | ⚠️ 骨架 | 全服活动 Actor 基础框架 |
| **home** | GlobalEventBus | ✅ 基础 | 全服事件投递已实现 |
| **home** | 数据持久化 | ❌ 缺失 | 无 p_data 表读写、无定时存盘 |
| **world** | MapSectorActor | ✅ 基础 | 分区 Actor + 消息处理 + WAL 恢复 |
| **world** | TimerWheel | ✅ 已实现 | 分层时间轮 |
| **world** | AOI | ⚠️ 骨架 | AoiManager 结构定义，广播逻辑待完善 |
| **world** | MapGrid | ⚠️ 骨架 | 格子存储基础实现 |
| **world** | 行军系统 | ⚠️ 骨架 | march/mod.rs 基础结构 |
| **world** | WAL | ✅ 基础 | 追加写入 + 恢复 + 截断 |
| **world** | Supervisor | ✅ 基础 | Actor 监督与重启 |
| **world** | CircuitBreaker | ✅ 基础 | 三态熔断器 |
| **world** | HealthChecker | ✅ 基础 | 心跳检测 |
| **world** | Metrics | ✅ 基础 | Prometheus 指标导出 |
| **world** | Router | ✅ 基础 | 一致性哈希路由 |
| **world** | Shutdown | ✅ 基础 | 优雅关闭 |

### 1.2 完成度评估

```
基础设施层  ████████░░░░░░░░  ~50%  （Gateway/Auth 骨架可用，缺协议兼容和持久化）
Home 服务   ████░░░░░░░░░░░░  ~25%  （Actor 框架可用，业务系统几乎全空）
World 服务  ██████░░░░░░░░░░  ~40%  （健壮性机制较完善，业务逻辑待填充）
业务系统    ██░░░░░░░░░░░░░░  ~10%  （活动框架骨架，其他系统未开始）
跨服/战斗   ░░░░░░░░░░░░░░░░   0%  （完全未开始）
```

---

## 二、可优化的方向

### 2.1 架构层面优化

| 优化项 | 当前状态 | 建议 | 优先级 |
|--------|---------|------|--------|
| AppContext 依赖注入 | 各组件手动传参 | 引入组合型 AppContext 统一注入 | P1 |
| 错误处理体系 | 混用 anyhow | 定义业务错误枚举 + thiserror | P1 |
| 日志规范化 | 基础 tracing | 统一日志格式、添加 span 追踪 | P2 |
| 配置管理 | 仅 TOML | 支持环境变量覆盖、多环境配置 | P2 |
| 连接池优化 | 默认配置 | 调优连接池参数、添加健康检查 | P2 |
| 内存分配器 | 系统默认 | 考虑 jemalloc/mimalloc 提升性能 | P3 |

### 2.2 功能层面补充

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 协议兼容层 | proto2 extension 编解码是与客户端对接的前提 | **P0** |
| 玩家数据持久化 | 无持久化则所有业务系统无法真正运行 | **P0** |
| 静态配置加载 | 无配置则所有业务逻辑无法驱动 | **P0** |
| Gateway 会话管理 | 完整的连接生命周期管理 | P1 |
| Home 核心系统 | 将领、背包、科技、装备、建筑 | P1 |
| World 核心系统 | 地图实体、行军完善、战斗触发 | P1 |
| 事件系统完善 | MissionType 补全、跨系统联动 | P1 |
| 活动系统完善 | 18 种玩法的具体实现 | P2 |
| 战斗引擎 | 回合制战斗计算 | P2 |
| 跨服系统 | 跨服匹配和战斗 | P3 |
| GM 工具 | 后台管理、数据查询、配置热加载触发 | P2 |
| 安全防护 | 协议加密、频率限制、数据校验 | P2 |

### 2.3 工程质量优化

| 优化项 | 说明 | 优先级 |
|--------|------|--------|
| 单元测试 | 当前几乎无测试覆盖 | P1 |
| 集成测试 | 端到端登录流程测试 | P1 |
| 压力测试 | 并发连接、消息吞吐基准测试 | P2 |
| CI/CD | 自动编译检查、测试、发布 | P2 |
| 文档注释 | 公共 API 的 rustdoc 注释 | P2 |
| 代码审查清单 | 统一的代码规范和审查标准 | P3 |

---

## 三、建议实施路线

### 第一阶段：打通核心链路（4-6 周）

目标：实现从客户端连接到玩家数据加载的完整流程。

```
协议兼容层（proto2 extension）
    ↓
静态配置加载（s_ 表）
    ↓
玩家数据持久化（p_data 表）
    ↓
Gateway 会话管理完善
    ↓
端到端联调：客户端 → Gateway → Auth → Home → 返回角色数据
```

### 第二阶段：核心业务系统（6-8 周）

目标：实现 SLG 核心玩法。

```
Home 核心系统（将领、背包、科技、装备、建筑）
    ↓
事件系统完善（任务进度联动）
    ↓
World 核心系统（地图实体、行军、战斗触发）
    ↓
战斗引擎（回合制计算）
```

### 第三阶段：运营与扩展（4-6 周）

目标：支撑运营活动和高级玩法。

```
活动框架 18 种玩法实现
    ↓
邮件、聊天、社交系统
    ↓
排行榜、VIP、商店
    ↓
GM 工具与后台管理
```

### 第四阶段：上线准备（3-4 周）

目标：生产环境就绪。

```
安全防护（加密、限流、校验）
    ↓
压力测试与性能调优
    ↓
跨服系统
    ↓
灰度发布与监控告警
```
