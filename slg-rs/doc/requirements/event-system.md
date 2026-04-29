# 事件总线系统需求文档

> 对应 Phase：Phase 8
> 优先级：P1（跨系统联动的基础）
> 预估工期：1-2 周

---

## 一、概述

事件总线是各业务系统之间解耦通信的核心机制。玩家的一个行为（如击杀敌人）可能同时影响主线任务进度、活动积分、成就解锁、里程碑等多个系统。对应 Java 版的 `EventBus + @Subscribe` 机制。

---

## 二、功能需求

### 2.1 Actor 内部事件（同步）

#### FR-EV-001：GameEvent 枚举
```rust
pub enum GameEvent {
    // 任务相关
    Mission(MissionEvent),
    // 玩家生命周期
    PlayerLogin { role_id: i64 },
    PlayerLogout { role_id: i64 },
    PlayerLevelUp { role_id: i64, new_level: i32 },
    // 资源变更
    ResourceChange { resource_type: i32, delta: i64, source: &'static str },
    // 战斗
    BattleEnd { role_id: i64, win: bool, enemy_power: i64, battle_type: i32 },
    // 建筑
    BuildingUpgrade { building_type: i32, new_level: i32 },
    BuildingComplete { building_type: i32, level: i32 },
    // 科技
    TechResearchComplete { tech_id: i32, level: i32 },
    // 将领
    HeroLevelUp { hero_id: i32, new_level: i32 },
    HeroStarUp { hero_id: i32, new_star: i32 },
    // 装备
    EquipForge { equip_id: i32, quality: i32 },
    // 活动触发
    ActivityTrigger(ActivityTriggerEvent),
    // 充值
    Recharge { amount: i64, product_id: String },
    // VIP
    VipLevelUp { new_level: i32 },
}
```

#### FR-EV-002：MissionType 枚举
```rust
pub enum MissionType {
    // 战斗类
    KillEnemy,              // 击杀敌人
    WinBattle,              // 赢得战斗
    AttackPlayer,           // 攻击玩家
    AttackNpc,              // 攻击 NPC
    DefendCity,             // 防守城池
    // 建设类
    BuildBuilding,          // 建造建筑
    UpgradeBuilding,        // 升级建筑
    ResearchTech,           // 研究科技
    TrainTroop,             // 训练部队
    // 资源类
    GatherResource,         // 采集资源
    ProduceResource,        // 生产资源
    SpendResource,          // 消耗资源
    // 将领类
    LevelUpHero,            // 升级将领
    StarUpHero,             // 升星将领
    RecruitHero,            // 招募将领
    // 社交类
    JoinAlliance,           // 加入联盟
    DonateAlliance,         // 联盟捐献
    HelpAlly,               // 帮助盟友
    // 活动类
    LoginGame,              // 登录游戏
    UseItem,                // 使用道具
    Recharge,               // 充值
    // 世界类
    SendMarch,              // 派兵出征
    ScoutEnemy,             // 侦查敌人
    OccupyResourcePoint,    // 占领资源点
    // ... 更多类型（Java 版有 50+ 种）
}
```

#### FR-EV-003：EventDispatcher（Actor 内部同步分发）
- 在 PlayerActor 内部同步分发事件
- 各 PlayerSystem 直接响应事件（无需跨线程）
- 分发顺序：按注册顺序依次调用
- 单个 handler 异常不影响其他 handler

#### FR-EV-004：事件处理器注册
- 每个 PlayerSystem 可以注册对特定事件的处理
- 推荐方式：PlayerActor 直接调用各 System 的 handle_event 方法
- 备选方式：通过 EventHandler trait 动态注册

### 2.2 跨 Actor 事件（异步）

#### FR-EV-005：GlobalEventBus
- 用于 PlayerActor → ActivityActor / MilestoneActor 等全服 Actor 的通信
- 通过 `mpsc::Sender<GlobalEvent>` 异步投递
- 优先使用 `try_send` 避免阻塞，队列满时降级为 `tokio::spawn`

#### FR-EV-006：GlobalEvent 枚举
```rust
pub enum GlobalEvent {
    // 里程碑全服任务
    WorldMilestoneMission { role_id: i64, mission_type: MissionType, params: Vec<i64> },
    // 里程碑阵营任务
    CampMilestoneMission { role_id: i64, camp_id: i32, mission_type: MissionType, params: Vec<i64> },
    // 排行榜更新
    RankUpdate { rank_type: i32, role_id: i64, value: i64 },
    // 活动积分更新
    ActivityScoreUpdate { activity_id: i32, form_id: i32, role_id: i64, delta: i64 },
    // 全服公告
    ServerAnnouncement { content: String },
}
```

#### FR-EV-007：事件转发规则
- 任务事件（MissionEvent）在 Actor 内部同步分发给：
  - MissionSystem（主线/日常/成长任务）
  - ActivitySystem（活动任务/积分）
- 需要全服处理的事件通过 GlobalEventBus 异步投递给：
  - ActivityActor（排行榜更新）
  - MilestoneActor（里程碑进度）

### 2.3 事件驱动的业务联动

#### FR-EV-008：任务进度联动
- 玩家完成某个行为 → 触发 MissionEvent
- MissionSystem 检查所有活跃任务，更新匹配的任务进度
- 任务完成时触发奖励发放

#### FR-EV-009：活动积分联动
- 玩家完成某个行为 → 触发 MissionEvent
- ActivitySystem 检查所有活跃活动的任务型玩法
- 更新活动任务进度或累计积分

#### FR-EV-010：里程碑联动
- 特定行为（如全服击杀 NPC 总数）需要汇总到全服
- PlayerActor 通过 GlobalEventBus 投递给 MilestoneActor
- MilestoneActor 汇总后检查里程碑是否达成

---

## 三、非功能需求

### 3.1 性能
- Actor 内部事件分发延迟 < 1μs（同步调用）
- GlobalEventBus 投递延迟 < 100μs
- 单个事件最多触发 20 个 handler

### 3.2 可靠性
- 单个 handler 异常不影响其他 handler
- GlobalEventBus 队列满时不丢失事件（降级为异步等待）

---

## 四、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| GameEvent 枚举 | ✅ 基础 | 需要补充更多事件类型 |
| MissionType 枚举 | ⚠️ 部分 | 仅 6 种，需要补全 50+ |
| EventDispatcher | ✅ 基础 | 可用 |
| GlobalEventBus | ✅ 基础 | 需要 try_send 优化 |
| PlayerActor 集成 | ✅ 基础 | dispatch_event 已实现 |
| 任务进度联动 | ❌ 缺失 | MissionSystem 未实现 |
| 活动积分联动 | ❌ 缺失 | ActivitySystem 事件处理待完善 |
| 里程碑联动 | ❌ 缺失 | MilestoneActor 未实现 |
