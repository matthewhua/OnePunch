# 网关服务（Gateway Service）需求文档

> 对应 Phase：Phase 1 基础设施
> 优先级：P0
> 预估工期：2-3 周

---

## 一、概述

Gateway 是客户端与后端服务之间的唯一入口，负责连接管理、协议编解码、消息路由和鉴权。对应 Java 版的 `GatewayServer` + `ClientGatewayProxy` + `GameGatewayProxy` + `GatewayRWork`。

---

## 二、功能需求

### 2.1 连接管理

#### FR-GW-001：TCP/TLS 监听
- 支持 TCP 明文连接（开发环境）和 TLS 加密连接（生产环境）
- 每个客户端连接分配一个独立的 tokio task
- 支持配置最大连接数上限（默认 10000）
- 连接建立时记录 peer_addr 和 conn_id

#### FR-GW-002：会话管理
- 维护 `DashMap<u64, Session>` 会话表
- Session 包含：conn_id、account_id、role_id、server_id、state、peer_addr
- 会话状态机：`Connected → Authenticated → InGame → Disconnecting`
- 支持踢重复登录（同一 account_id 只允许一个活跃会话）

#### FR-GW-003：心跳检测
- 客户端每 30 秒发送心跳包（cmd=9001）
- 服务端 60 秒未收到心跳则断开连接
- 心跳超时触发下线流程（通知 Home Service 存盘）

#### FR-GW-004：连接断开处理
- 客户端主动断开：立即触发下线流程
- 网络异常断开：等待 30 秒重连窗口，超时后触发下线
- 下线流程：通知 Home Service 玩家下线 → 等待存盘完成 → 清理 Session

### 2.2 协议编解码

#### FR-GW-005：帧编解码
- 帧格式：`4字节大端总长度 + 4字节大端命令号 + N字节 protobuf body`
- 总长度包含自身 4 字节
- 支持粘包/拆包处理（基于长度头）
- 单帧最大长度限制：64KB（可配置）

#### FR-GW-006：proto2 Extension 兼容
- 解码 Base 消息后，根据 command 提取对应的 extension 字段
- 支持构建带 extension 的 Base 响应消息
- 详见 [协议兼容层需求文档](./protocol-compat.md)

### 2.3 消息路由

#### FR-GW-007：命令号路由
- 根据 cmd 将消息路由到对应的后端服务：
  - Auth 命令（1001-1108）→ Auth Service
  - Home 命令（个人数据相关）→ Home Service
  - World 命令（2000-3999）→ World Service
- 路由表可配置、可热加载

#### FR-GW-008：请求-响应匹配
- 每个请求分配唯一的 request_id
- 响应通过 request_id 匹配回原始连接
- 请求超时处理（默认 10 秒）

### 2.4 鉴权

#### FR-GW-009：首包鉴权
- 连接建立后，首个消息必须是 LoginRq 或 BeginGameRq
- 非鉴权消息在未认证状态下直接拒绝
- 鉴权通过后更新 Session 状态为 Authenticated

#### FR-GW-010：Session Token 校验
- 每个请求携带 session_token
- Gateway 本地缓存 token → account_id 映射（TTL 5 分钟）
- 缓存未命中时调用 Auth Service 验证

### 2.5 流量控制

#### FR-GW-011：连接级限流
- 单连接每秒最多 100 条消息（可配置）
- 超限后返回 ServerBusy 错误码
- 持续超限 10 秒则断开连接

#### FR-GW-012：全局限流
- 全服消息吞吐上限（可配置，默认 500K msg/s）
- 超限时拒绝新连接，已有连接降级处理

---

## 三、非功能需求

### 3.1 性能
- 单 Gateway 实例支持 10000+ 并发连接
- 消息转发延迟 < 1ms（p99）
- 内存占用 < 500MB（10000 连接时）

### 3.2 可用性
- 支持优雅关闭（等待所有连接断开或超时 30 秒）
- 支持热重启（新实例接管新连接，旧实例处理完存量连接后退出）

### 3.3 可观测性
- 暴露 Prometheus 指标：活跃连接数、消息吞吐、路由延迟
- 结构化日志：连接建立/断开、鉴权结果、路由错误

---

## 四、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| TCP 监听 | ✅ 已实现 | - |
| TLS | ❌ 缺失 | 需要添加 tokio-rustls |
| 会话管理 | ❌ 缺失 | 需要实现 Session 结构和状态机 |
| 帧编解码 | ⚠️ 基础 | 需要完善 proto2 extension 处理 |
| 消息路由 | ⚠️ 基础 | 需要补全命令号映射 |
| 心跳检测 | ❌ 缺失 | 需要实现 |
| 限流 | ❌ 缺失 | 需要实现 |
| 踢重复登录 | ❌ 缺失 | 需要实现 |

---

## 五、接口定义

### 5.1 与 Auth Service 的接口
```
gRPC AuthService {
    rpc ValidateToken(ValidateTokenRq) returns (ValidateTokenRs);
    rpc Login(LoginRq) returns (LoginRs);
}
```

### 5.2 与 Home Service 的接口
```
gRPC HomeService {
    rpc Dispatch(DispatchRq) returns (DispatchRs);  // 转发玩家命令
    rpc PlayerOffline(PlayerOfflineRq) returns (PlayerOfflineRs);  // 通知下线
}
```

### 5.3 与 World Service 的接口
```
gRPC WorldService {
    rpc Dispatch(DispatchRq) returns (DispatchRs);  // 转发世界命令
}
```

---

## 六、配置项

```toml
[gateway]
bind_addr = "0.0.0.0:9527"
max_connections = 10000
max_frame_size = 65536
heartbeat_interval_secs = 30
heartbeat_timeout_secs = 60
reconnect_window_secs = 30
rate_limit_per_second = 100

[gateway.tls]
enabled = false
cert_path = "certs/server.crt"
key_path = "certs/server.key"
```
