# Phase 8：事件总线系统（EventBus）Task List

> 执行前提：已完成 Phase 3/6 相关内容，代码可通过 `cargo check`

---

## Step 1：事件类型定义（`shared` crate）

### 1.1 新建 `crates/shared/src/event.rs`

在此文件中定义以下内容：

- [ ] 定义 `MissionType` 枚举（对应 Java 的任务类型）
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub enum MissionType {
      KillEnemy,
      GatherResource,
      BuildBuilding,
      TrainTroop,
      LoginGame,
      UseItem,
      // ... 补充其他类型
      Unknown,
  }
  ```

- [ ] 定义 `MissionEvent` 结构体
  ```rust
  #[derive(Debug, Clone)]
  pub struct MissionEvent {
      pub role_id: i64,
      pub mission_type: MissionType,
      pub params: Vec<i64>,  // 参数（如击杀数量等）
  }
  ```

- [ ] 定义 `ActivityTriggerEvent` 结构体
  ```rust
  #[derive(Debug, Clone)]
  pub struct ActivityTriggerEvent {
      pub role_id: i64,
      pub trigger_type: i32,
      pub params: Vec<i64>,
  }
  ```

- [ ] 定义 `GameEvent` 枚举（PlayerActor 内部同步事件）
  ```rust
  #[derive(Debug, Clone)]
  pub enum GameEvent {
      Mission(MissionEvent),
      PlayerLogin { role_id: i64 },
      PlayerLogout { role_id: i64 },
      ActivityTrigger(ActivityTriggerEvent),
      ResourceChange { resource_type: i32, delta: i64 },
      BattleEnd { role_id: i64, win: bool, enemy_power: i64 },
  }
  ```

- [ ] 定义 `GlobalEvent` 枚举（跨 Actor 异步事件）
  ```rust
  #[derive(Debug, Clone)]
  pub enum GlobalEvent {
      WorldMilestoneMission { role_id: i64, mission_type: MissionType, params: Vec<i64> },
      CampMilestoneMission { role_id: i64, camp_id: i32, mission_type: MissionType, params: Vec<i64> },
      RankUpdate { rank_type: i32, role_id: i64, value: i64 },
  }
  ```

- [ ] 定义 `PlayerContext` 结构体（事件处理器调用时的上下文）
  ```rust
  /// 供 EventHandler::handle 访问 PlayerActor 当前状态的上下文
  pub struct PlayerContext {
      pub role_id: i64,
      pub account_id: i64,
      // 需要 Handler 访问的跨系统数据可以在这里追加
  }
  ```

- [ ] 定义 `EventHandler` trait
  ```rust
  pub trait EventHandler: Send {
      /// 判断该 Handler 是否对此事件感兴趣
      fn interested_in(&self, event: &GameEvent) -> bool;
      /// 处理事件（同步，PlayerActor 线程内运行）
      fn handle(&mut self, event: &GameEvent, ctx: &mut PlayerContext);
  }
  ```

- [ ] 实现 `EventDispatcher` 结构体
  ```rust
  pub struct EventDispatcher {
      handlers: Vec<Box<dyn EventHandler>>,
  }

  impl EventDispatcher {
      pub fn new() -> Self { Self { handlers: Vec::new() } }
      pub fn register(&mut self, handler: Box<dyn EventHandler>) {
          self.handlers.push(handler);
      }
      pub fn dispatch(&mut self, event: &GameEvent, ctx: &mut PlayerContext) {
          for handler in &mut self.handlers {
              if handler.interested_in(event) {
                  handler.handle(event, ctx);
              }
          }
      }
  }
  ```

### 1.2 将 `event` 模块注册到 `crates/shared/src/lib.rs`

- [ ] 在 `lib.rs` 中追加：
  ```rust
  pub mod event;
  ```

---

## Step 2：`ActivitySystem` 实现 `EventHandler`

> 文件：`crates/home/src/systems/activity/mod.rs`

- [ ] 为 `ActivitySystem` 实现 `shared::event::EventHandler` trait
  - `interested_in`：返回 `true` 当事件是 `GameEvent::Mission` 或 `GameEvent::ActivityTrigger`
  - `handle`：
    - 收到 `GameEvent::Mission` → 调用 `self.on_mission_event(mission_event)`
    - 收到 `GameEvent::ActivityTrigger` → 调用 `self.on_activity_trigger(trigger_event)`

- [ ] 在 `ActivitySystem` 上新增以下方法（内部逻辑可先留 `TODO`）：
  ```rust
  fn on_mission_event(&mut self, event: &MissionEvent) {
      // TODO: 遍历活动，更新任务玩法（TaskForm）进度
  }

  fn on_activity_trigger(&mut self, event: &ActivityTriggerEvent) {
      // TODO: 根据触发类型，激活/更新活动状态
  }
  ```

- [ ] 新增 `event_handler` 工厂方法，把 `ActivitySystem` 本身包装为 `Box<dyn EventHandler>`：
  ```rust
  // 注意：由于所有权，返回 handler 需配合 Arc<Mutex<ActivitySystem>> 或传 clone
  // 推荐方案：ActivitySystem 直接实现 EventHandler，PlayerActor 通过引用调用
  // （见 Step 3 详细说明）
  ```

---

## Step 3：`PlayerActor` 集成 `EventDispatcher`

> 文件：`crates/home/src/actors/player_actor.rs`

- [ ] 在 `PlayerActor` 结构体中新增字段：
  ```rust
  pub event_dispatcher: shared::event::EventDispatcher,
  pub ctx: shared::event::PlayerContext,
  ```

- [ ] 在 `PlayerActor::new()` 中初始化：
  ```rust
  event_dispatcher: EventDispatcher::new(),
  ctx: PlayerContext { role_id, account_id },
  ```

- [ ] 新增 `setup_event_handlers` 方法，在 Actor 启动时注册各系统 Handler：

  > **实现策略**：由于 `ActivitySystem` 已经在 `PlayerActor` 中以 owned 形式存在，  
  > 推荐**不使用 Box<dyn EventHandler>**，而是在 `dispatch` 时直接调用各 system 的 `handle_event` 方法，  
  > 以避免所有权/借用冲突。  
  > 如果需要动态注册，可以改为 `Arc<Mutex<ActivitySystem>>`（Phase 后续优化点）。
  
  简化版（推荐先实现）：
  ```rust
  pub fn dispatch_event(&mut self, event: &GameEvent) {
      // 直接分发给各系统，无需 trait object
      self.activity_system.handle_event(event, &mut self.ctx);
      // self.mission_system.handle_event(event, &mut self.ctx); // 未来
  }
  ```

- [ ] 在登录处理 `handle_role_login` 中，触发 `PlayerLogin` 事件：
  ```rust
  let event = GameEvent::PlayerLogin { role_id: self.role_id };
  self.dispatch_event(&event);
  ```

- [ ] 在 `PlayerMessage` 枚举中新增：
  ```rust
  /// 游戏内行为事件（击杀、建造等）
  DispatchGameEvent(GameEvent),
  ```

- [ ] 在 `PlayerActor::run()` 的 `match msg` 中处理新消息：
  ```rust
  PlayerMessage::DispatchGameEvent(event) => {
      self.dispatch_event(&event);
  }
  ```

---

## Step 4：`GlobalEventBus` 实现（跨 Actor 全服事件）

> 新建文件：`crates/home/src/actors/global_event_bus.rs`

- [ ] 定义 `GlobalEventBus` 结构体：
  ```rust
  use shared::event::GlobalEvent;
  use tokio::sync::mpsc;

  #[derive(Clone)]
  pub struct GlobalEventBus {
      activity_tx: mpsc::Sender<GlobalEvent>,
      // milestone_tx: mpsc::Sender<GlobalEvent>,  // 未来扩展
  }

  impl GlobalEventBus {
      pub fn new(activity_tx: mpsc::Sender<GlobalEvent>) -> Self {
          Self { activity_tx }
      }

      /// 发布全服事件（fire-and-forget）
      pub fn publish(&self, event: GlobalEvent) {
          let tx = self.activity_tx.clone();
          tokio::spawn(async move {
              let _ = tx.send(event).await;
          });
      }
  }
  ```

- [ ] 在 `crates/home/src/actors/mod.rs` 中 pub 注册：
  ```rust
  pub mod global_event_bus;
  ```

---

## Step 5：`ActivityActor` 响应 `GlobalEvent`

> 文件：`crates/home/src/actors/activity_actor.rs`

- [ ] 在 `ActivityMessage` 枚举中新增：
  ```rust
  /// 全服事件转发
  GlobalEventReceived(shared::event::GlobalEvent),
  ```

- [ ] 在 `ActivityActor` 结构体中新增 GlobalEvent 接收通道：
  ```rust
  global_event_rx: mpsc::Receiver<shared::event::GlobalEvent>,
  ```

- [ ] 更新 `ActivityActor::new()` 接受 `global_event_rx` 参数

- [ ] 在 `ActivityActor::run()` 的 `tokio::select!` 中新增分支：
  ```rust
  Some(event) = self.global_event_rx.recv() => {
      self.handle_global_event(event).await;
  }
  ```

- [ ] 实现 `handle_global_event` 方法：
  ```rust
  async fn handle_global_event(&mut self, event: GlobalEvent) {
      match event {
          GlobalEvent::WorldMilestoneMission { role_id, mission_type, params } => {
              // TODO: 更新全服里程碑进度
          }
          GlobalEvent::CampMilestoneMission { role_id, camp_id, mission_type, params } => {
              // TODO: 更新阵营里程碑进度
          }
          GlobalEvent::RankUpdate { rank_type, role_id, value } => {
              // TODO: 更新排行榜
          }
      }
  }
  ```

---

## Step 6：Wire Up（Main 启动集成）

> 文件：`crates/home/src/main.rs` 或 `service.rs`

- [ ] 启动时创建 `GlobalEvent` channel：
  ```rust
  let (global_event_tx, global_event_rx) = mpsc::channel::<GlobalEvent>(1024);
  ```

- [ ] 创建 `GlobalEventBus` 实例并注入到 `PlayerActor`

- [ ] 将 `global_event_rx` 传递给 `ActivityActor`

- [ ] 在 `PlayerActor` 结构体中新增 `global_event_bus: GlobalEventBus` 字段（用于向全服投递事件）

---

## Step 7：验证与编译

- [ ] 运行 `cargo check -p home` 确保无编译错误
- [ ] 运行 `cargo check -p shared` 确保 event 模块导出正常
- [ ] （可选）补充基础单测：
  - `EventDispatcher` 分发测试：注册一个简单 Handler，验证收到事件后被调用
  - `GlobalEventBus::publish` 发送测试：验证消息投递到 channel 成功

---

## 实现顺序（推荐）

```
Step 1 → Step 2 → Step 3 → cargo check → Step 4 → Step 5 → Step 6 → cargo check
```

## 关键设计决策（执行时参考）

| 场景 | 方案 |
|------|------|
| PlayerActor 内部事件分发 | 同步调用，不用 `Box<dyn EventHandler>`，直接调用 `system.handle_event()` |
| 跨 Actor 全服事件 | `tokio::mpsc::channel<GlobalEvent>`，fire-and-forget |
| 事件定义位置 | `shared::event` 模块，所有 crate 共享 |
| Handler 上下文传递 | `PlayerContext` 结构体（在 `shared::event` 中定义） |
