# 数据持久化需求文档

> 优先级：P0（无持久化则所有业务系统无法真正运行）
> 预估工期：1-2 周

---

## 一、概述

玩家数据存储在 MySQL 的 `p_` 前缀表中，核心是 `p_account`（账号表）和 `p_data`（玩家数据表，各功能模块以 blob 字段存储）。对应 Java 版的 `Player` 实体 + `FunctionEntity` 体系 + `PlayerDao`。

---

## 二、功能需求

### 2.1 数据表结构

#### FR-PS-001：p_account 表
```sql
CREATE TABLE p_account (
    account_id BIGINT PRIMARY KEY,
    platform VARCHAR(32),
    plat_account VARCHAR(128),
    server_id INT,
    role_id BIGINT UNIQUE,
    nick VARCHAR(64),
    level INT DEFAULT 1,
    vip_level INT DEFAULT 0,
    camp_id INT DEFAULT 0,
    create_time BIGINT,
    login_time BIGINT,
    logout_time BIGINT,
    status INT DEFAULT 0
);
```

#### FR-PS-002：p_data 表
```sql
CREATE TABLE p_data (
    role_id BIGINT PRIMARY KEY,
    -- 各功能模块的 blob 数据
    building_func BLOB,        -- 建筑系统
    hero_func BLOB,            -- 将领系统
    backpack_func BLOB,        -- 背包系统
    tech_func BLOB,            -- 科技系统
    equip_func BLOB,           -- 装备系统
    mission_func BLOB,         -- 任务系统
    activity_func BLOB,        -- 活动系统
    vip_func BLOB,             -- VIP 系统
    shop_func BLOB,            -- 商店系统
    mail_func BLOB,            -- 邮件系统
    skin_func BLOB,            -- 皮肤系统
    social_func BLOB,          -- 社交系统
    lord_equip_func BLOB,      -- 领主装备
    lord_talent_func BLOB,     -- 领主天赋
    -- 更多功能模块...
    update_time BIGINT
);
```

### 2.2 数据加载

#### FR-PS-003：玩家数据加载
- 玩家登录时从 MySQL 加载 p_account 和 p_data
- 将各 blob 字段反序列化为对应的 PlayerSystem
- 加载失败时返回错误，不创建 PlayerActor
- 支持新玩家创建（p_account 和 p_data 初始化）

#### FR-PS-004：批量加载优化
- 使用单条 SQL 查询加载 p_data 的所有字段（避免多次查询）
- 反序列化在 PlayerActor 的 tokio task 中执行（不阻塞数据库连接）
- 加载超时保护（默认 10 秒）

### 2.3 数据存盘

#### FR-PS-005：定时存盘
- 每 5 分钟自动存盘一次（可配置）
- 仅存盘标记为 dirty 的功能模块（减少 IO）
- 存盘在独立的 tokio task 中异步执行，不阻塞 PlayerActor 消息处理

#### FR-PS-006：下线存盘
- 玩家下线时立即触发全量存盘
- 存盘完成后才释放 PlayerActor 资源
- 存盘超时保护（默认 10 秒），超时后强制释放

#### FR-PS-007：关键操作存盘
- 充值、交易等关键操作后立即存盘
- 使用 dirty 标记 + 立即触发机制

#### FR-PS-008：Dirty 标记机制
- 每个 PlayerSystem 维护 dirty 标志
- 数据变更时设置 dirty = true
- 存盘时仅序列化 dirty 的模块，存盘后重置 dirty = false
- 全量存盘时忽略 dirty 标记

### 2.4 数据安全

#### FR-PS-009：存盘原子性
- 单个玩家的所有 dirty 模块在一个事务中写入
- 事务失败时回滚，下次存盘周期重试
- 连续 3 次失败后告警

#### FR-PS-010：数据版本控制
- p_data 表增加 version 字段（乐观锁）
- 存盘时检查 version，防止并发写入覆盖
- version 冲突时重新加载数据

#### FR-PS-011：紧急存盘
- Actor panic 时触发紧急存盘（catch_unwind 后执行）
- 紧急存盘使用独立的数据库连接（不依赖连接池状态）
- 紧急存盘失败时将数据序列化到本地文件作为兜底

### 2.5 全局数据

#### FR-PS-012：p_global 表
- 全服共享数据（世界地图、活动公共数据、排行榜等）
- 由 ActivityActor / WorldActor 管理
- 定时存盘 + 关闭存盘

---

## 三、非功能需求

### 3.1 性能
- 单玩家数据加载 < 50ms
- 单玩家存盘 < 20ms
- 支持 1000 个玩家同时存盘（批量写入优化）

### 3.2 可靠性
- 存盘成功率 > 99.99%
- 数据丢失窗口 < 5 分钟（定时存盘间隔）

---

## 四、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| p_role 读写 | ✅ 完成 | PlayerDao: CRUD + login/logout 时间更新 |
| p_data 读写 | ✅ 完成 | PlayerDao: load_all / save_data (事务 upsert) |
| p_global 读写 | ✅ 完成 | PlayerDao: load_global / save_global |
| 定时存盘 | ✅ 完成 | PlayerActor 每 5 分钟自动存盘 dirty 模块 |
| Dirty 标记 | ✅ 完成 | PlayerSystem trait: is_dirty / mark_dirty / clear_dirty |
| 下线存盘 | ✅ 完成 | Shutdown 消息触发全量存盘 |
| 关键操作存盘 | ✅ 完成 | ForceSave 消息触发立即存盘 |
| 紧急存盘 | ✅ 完成 | 连续 3 次失败后序列化到本地文件 |
| 存盘超时保护 | ✅ 完成 | 10 秒超时 |
| 新玩家初始化 | ✅ 完成 | init_player_data 批量插入空行 |
| keyId 常量 | ✅ 完成 | 25 个模块 keyId 定义 |
| 踢重复登录 | ✅ 完成 | PlayerManager spawn 时自动踢旧 Actor |
| 优雅关闭 | ✅ 完成 | shutdown_all 通知所有玩家存盘 |
| 数据版本控制 | ❌ 待实现 | 乐观锁（后续优化） |

---

## 五、PlayerSystem trait 扩展

```rust
pub trait PlayerSystem: Send + Sync {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()>;
    fn save_to_bin(&self) -> Result<Vec<u8>>;
    
    // 新增
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
    fn column_name(&self) -> &'static str;  // 对应 p_data 表的列名
}
```

---

## 六、配置项

```toml
[persistence]
save_interval_secs = 300        # 定时存盘间隔
save_timeout_secs = 10          # 存盘超时
max_retry_count = 3             # 存盘失败重试次数
emergency_save_path = "./emergency_saves"  # 紧急存盘目录
```
