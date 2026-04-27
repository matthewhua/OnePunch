# Phase 8：事件总线系统（EventBus）

> 状态：待实现
> 前置依赖：Phase 3（Home Service）、Phase 6（活动框架）
> 预估工期：1-2 周

---

## 一、目标

实现 Rust 版的事件总线系统，替代 Java 版的 `EventBus + @Subscribe` 机制，驱动任务进度、活动积分、里程碑等跨系统联动。

---

## 二、Java 版事件系统分析

### 2.1 事件分发机制

Java 版通过 `EventBus.post()` 发布事件，各 Service 通过 `@Subscribe` 注解订阅：

```java
// 发布事件
BaseMissionEvent.dispatch(player, MissionType.KILL_ENEMY, killCount);

// 内部分发到多个子事件
EventBus.post(new MainMissionEvent(...));      // 主线任务
EventBus.post(new ActivityMissionEvent(...));   // 活动任务
EventBus.post(new CampMissionEvent(...));       // 阵营任务
EventBus.post(new DailyMissionEvent(...));      // 日常任务
EventBus.post(new GrownMissionEvent(...));      // 成长任务
EventBus.post(new MilestoneMissionEvent(...));  // 里程碑任务

// 订阅方
@Subscribe(threadMode = GameThreadMode.PLAYER)
public void onMissionEvent(ActivityMissionEvent event) {
    // 更新活动任务进度 / 积分
}
```

### 2.2 事件类型

| 事件类 | 触发场景 | 订阅方 |
|--------|---------|--------|
| BaseMissionEvent | 任何任务相关行为 | 分发器 |
| MainMissionEvent | 主线任务进度 | MissionFunction |
| ActivityMissionEvent | 活动任务/积分 | ActivityService |
| CampMissionEvent | 阵营任务 | CampService |
| DailyMissionEvent | 日常任务 | MissionFunction |
| GrownMissionEvent | 成长任务 | MissionFunction |
| MilestoneMissionEvent | 里程碑个人任务 | MilestoneService |
| WorldMilestoneMissionEvent | 里程碑全服任务 | MilestoneService（全服） |
| CampMilestoneMissionEvent | 里程碑阵营任务 | MilestoneService（阵营） |
| PlayerLoginEvent | 玩家登录 | 多个系统 |
| ActivityTriggerEvent | 活动触发条件 | CommonActivityActor |

---

## 三、Rust 版设计

### 3.1 编译期类型安全的事件系统

```rust
/// 游戏事件枚举（替代 Java 的事件类继承体系）
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// 任务进度事件（对应 BaseMissionEvent）
    Mission(MissionEvent),
    /// 玩家登录
    PlayerLogin { role_id: i64 },
    /// 玩家下线
    PlayerLogout { role_id: i64 },
    /// 活动触发
    ActivityTrigger(ActivityTriggerEvent),
    /// 资源变更
    ResourceChange { resource_type: i32, delta: i64 },
    /// 战斗结束
    BattleEnd { role_id: i64, win: bool, enemy_power: i64 },
}

#[derive(Debug, Clone)]
pub struct MissionEvent {
    pub role_id: i64,
    pub mission_type: MissionType,
    pub params: Vec<i64>,
}

/// 事件分发器（PlayerActor 内部使用，无需跨线程）
pub struct EventDispatcher {
    /// 订阅者列表：事件类型 → 处理函数
    handlers: Vec<Box<dyn EventHandler>>,
}

pub trait EventHandler: Send {
    fn handle(&mut self, event: &GameEvent, ctx: &mut PlayerContext);
    fn interested_in(&self, event: &GameEvent) -> bool;
}

impl EventDispatcher {
    pub fn dispatch(&mut self, event: &GameEvent, ctx: &mut PlayerContext) {
        for handler in &mut self.handlers {
            if handler.interested_in(event) {
                handler.handle(event, ctx);
            }
        }
    }
}
```

### 3.2 在 PlayerActor 中集成

```rust
impl PlayerActor {
    fn setup_event_handlers(&mut self) {
        // 注册各系统的事件处理器
        self.event_dispatcher.register(self.systems.mission.event_handler());
        self.event_dispatcher.register(self.systems.activity.event_handler());
        // ...
    }
    
    /// 业务逻辑中触发事件
    fn on_kill_enemy(&mut self, kill_count: i64) {
        let event = GameEvent::Mission(MissionEvent {
            role_id: self.role_id,
            mission_type: MissionType::KillEnemy,
            params: vec![kill_count],
        });
        self.event_dispatcher.dispatch(&event, &mut self.ctx);
    }
}
```

### 3.3 跨 Actor 事件（全服事件）

里程碑的全服/阵营任务需要跨 Actor 通信：

```rust
/// 全服事件通道
pub struct GlobalEventBus {
    /// ActivityActor 的发送端
    activity_tx: mpsc::Sender<GlobalEvent>,
    /// MilestoneActor 的发送端（如果独立）
    milestone_tx: mpsc::Sender<GlobalEvent>,
}

pub enum GlobalEvent {
    /// 里程碑全服任务进度
    WorldMilestoneMission { role_id: i64, mission_type: MissionType, params: Vec<i64> },
    /// 里程碑阵营任务进度
    CampMilestoneMission { role_id: i64, camp_id: i32, mission_type: MissionType, params: Vec<i64> },
    /// 排行榜更新
    RankUpdate { rank_type: i32, role_id: i64, value: i64 },
}
```

---

## 四、实现步骤

### Step 1：事件定义（2 天）
- [ ] 定义 GameEvent 枚举和 MissionType 枚举
- [ ] 定义 EventHandler trait
- [ ] 实现 EventDispatcher

### Step 2：PlayerActor 集成（3 天）
- [ ] 在 PlayerActor 中集成 EventDispatcher
- [ ] 各 PlayerSystem 实现 EventHandler
- [ ] 任务系统（MissionSystem）响应 MissionEvent

### Step 3：全服事件（3 天）
- [ ] 实现 GlobalEventBus
- [ ] ActivityActor 响应全服事件
- [ ] 里程碑全服/阵营任务的跨 Actor 通信

---

## 五、给 AI 的实现提示词

```
你是一个 Rust 游戏服务器开发者。请实现事件总线系统。

设计要求：
1. PlayerActor 内部的事件分发是同步的（单线程 Actor 模型，无需锁）
2. 跨 Actor 的全服事件通过 tokio::mpsc channel 异步传递
3. 事件类型使用 enum 而非 trait object，编译期类型安全
4. 各 PlayerSystem 通过实现 EventHandler trait 订阅感兴趣的事件

对应 Java 实现：
- EventBus.post() → EventDispatcher.dispatch()
- @Subscribe → EventHandler trait
- BaseMissionEvent.dispatch() → GameEvent::Mission
- GameThreadMode.PLAYER → PlayerActor 内部同步处理
- 全服事件（WorldMilestoneMissionEvent）→ GlobalEventBus channel
```
