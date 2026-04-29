# 运维与可观测性需求文档

> 对应 Phase：Phase 4 P2 + 补充
> 优先级：P2
> 预估工期：1-2 周

---

## 一、概述

可观测性系统为游戏服务端提供监控、告警和诊断能力，确保线上问题能被快速发现和定位。包括 Metrics 指标、健康检查、结构化日志和 GM 工具。

---

## 二、功能需求

### 2.1 Metrics 指标

#### FR-OB-001：Prometheus 指标导出
- 通过 HTTP `/metrics` 端点导出 Prometheus 格式指标
- 每个服务（Gateway、Auth、Home、World）独立导出
- 指标命名规范：`slg_{service}_{metric_name}`

#### FR-OB-002：Gateway 指标
| 指标 | 类型 | 说明 |
|------|------|------|
| slg_gateway_connections_active | Gauge | 当前活跃连接数 |
| slg_gateway_connections_total | Counter | 累计连接数 |
| slg_gateway_messages_received_total | Counter | 收到的消息总数 |
| slg_gateway_messages_sent_total | Counter | 发送的消息总数 |
| slg_gateway_message_latency_seconds | Histogram | 消息处理延迟 |
| slg_gateway_route_errors_total | Counter | 路由错误数 |

#### FR-OB-003：Home 指标
| 指标 | 类型 | 说明 |
|------|------|------|
| slg_home_players_online | Gauge | 在线玩家数 |
| slg_home_actor_count | Gauge | 活跃 Actor 数 |
| slg_home_command_latency_seconds | Histogram | 命令处理延迟（按 cmd 分组） |
| slg_home_save_duration_seconds | Histogram | 存盘耗时 |
| slg_home_save_errors_total | Counter | 存盘失败数 |
| slg_home_events_dispatched_total | Counter | 事件分发数 |

#### FR-OB-004：World 指标
| 指标 | 类型 | 说明 |
|------|------|------|
| slg_world_sector_queue_length | Gauge | 各 Sector 消息队列长度 |
| slg_world_marching_troops | Gauge | 行军中的部队数 |
| slg_world_tick_duration_seconds | Histogram | tick 处理耗时 |
| slg_world_battles_total | Counter | 战斗触发数 |
| slg_world_sector_messages_total | Counter | Sector 处理的消息数 |

### 2.2 健康检查

#### FR-OB-005：服务健康检查
- HTTP `/health` 端点返回服务健康状态
- 检查项：数据库连接、Redis 连接、各 Actor 心跳
- 返回格式：`{ "status": "healthy", "checks": {...} }`

#### FR-OB-006：Actor 心跳检测
- 每个 Actor 在 tick 时上报心跳
- 超过阈值（默认 30 秒）未上报视为卡死
- 卡死 Actor 触发告警和自动重启

#### FR-OB-007：依赖服务检查
- 定期检查 MySQL 连接可用性
- 定期检查 Redis 连接可用性（如果使用）
- 定期检查 gRPC 服务间连通性

### 2.3 结构化日志

#### FR-OB-008：日志规范
- 使用 `tracing` 框架
- 日志级别：ERROR、WARN、INFO、DEBUG、TRACE
- 结构化字段：role_id、cmd、sector_id、duration_ms
- 日志格式：JSON（生产）/ 人类可读（开发）

#### FR-OB-009：关键日志点
| 场景 | 级别 | 必含字段 |
|------|------|---------|
| 玩家登录/下线 | INFO | role_id, account_id |
| 命令处理 | DEBUG | role_id, cmd, duration_ms |
| 存盘 | INFO | role_id, dirty_modules, duration_ms |
| 战斗触发 | INFO | attacker_id, defender_id, battle_type |
| Actor panic | ERROR | actor_id, panic_info |
| 存盘失败 | ERROR | role_id, error |
| 配置热加载 | INFO | reload_duration_ms |

#### FR-OB-010：日志采集
- 日志输出到 stdout（容器化部署）
- 支持输出到文件（传统部署）
- 日志轮转：按大小或时间轮转

### 2.4 GM 工具

#### FR-OB-011：GM 命令
| 命令 | 说明 |
|------|------|
| reload_config | 触发配置热加载 |
| kick_player {role_id} | 踢玩家下线 |
| ban_account {account_id} | 封禁账号 |
| add_item {role_id} {item_id} {count} | 给玩家添加道具 |
| add_resource {role_id} {type} {amount} | 给玩家添加资源 |
| set_level {role_id} {level} | 设置玩家等级 |
| server_status | 查看服务器状态 |
| player_info {role_id} | 查看玩家信息 |

#### FR-OB-012：GM 接口
- 通过 HTTP API 提供 GM 功能
- 需要鉴权（GM token）
- 操作日志记录（谁在什么时间执行了什么操作）

### 2.5 告警

#### FR-OB-013：告警规则
| 条件 | 级别 | 说明 |
|------|------|------|
| 在线人数突降 50% | 严重 | 可能服务异常 |
| 存盘失败率 > 1% | 严重 | 数据库可能异常 |
| Actor 卡死数 > 0 | 警告 | 需要排查 |
| 消息队列积压 > 1000 | 警告 | 可能过载 |
| 内存使用 > 80% | 警告 | 需要关注 |
| tick 超时 > 100ms | 警告 | 性能问题 |

---

## 三、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| Prometheus 指标 | ✅ World 基础 | Gateway/Home 缺失 |
| 健康检查 | ✅ World 基础 | 其他服务缺失 |
| 结构化日志 | ⚠️ 基础 tracing | 需要规范化 |
| GM 工具 | ❌ 缺失 | 需要实现 |
| 告警 | ❌ 缺失 | 需要实现 |

---

## 四、配置项

```toml
[observability]
metrics_bind_addr = "0.0.0.0:9090"
health_check_interval_secs = 10
actor_heartbeat_timeout_secs = 30

[observability.log]
level = "info"
format = "json"          # "json" 或 "pretty"
output = "stdout"        # "stdout" 或 "file"
file_path = "./logs/slg.log"
max_file_size_mb = 100
max_files = 10

[observability.gm]
enabled = true
bind_addr = "0.0.0.0:8080"
auth_token = "your-gm-token"
```
