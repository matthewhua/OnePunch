# 登录与创角模块技术实现设计文档

本文档基于 `Game.proto` 定义，从现代 Rust 异步微服务架构的最佳实践出发，深入解析 SLG 服务器中的网关与 World 服在“登录到创角、进入游戏”阶段的设计方案。

## 1. 接口与数据流分析

整个登录流分为三个阶段：**鉴权初始化 (`BeginGame`)** -> **创角 (`CreateRole`)** -> **进入游戏 (`RoleLogin`)**。

### 调用链路清晰化 (Client -> Gateway -> World)

1. **`BeginGameRq` (环境与令牌鉴权)**:
   - **调用方式**: Unary (单次请求-响应)。
   - **Gateway 职能**: 客户端首先连接至 Gateway，Gateway 生成一个临时 `SessionId`（代表当前物理连接），解析协议头，进行初步限流防刷。无需解析完整 Payload，或仅做基础解密，将请求附加 `SessionId` 和 `IP` 转发给 World。
   - **World 职能**: 接收到请求后，通过 `keyId` 和 `token` 访问 Redis/Auth服 完成令牌校验。如果校验通过：
     - 若检查到玩家未创角：返回 `state = 1`，并在返回中带入 `repeated string name` 供客户端摇骰子。
     - 若检查到已创角：返回 `state = 2` 和 `roleId`。

2. **`CreateRoleRq` / `GetNamesRq` (创建角色)**:
   - **调用方式**: Unary。
   - **数据流**: Gateway 依据临时 `SessionId` 透传给 World。World 校验名字唯一性（利用 Redis/DB 唯一索引），执行数据库 INSERT，成功后返回 `state = 1`。客户端随后准备进入游戏。

3. **`RoleLoginRq` (正式进入游戏，建立双向/推送流)**:
   - **调用方式**: **Streaming (双向流建立)**。
   - **网关链路跃迁**: 此时 World 服需要从数据库中完整加载玩家数据结构到内存中。
   - **推送通道**: SLG 游戏中频繁的数据下发（如 `SyncResourceChangeInfoRs`, `SyncRoleStatusChangeRs` 等），**采用 Gateway 与 World 之间的长连接双向流（Bidi-Streaming）或 Server Streaming 来维持**。World 将持有针对该 `SessionId` 的下行 Channel 发送端；之后所有针对此 `roleId` 的服务器主动推送，均通过该 Channel 异步推给 Gateway，由 Gateway 封包后转发物理连接。

---

## 2. 核心领域模型 (Domain Model)

在 Rust 中，我们要严格避免类似于 Java 中万物皆面条式嵌套引用的问题，必须迎合生命周期与所有权，坚决避免 `Arc<RwLock<T>>` 嵌套地狱。因此，内存模型设计应围绕 **Actor 模式（消息传递替代共享内存）** 为核心。

```rust
// --- World 服：玩家状态枚举 ---
pub enum PlayerState {
    Authenticating,    // 鉴权中 (BeginGame)
    RoleCreating,      // 创角中 (CreateRole)
    Loading,           // 正在从 DB 加载全量数据 (RoleLogin 阶段)
    InGame,            // 游戏中
    OfflineRetained,   // 离线驻留 (SLG特有，玩家下线地图数据仍在内存)
}

// --- World 服：游戏内该模块的核心实体 ---
/// 每个玩家在被激活后，独占一个 tokio 绿色线程（Task/Actor）。
pub struct PlayerActor {
    pub role_id: i64,
    pub account_id: i64,
    pub state: PlayerState,

    // ECS 或组件化管理：按业务领域拆分数据结构，不用一个巨型 Struct
    pub base_info: ComponentBaseInfo, // 包含昵称(nick)、头像、阵营(camp)等
    pub resources: ComponentResources, // 包含体力(stamina)、资源增量计算逻辑

    // 该玩家对应的 Gateway 推送通道 Sender (不使用锁控制，单向推数据)
    // Gateway 下线时，Receiver 会 Drop，此处发送会返回 Error 便于 World 捕获下线事件
    pub gateway_tx: Option<tokio::sync::mpsc::UnboundedSender<PushPacket>>, 
}

// 定义通过 Channel 传递给 PlayerActor 执行的具体指令
pub enum PlayerMessage {
    ReqCreateRole(CreateRoleRq, tokio::sync::oneshot::Sender<CreateRoleRs>),
    ReqRoleLogin(RoleLoginRq, tokio::sync::oneshot::Sender<RoleLoginRs>),
    // ... 其他业务请求
}
```
**设计哲学**：数据和行为都在 `tokio::task` 的局部上下文中。无论是外部请求（RPC）还是内部定时器更新（如每秒涨木材），都组装成 `PlayerMessage` 通过 `tokio::sync::mpsc::Sender` 发送给该玩家所在的逻辑协程去串行处理。

---

## 3. 状态管理与并发策略 (关键)

**1. 内存常驻策略：随用随取还是常驻？**
- **结论：SLG 必须按集群分布做内存常驻 (Resident In Memory)。**
- 由于 SLG 中玩家下线后，城池依然能够被攻击、侦查、发生资源抢夺，因此 `Player` 实体不能随着 `Session` 断开而从内存彻底释放。通常在 `RoleLogin` 时加载，断开连接时，剥离网络流 `gateway_tx` 并切换为 `OfflineRetained` 状态。数据采用定时 / 脏标记（Dirty Bit）落地数据库即可。

**2. 登录与创角的并发竞争控制（防挤号、并发创角）：**
- **顶层管理 (Session Manager)**：
  在 World 服中，使用 `DashMap<i64, mpsc::Sender<PlayerMessage>>` 来管理 `AccountId / RoleId` 对应的 Actor 地址。
- **同账号并发排队**：如果两个客户端同时对同一账号发 `BeginGame`，它们会在网关分别拿到不同的 `SessionId`。路由到 World 服时，World 服先锁该 `AccountId` 或通过消息路由给已存在的 Actor，使得“顶号/挤号”可以被序列化处理。
- **防重复创角与名字重复**：
  创角的并发不需要在业务内存加全局锁。最佳实践是利用 Redis `SETNX` (针对角色名) 以及数据库层次的唯一索引 (Unique Index on Nickname)。如果多个人同时抢同一个随机名字，其中一个写入成功，其他抛出 Duplicate 错误，捕获后返回告诉客户端重试。

**3. 使用 Actor 模型规避“多只部队同时攻击”的并发问题**：
- World 服中不应有全局大锁。对于地块竞争、城防变更，地块本身（MapCell）或被防守城池的 `PlayerActor` 作为状态持有者。多支部队实际上转化为 `CombatMessage::Attack(部队数据)` 等消息，投递给**防守方**的 Actor 信箱 (Channel)。由该 Actor 逐个消费消息、结算血量。**从而实现全异步完全无锁并发**。

---

## 4. 异常处理边界

设计高可用网关的一个核心要点是：**拦截脏流量，保护 World 服不受底层连接细节困扰**。

### Gateway 层拦截（不需要穿透到 World）
1. **连接建立风暴与 DdoS 防护**：根据 IP 对 TCP 连接进行令牌桶限流。
2. **协议加解密与包体安全**：基于加密机制对消息进行解密处理；若包体序列化失败、或者超过最大长度，Gateway 直接静默 Drop 或断开连接。
3. **极简 Token 初步校验**：对于一些明显的黑产/黑名单/过期 JWT Token，Gateway 直接结合 Redis 挡回，返回协议层约定的统一错误，无需消耗 World 算力。
4. **断连异常 (`SyncConnectBrokenRs`)**：客户端主动/被动断开（心跳超时等），网关自身捕获 TCP FIN 置位后，直接清理本层资源，并通过内部通道给 World 发送断连通知，World 根据此通知执行玩家网络流解绑等下线逻辑。

### World 逻辑层拦截 (通过业务 Rs 的 state 抛出)
1. **防沉迷 / 封禁账号等业务阻断**：`BeginGameRs::state = 3`，扩展参数带回封停时间和原因。这涉及到读取运营黑白名单。
2. **创角逻辑校验**：名字包含特殊字符、涉及敏感词库（通常需要调内部第三方过滤服务），向客户端返回 `CreateRoleRs::state = 0` 及对应错误提示。
3. **合规性/游戏版本一致性检查**：检查 `BeginGameRq.curVersion`，发现客户端版本过旧，在业务中引发强更指令。
4. **底层存储故障**：数据库或 Redis 网络抖动引起的超时或异常，可以转化为错误码抛给客户端，或由 gRPC error 传递回网关由网关统一处理。
