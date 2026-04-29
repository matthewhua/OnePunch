# 认证服务（Auth Service）需求文档

> 对应 Phase：Phase 2
> 优先级：P1
> 预估工期：1-2 周

---

## 一、概述

Auth Service 负责玩家账号的登录验证、渠道适配和 Session 管理。对应 Java 版的 `Center` 服务（`DoLogin` + `AccountService` + `PublisherBase`）。

---

## 二、功能需求

### 2.1 登录验证

#### FR-AU-001：渠道登录验证
- 接收客户端的登录请求（platform、account、token）
- 根据 platform 选择对应的渠道验证器
- 调用渠道 SDK 验证 token 有效性
- 验证通过后查询或创建账号

#### FR-AU-002：渠道适配器（PlatformVerifier）
- 定义 `PlatformVerifier` trait，每个渠道实现一个适配器
- 需要支持的渠道：
  - 内部测试渠道（直接通过，用于开发测试）
  - 微信登录
  - Apple ID 登录
  - Google Play 登录
  - Facebook 登录
  - 自定义渠道（可扩展）
- 渠道验证失败返回明确的错误码

#### FR-AU-003：账号管理
- 查询账号：根据 platform + plat_account 查询 p_account
- 创建账号：首次登录时自动创建账号，分配 account_id
- 账号封禁检查：登录时检查账号状态，封禁账号拒绝登录
- 账号绑定：支持一个渠道账号绑定多个区服角色

### 2.2 Session 管理

#### FR-AU-004：Session Token 生成
- 登录成功后生成 session_token（JWT 或随机字符串）
- Token 包含：account_id、server_id、过期时间
- Token 有效期：24 小时（可配置）

#### FR-AU-005：Session 存储
- 短期方案：内存 HashMap（当前已实现）
- 长期方案：Redis 存储（支持多 Gateway 实例共享）
- Session 数据：account_id、server_id、login_time、last_active_time

#### FR-AU-006：Session 验证
- Gateway 调用 `ValidateToken` 验证 session_token
- 验证通过返回 account_id 和 server_id
- Token 过期或无效返回错误码

#### FR-AU-007：踢人机制
- 同一账号重复登录时，踢掉旧 Session
- 通知 Gateway 断开旧连接
- 通知 Home Service 旧玩家下线存盘

### 2.3 区服管理

#### FR-AU-008：区服列表
- 登录成功后返回可用的区服列表
- 区服信息：server_id、server_name、status（新服/推荐/满员）、open_time
- 标记玩家已有角色的区服

#### FR-AU-009：区服状态
- 维护区服在线人数
- 区服维护状态（维护中不允许登录）
- 区服合服映射

---

## 三、非功能需求

### 3.1 性能
- 登录验证延迟 < 100ms（不含渠道 SDK 调用）
- Session 验证延迟 < 5ms
- 支持 1000 QPS 并发登录

### 3.2 安全
- Token 不可伪造（使用 HMAC 签名或 JWT）
- 防暴力破解（同一 IP 登录频率限制）
- 渠道 token 一次性使用（防重放攻击）

---

## 四、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| gRPC Login | ✅ 基础 | 可用 |
| gRPC ValidateToken | ✅ 基础 | 可用 |
| Session 内存存储 | ✅ 已实现 | 需要迁移到 Redis |
| PlatformVerifier trait | ❌ 缺失 | 需要定义和实现 |
| 渠道适配器 | ❌ 缺失 | 至少需要测试渠道 |
| 账号 MySQL 读写 | ❌ 缺失 | 需要实现 |
| 区服列表 | ❌ 缺失 | 需要实现 |
| 踢人机制 | ❌ 缺失 | 需要实现 |
| JWT Token | ❌ 缺失 | 当前用简单字符串 |

---

## 五、接口定义

```protobuf
service AuthService {
    rpc Login(LoginRq) returns (LoginRs);
    rpc ValidateToken(ValidateTokenRq) returns (ValidateTokenRs);
    rpc GetServerList(GetServerListRq) returns (GetServerListRs);
    rpc KickSession(KickSessionRq) returns (KickSessionRs);
}
```

---

## 六、配置项

```toml
[auth]
token_ttl_hours = 24
max_login_rate_per_ip = 10    # 每 IP 每分钟最大登录次数
session_store = "memory"       # "memory" 或 "redis"

[auth.redis]
url = "redis://127.0.0.1:6379"
session_prefix = "session:"

[auth.platforms]
# 各渠道的 SDK 配置
test.enabled = true
wechat.app_id = ""
wechat.app_secret = ""
```
