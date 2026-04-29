# Home 服务需求文档

> 对应 Phase：Phase 3
> 优先级：P0
> 预估工期：4-6 周

---

## 一、概述

Home Service 管理所有玩家的个人数据，每个在线玩家对应一个 PlayerActor（tokio task）。对应 Java 版的 `Game Server` 中与玩家个人数据相关的部分（`Player` 实体 + `FunctionEntity` + 各 `Handler`/`Service`）。

---

## 二、功能需求

### 2.1 PlayerActor 生命周期

#### FR-HM-001：Actor 创建
- 玩家登录时由 PlayerManager 创建 PlayerActor
- 创建流程：分配 mpsc channel → 从 DB 加载数据 → 反序列化各 System → spawn tokio task
- 创建失败时返回错误给 Gateway，不保留残留资源

#### FR-HM-002：Actor 消息循环
- 使用 `tokio::select!` 多路复用：
  - 玩家消息（mpsc channel）
  - 定时 tick（每秒）
  - 定时存盘（每 5 分钟）
  - 配置热加载（watch channel）
  - 关闭信号（broadcast channel）
- 消息处理使用 `catch_unwind` 包裹，单条消息 panic 不杀死 Actor

#### FR-HM-003：Actor 销毁
- 玩家下线触发销毁流程
- 销毁流程：全量存盘 → 清理资源 → 从 PlayerManager 移除 → task 退出
- 超时保护：存盘超过 10 秒强制退出

### 2.2 PlayerManager

#### FR-HM-004：在线玩家管理
- 维护 `DashMap<i64, PlayerHandle>` 在线玩家表
- PlayerHandle 包含：role_id、mpsc::Sender、创建时间
- 支持查询在线状态、发送消息、强制下线

#### FR-HM-005：踢重复登录
- 同一 role_id 重复登录时，先踢掉旧 Actor
- 踢人流程：发送 Shutdown 消息 → 等待旧 Actor 存盘退出 → 创建新 Actor
- 等待超时后强制创建新 Actor

### 2.3 PlayerSystem 框架

#### FR-HM-006：统一系统接口
```rust
pub trait PlayerSystem: Send + Sync {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()>;
    fn save_to_bin(&self) -> Result<Vec<u8>>;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
    fn column_name(&self) -> &'static str;
    fn tick(&mut self) {}  // 每秒 tick，默认空实现
    fn on_login(&mut self) {}  // 登录时调用
    fn on_logout(&mut self) {}  // 下线时调用
    fn on_daily_reset(&mut self) {}  // 每日重置
}
```

#### FR-HM-007：命令分发
- 根据 cmd 号将消息路由到对应的 PlayerSystem
- 每个 System 实现 `handle_command(cmd, payload) -> Result<Vec<u8>>`
- 未知命令返回错误码

### 2.4 定时任务

#### FR-HM-008：每秒 Tick
- 驱动各系统的定时逻辑（建筑升级倒计时、科技研究倒计时等）
- 检查 buff 过期
- 更新在线时长

#### FR-HM-009：每日重置
- 检测跨天（基于服务器时区）
- 触发各系统的每日重置逻辑
- 重置日常任务、商店刷新、签到状态等

#### FR-HM-010：每周重置
- 检测跨周
- 重置周常任务、竞技场赛季等

### 2.5 gRPC 服务接口

#### FR-HM-011：Home gRPC Service
```protobuf
service HomeService {
    // Gateway 转发的玩家命令
    rpc Dispatch(DispatchRq) returns (DispatchRs);
    // 玩家上线
    rpc PlayerOnline(PlayerOnlineRq) returns (PlayerOnlineRs);
    // 玩家下线
    rpc PlayerOffline(PlayerOfflineRq) returns (PlayerOfflineRs);
    // World Service 的通知（战报、部队返回等）
    rpc WorldNotify(WorldNotifyRq) returns (WorldNotifyRs);
    // 全服广播（活动开启、系统公告等）
    rpc Broadcast(BroadcastRq) returns (BroadcastRs);
}
```

---

## 三、非功能需求

### 3.1 性能
- 单 Home 实例支持 5000+ 在线玩家
- 命令处理延迟 < 5ms（p99）
- 每秒 tick 处理时间 < 10ms（所有在线玩家）

### 3.2 可靠性
- Actor panic 后自动重启（从 DB 重新加载数据）
- 存盘失败不影响消息处理
- 优雅关闭时所有在线玩家存盘

---

## 四、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| PlayerActor 消息循环 | ✅ 基础 | 缺少 tick、存盘、catch_unwind |
| PlayerManager | ✅ 基础 | 缺少踢重复登录 |
| PlayerSystem trait | ✅ 定义 | 缺少 dirty、tick、daily_reset |
| 命令分发 | ⚠️ 骨架 | 仅处理 RoleLogin 和 GetRoleData |
| 数据加载/存盘 | ❌ 缺失 | 核心缺失 |
| 每秒 tick | ❌ 缺失 | 需要添加 |
| 每日重置 | ❌ 缺失 | 需要添加 |
| gRPC 服务 | ⚠️ 骨架 | 需要完善 |
| catch_unwind | ❌ 缺失 | 需要添加 |
| 优雅关闭 | ❌ 缺失 | 需要添加 |
