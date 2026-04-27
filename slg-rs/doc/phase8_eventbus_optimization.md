# Phase 8：EventBus 事件总线系统后续优化指南

在完成了基础的事件及消息投递机制后，系统已经处于可用状态并通过了编译检查。然而，基于 Rust 和 Async/Await 的并发模型，现有的基础实现在面对游戏上线后的**高并发、高爆发**场景时，有一些架构层面的冗余可以进一步被消除。

以下是针对 Phase 8（事件总线框架）提出的进阶优化和重构建议文档。

---

## 1. GlobalEventBus 高并发投递性能优化

### 当前现状
在 `crates/home/src/actors/global_event_bus.rs` 中，消息的发送逻辑如下：
```rust
pub fn publish(&self, event: GlobalEvent) {
    let tx = self.activity_tx.clone();
    tokio::spawn(async move {
        let _ = tx.send(event).await;
    });
}
```
**问题分析**：
采用 `tokio::spawn` 实现了 fire-and-forget（即用即弃的开辟异步任务）。在常规情况下它运行得很好，但如果在短时间内触发“流量风暴”（例如全服玩家集中上线、攻城战导致短时间内产生几十万次战功统计广播等），底层系统会瞬间创建海量微小的 Tokio 任务，导致运行时调度器资源被大量占用，内存占用飙升。

### 优化方案：采用 `try_send` 退避机制
我们可以优先尝试直接同步无阻塞地将事件塞入 Channel 缓冲区中。只有当队列过满时，才记录告警或退避：

```rust
pub fn publish(&self, event: GlobalEvent) {
    // 优先尝试无重试、无协程地塞入队列
    match self.activity_tx.try_send(event) {
        Ok(_) => {
            // 发送成功，开销极小
        }
        Err(tokio::sync::mpsc::error::TrySendError::Full(event)) => {
            // 队列已满：这时候才降级利用 tokio::spawn 异步等待
            // （也可以在系统中配置丢弃策略，或者监控队列过长的报警）
            tracing::warn!("GlobalEventBus channel is FULL! Spawning async task to wait.");
            let tx = self.activity_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(event).await;
            });
        }
        Err(e) => {
            // 通道关闭等异常
            tracing::error!("GlobalEventBus publish error: {}", e);
        }
    }
}
```

---

## 2. ActivityActor 接收通道结构的合并简化

### 当前现状
在 `crates/home/src/actors/activity_actor.rs` 内部，我们当前维护了两种形式重叠的接收机制：
1. `ActivityMessage::GlobalEventReceived(GlobalEvent)` 枚举（这说明 `ActivityMessage` 主通道本可以承载 `GlobalEvent`）。
2. 在 Actor Struct 和 `run` 的首层又多了一个 `global_event_rx: mpsc::Receiver<GlobalEvent>` 专用通道，并在 `tokio::select!` 中多路复用。

**问题分析**：
“泛类型总通道”和“专用侧信道”混用，增加了并发状态机的心智负担。如果不合并，由于 `tokio::select!` 对待分支是伪随机公平的，会导致当局部消息多时，全局消息和单体消息的优先次序不可控，也使得代码显得冗长。

### 优化方案：合并多向通道为单一 `Receiver`
让 `GlobalEventBus` 持有的不再是原生的 `Sender<GlobalEvent>`，而是持有将其包装好的业务外层代理，使得 `ActivityActor` 重回单通道即可：

**步骤一：将转换层放到外侧**
在 `main.rs` 向 `GlobalEventBus` 传递通道时，不必新建专门的 `GlobalEvent` 通道，而是利用 `mpsc::Sender<ActivityMessage>` 并进行类型降级映射（或在 Bus 内转换）。

**步骤二：裁减掉 `global_event_rx`**
```rust
// activity_actor.rs 优化后：
pub struct ActivityActor {
    global_activities: HashMap<i32, GlobalActivityData>,
    rx: mpsc::Receiver<ActivityMessage>,
    notify_tx: Option<mpsc::Sender<ActivityNotify>>,
    // 移除 global_event_rx
}

impl ActivityActor {
    pub async fn run(mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    // 全局事件、心跳、排位都在主通道处理
                    self.handle_message(msg).await; 
                }
                _ = interval.tick() => {
                    self.on_tick().await;
                }
            }
        }
    }
}
```
这样做能有效减少并发队列争抢，让代码逻辑（尤其是关闭机制、垃圾回收机制）更加健壮统一。

---

## 3. 全局对象及总线的模块化注入注册 (DI/Service Locator)

### 当前现状
在 `crates/home/src/main.rs` 中，事件总线 `event_bus` 经由局部变量分配，然后再一路显式注入到了 `PlayerManager` 和 `PlayerActor`。

```rust
let event_bus = GlobalEventBus::new(...);
let manager = Arc::new(PlayerManager::new(event_bus.clone()));
let actor = PlayerActor::new(..., event_bus.clone());
```
如果在不远的未来，我们的跨模块服务不止事件总线，还有诸如 `WorldMapManager`（大地图管理器）、`GuildManager`（联盟模块）、`ChatRoomManager`（聊天频道）等都需要在 Actor 生命周期中获取，那么 Actor 的初始参数列表会极速膨胀。

### 优化方案：引入组合型的 AppContext 上下文
可以创建一个全局或者局部的只读状态对象，将所有需要穿诱底层的工具集合并。

```rust
// 新建 crates/home/src/context.rs
pub struct AppContext {
    pub event_bus: GlobalEventBus,
    // pub chat_bus: ChatEventBus,
    // pub guild_manager: Arc<GuildManager>,
}

// 在 main.rs 中
let app_ctx = Arc::new(AppContext {
    event_bus,
});

// Actor 和 Manager 仅仅传递 AppContext 即可
let manager = Arc::new(PlayerManager::new(app_ctx.clone()));
let actor = PlayerActor::new(account_id, role_id, rx, self.app_ctx.clone());

// Actor 内部通过 ctx 获取自己关心的组件
self.app_ctx.event_bus.publish(event);
```
这种设计对横向扩展系统十分友好，防止后期每次增加全局功能时都需要改动大量的基础架构相关函数签名。
