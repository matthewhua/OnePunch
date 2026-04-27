# Phase 4 补充：World Service 大地图改进 + 系统健壮性设计

> 状态：设计阶段
> 前置依赖：Phase 1（基础设施）、Phase 3（Home Service 基础框架）
> 说明：本文档分析当前 World Service 的不足，提出大地图改进方案，并系统性地回答"Rust 重写如何保证健壮性"

---

## 一、当前 World Service 差距分析

### 1.1 已有 vs 缺失

| 模块 | 当前状态 | 缺失/问题 |
|------|---------|-----------|
| MapGrid | 有基础格子存储 | 无实体类型区分、无地图初始化加载、无持久化 |
| AoiManager | 有订阅/取消 | 无事件广播机制、无视野变更推送、无与 Gateway 的连接 |
| MarchingManager | 有 tick 到达检测 | 无战斗触发、无采集逻辑、无召回、无加速、用 DashMap 遍历性能差 |
| MapSectorActor | **完全缺失** | 没有分区 Actor，当前是全局共享状态 |
| 部队系统 | 只有 BaseTroop proto | 无部队属性计算、无编队校验、无部队上限 |
| 战斗触发 | **完全缺失** | 无 PvP/PvE 战斗触发和结算 |
| 采集系统 | **完全缺失** | 无资源点管理、采集进度、采集完成 |
| NPC 系统 | **完全缺失** | 无 NPC 刷新、据点、野怪 |
| 城池实体 | **完全缺失** | 玩家城池在地图上的表现 |
| 阵营系统 | **完全缺失** | 阵营领地、阵营战 |
| 联盟系统 | **完全缺失** | 集结、联盟领地 |
| 侦查系统 | **完全缺失** | 侦查行军、侦查报告 |
| 迷雾系统 | **完全缺失** | 战争迷雾、视野解锁 |

### 1.2 架构层面的问题

当前 world crate 有几个根本性的架构问题：

1. **无 Actor 隔离**：`MapGrid`、`AoiManager`、`MarchingManager` 都是全局共享的 `Arc<DashMap>`，高并发下会有锁竞争
2. **无故障隔离**：一个 panic 会影响整个 World Service
3. **无背压机制**：没有消息队列限流，大量行军到达时可能雪崩
4. **无优雅降级**：没有超时、重试、熔断机制
5. **行军 tick 用 DashMap.retain() 全表扫描**：O(n) 复杂度，万人同屏时性能灾难

---

## 二、大地图架构改进方案

### 2.1 MapSectorActor 分区 Actor 模型

将世界地图按区域划分为多个独立的 Actor，每个 Actor 管理一块区域，彼此隔离：

```
世界地图 1300x1300
├── Sector(0,0)  管理 [0,0] ~ [325,325] 区域
├── Sector(1,0)  管理 [326,0] ~ [650,325] 区域
├── Sector(0,1)  管理 [0,326] ~ [325,650] 区域
├── ...
└── Sector(3,3)  管理 [976,976] ~ [1300,1300] 区域

每个 Sector 是一个独立的 tokio task：
- 拥有自己的 mpsc::Receiver<SectorMessage>
- 拥有该区域内的所有格子数据（独占，无锁）
- 拥有该区域内的所有部队（独占，无锁）
- 200ms tick 驱动行军、战斗、采集
```

```rust
pub struct MapSectorActor {
    sector_id: (u32, u32),
    /// 该分区内的格子数据（独占所有权，无需锁）
    tiles: HashMap<i32, MapTile>,
    /// 该分区内的行军部队（用时间轮替代 HashMap 遍历）
    march_wheel: TimerWheel<TroopId>,
    /// 该分区内的采集部队
    gathering: HashMap<TroopId, GatheringState>,
    /// 消息接收
    rx: mpsc::Receiver<SectorMessage>,
    /// 邻居 Sector 的发送端（跨区行军转移）
    neighbors: HashMap<(u32, u32), mpsc::Sender<SectorMessage>>,
    /// Home Service 通知通道
    home_notify: mpsc::Sender<WorldToHomeEvent>,
    /// AOI 广播通道
    aoi_broadcast: mpsc::Sender<AoiBroadcast>,
}
```

### 2.2 时间轮替代全表扫描

当前 `MarchingManager.tick()` 用 `DashMap.retain()` 遍历所有部队检查到达时间，这是 O(n)。

改用分层时间轮（Hierarchical Timer Wheel）：

```rust
/// 分层时间轮：O(1) 插入，O(1) tick
/// 适合大量定时事件（行军到达、采集完成、buff 过期等）
pub struct TimerWheel<T> {
    /// 粗粒度轮（1 秒一格，60 格 = 1 分钟）
    seconds: [Vec<TimerEntry<T>>; 60],
    /// 细粒度轮（100ms 一格，10 格 = 1 秒）
    ticks: [Vec<TimerEntry<T>>; 10],
    /// 溢出桶（超过 1 分钟的事件）
    overflow: BTreeMap<i64, Vec<TimerEntry<T>>>,
    current_tick: u64,
}

impl<T> TimerWheel<T> {
    /// 插入定时事件 O(1)
    pub fn schedule(&mut self, deadline_ms: i64, data: T) { /* ... */ }
    /// tick 推进，返回到期事件 O(1) 均摊
    pub fn advance(&mut self) -> Vec<T> { /* ... */ }
}
```

### 2.3 AOI 事件广播改进

当前 AOI 只有订阅管理，没有实际的事件广播。需要：

```rust
/// AOI 事件类型
pub enum AoiEvent {
    /// 实体进入视野
    EntityEnter { entity: MapEntitySnapshot },
    /// 实体离开视野
    EntityLeave { entity_id: i64, pos: i32 },
    /// 实体状态更新（行军位置插值、采集进度等）
    EntityUpdate { entity_id: i64, updates: Vec<FieldUpdate> },
    /// 部队出发
    MarchStart { troop: TroopSnapshot },
    /// 部队到达
    MarchArrive { troop_id: i64, pos: i32 },
    /// 战斗发生
    BattleOccurred { pos: i32, attacker_id: i64, defender_id: i64 },
}

/// AOI 广播器：负责将 Sector 内的事件推送给关注该区域的玩家
pub struct AoiBroadcaster {
    /// Grid → 订阅该 Grid 的玩家连接（Gateway 的推送通道）
    subscriptions: HashMap<i32, Vec<mpsc::Sender<AoiEvent>>>,
}
```

### 2.4 跨区行军

部队从 Sector A 行军到 Sector B 时，需要跨 Actor 转移：

```rust
/// 跨区行军流程：
/// 1. Sector A 检测到部队即将离开本区域
/// 2. Sector A 发送 TransferTroop 消息给 Sector B
/// 3. Sector B 接收部队，继续行军 tick
/// 4. Sector A 删除该部队

pub enum SectorMessage {
    /// 玩家命令（派兵、召回等）
    PlayerCommand { role_id: i64, cmd: u32, payload: Bytes, reply: oneshot::Sender<Result<Bytes>> },
    /// 跨区部队转移
    TransferTroop { troop: Troop },
    /// tick 驱动
    Tick,
    /// 配置热加载
    ConfigReload(Arc<StaticConfig>),
}
```

---

## 三、Rust 系统健壮性设计（对标 Akka Cluster）

Java 的 Akka Cluster 提供了：监督策略（Supervision）、故障隔离、位置透明、集群分片。Rust 没有现成的 Akka，但可以通过以下机制达到同等甚至更强的健壮性。

### 3.1 对照表：Akka Cluster vs Rust 方案

| Akka Cluster 能力 | Rust 对应方案 | 说明 |
|-------------------|-------------|------|
| Supervision（监督树） | tokio JoinSet + 自动重启 | Actor panic 后自动重新 spawn |
| Let-it-crash 哲学 | catch_unwind + Actor 重启 | Rust panic 可以被捕获，不会崩整个进程 |
| 故障隔离 | 每个 Actor 独立 tokio task | 一个 task panic 不影响其他 task |
| 背压（Backpressure） | bounded mpsc channel | 消息队列满时自动背压 |
| 位置透明 | tonic gRPC | 进程内 channel / 跨进程 gRPC 统一接口 |
| Cluster Sharding | 自建分片路由 | role_id % N 路由到对应 Home 节点 |
| Cluster Singleton | 分布式锁 / Leader 选举 | etcd lease 或 Redis RedLock |
| Persistence（事件溯源） | WAL + 快照 | 可选，用于关键状态恢复 |
| Dead Letter | 死信队列 | 消息发送失败时的兜底处理 |
| Circuit Breaker | tower 中间件 | gRPC 调用的熔断器 |

### 3.2 Actor 监督与自动重启

```rust
/// Actor 监督器：管理一组 Actor 的生命周期
/// 对标 Akka 的 SupervisorStrategy
pub struct ActorSupervisor {
    /// 被监督的 Actor 集合
    actors: JoinSet<ActorResult>,
    /// Actor 工厂（用于重启）
    factories: HashMap<ActorId, Box<dyn ActorFactory>>,
    /// 重启策略
    restart_policy: RestartPolicy,
    /// 重启计数（用于检测频繁崩溃）
    restart_counts: HashMap<ActorId, RestartCounter>,
}

pub enum RestartPolicy {
    /// 总是重启（对标 Akka OneForOneStrategy + Restart）
    Always { max_retries: u32, within: Duration },
    /// 指数退避重启
    ExponentialBackoff { initial: Duration, max: Duration, max_retries: u32 },
    /// 不重启，记录错误
    Stop,
}

impl ActorSupervisor {
    pub async fn run(mut self) {
        loop {
            // JoinSet 会在任何一个 task 完成时返回
            match self.actors.join_next().await {
                Some(Ok(ActorResult::Completed(id))) => {
                    tracing::info!("Actor {:?} 正常退出", id);
                }
                Some(Ok(ActorResult::Failed(id, err))) => {
                    tracing::error!("Actor {:?} 异常退出: {}", id, err);
                    self.maybe_restart(id).await;
                }
                Some(Err(join_err)) => {
                    // task panic 被 JoinSet 捕获
                    if join_err.is_panic() {
                        tracing::error!("Actor panic: {:?}", join_err);
                        // 从 panic 信息中提取 ActorId，尝试重启
                        self.handle_panic(join_err).await;
                    }
                }
                None => break, // 所有 Actor 都退出了
            }
        }
    }

    async fn maybe_restart(&mut self, id: ActorId) {
        let counter = self.restart_counts.entry(id).or_default();
        if counter.should_restart(&self.restart_policy) {
            tracing::warn!("重启 Actor {:?}（第 {} 次）", id, counter.count);
            if let Some(factory) = self.factories.get(&id) {
                let actor = factory.create();
                self.actors.spawn(actor.run());
                counter.record_restart();
            }
        } else {
            tracing::error!("Actor {:?} 重启次数超限，放弃重启", id);
        }
    }
}
```

### 3.3 PlayerActor 的 panic 安全

```rust
impl PlayerActor {
    pub async fn run(mut self) -> ActorResult {
        let role_id = self.role_id;

        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    // 用 catch_unwind 包裹消息处理，防止单条消息 panic 杀死整个 Actor
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        self.handle_message(msg)
                    }));

                    match result {
                        Ok(Ok(())) => {},
                        Ok(Err(e)) => {
                            // 业务错误，记录日志，继续运行
                            tracing::warn!("玩家 {} 消息处理错误: {}", role_id, e);
                        }
                        Err(panic_info) => {
                            // panic！紧急存盘，然后让监督器重启
                            tracing::error!("玩家 {} 消息处理 panic: {:?}", role_id, panic_info);
                            self.emergency_save().await;
                            return ActorResult::Failed(
                                ActorId::Player(role_id),
                                anyhow::anyhow!("panic in message handler"),
                            );
                        }
                    }
                }
                // ... tick, save 等
            }
        }
    }
}
```

### 3.4 背压与过载保护

```rust
/// 使用 bounded channel 实现背压
/// 当 Actor 处理不过来时，发送方会被阻塞（或返回 TrySendError）
pub fn create_actor_channel() -> (mpsc::Sender<PlayerMessage>, mpsc::Receiver<PlayerMessage>) {
    // 缓冲区 256 条消息，超过后发送方背压
    mpsc::channel(256)
}

/// Gateway 侧的过载保护
impl Router {
    pub async fn route(&self, session: &Session, cmd: u32, payload: Bytes) -> Result<Bytes> {
        let tx = self.get_actor_sender(session.role_id)?;

        // 带超时的发送，防止 Actor 卡死导致 Gateway 线程阻塞
        match tokio::time::timeout(Duration::from_secs(5), tx.send(msg)).await {
            Ok(Ok(())) => { /* 等待响应 */ }
            Ok(Err(_)) => {
                // Actor 已关闭（channel closed）
                Err(GameError::PlayerOffline)
            }
            Err(_) => {
                // 超时，Actor 可能过载
                tracing::warn!("玩家 {} Actor 消息队列满，超时", session.role_id);
                Err(GameError::ServerBusy)
            }
        }
    }
}
```

### 3.5 优雅关闭（Graceful Shutdown）

```rust
/// 全局关闭信号
pub struct ShutdownCoordinator {
    /// 关闭信号广播
    shutdown_tx: broadcast::Sender<()>,
    /// 等待所有 Actor 完成
    actor_tracker: Arc<AtomicUsize>,
}

impl ShutdownCoordinator {
    /// 触发优雅关闭
    pub async fn shutdown(&self, timeout: Duration) {
        tracing::info!("开始优雅关闭...");

        // 1. 广播关闭信号
        let _ = self.shutdown_tx.send(());

        // 2. 停止接受新连接（Gateway）
        // 3. 等待所有在线玩家存盘
        let deadline = Instant::now() + timeout;
        while self.actor_tracker.load(Ordering::Relaxed) > 0 {
            if Instant::now() > deadline {
                tracing::warn!("优雅关闭超时，强制退出。剩余 {} 个 Actor",
                    self.actor_tracker.load(Ordering::Relaxed));
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        tracing::info!("优雅关闭完成");
    }
}

/// 每个 Actor 监听关闭信号
impl PlayerActor {
    pub async fn run(mut self) {
        let mut shutdown_rx = self.shutdown_rx.subscribe();

        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => { /* 处理消息 */ }
                _ = shutdown_rx.recv() => {
                    tracing::info!("玩家 {} 收到关闭信号，开始存盘", self.role_id);
                    self.save_all().await;
                    break;
                }
            }
        }
    }
}
```

### 3.6 跨服务调用的熔断与重试

```rust
/// 使用 tower 中间件实现熔断器（对标 Akka 的 CircuitBreaker）
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;

/// Home → World 的 gRPC 调用，加上熔断和超时
pub fn build_world_client(channel: Channel) -> WorldServiceClient<impl Service> {
    let svc = ServiceBuilder::new()
        // 超时：单次调用最多 3 秒
        .layer(TimeoutLayer::new(Duration::from_secs(3)))
        // 并发限制：最多 1000 个并发请求
        .layer(ConcurrencyLimitLayer::new(1000))
        // 重试：网络错误重试 2 次
        .layer(RetryLayer::new(RetryPolicy::new(2)))
        .service(channel);

    WorldServiceClient::new(svc)
}

/// 自定义重试策略
pub struct RetryPolicy {
    max_retries: u32,
}

impl tower::retry::Policy<Request, Response, Status> for RetryPolicy {
    fn retry(&self, _req: &Request, result: Result<&Response, &Status>) -> Option<Self> {
        match result {
            Err(status) if status.code() == Code::Unavailable => {
                // 服务不可用，重试
                Some(Self { max_retries: self.max_retries - 1 })
            }
            _ => None, // 其他情况不重试
        }
    }
}
```

### 3.7 持久化安全：WAL + 快照

```rust
/// 关键状态的 Write-Ahead Log
/// 防止 Actor 崩溃时丢失未存盘的数据
pub struct WriteAheadLog {
    /// WAL 文件（append-only）
    file: tokio::fs::File,
    /// 序列号
    sequence: u64,
}

impl WriteAheadLog {
    /// 在执行关键操作前，先写 WAL
    pub async fn append(&mut self, entry: WalEntry) -> Result<u64> {
        let seq = self.sequence;
        self.sequence += 1;
        let data = bincode::serialize(&entry)?;
        self.file.write_all(&data).await?;
        self.file.flush().await?;
        Ok(seq)
    }

    /// Actor 重启时，从 WAL 恢复未完成的操作
    pub async fn recover(&mut self) -> Result<Vec<WalEntry>> {
        // 读取 WAL 文件，返回未确认的条目
        todo!()
    }
}

/// WAL 条目
#[derive(Serialize, Deserialize)]
pub enum WalEntry {
    /// 行军发起（防止发兵后 crash 导致部队丢失）
    MarchStart { troop: Troop },
    /// 战斗结算（防止结算中途 crash 导致奖励丢失）
    BattleSettle { report: BattleReport },
    /// 资源变更
    ResourceChange { role_id: i64, changes: Vec<(i32, i64)> },
}
```

### 3.8 健康检查与可观测性

```rust
/// 健康检查端点（对标 Akka Cluster 的 health check）
pub struct HealthChecker {
    /// 各 Actor 的最后心跳时间
    heartbeats: DashMap<ActorId, Instant>,
    /// 告警阈值
    stale_threshold: Duration,
}

impl HealthChecker {
    /// 定期检查，发现卡死的 Actor
    pub fn check(&self) -> HealthReport {
        let now = Instant::now();
        let mut stale_actors = Vec::new();

        for entry in self.heartbeats.iter() {
            if now.duration_since(*entry.value()) > self.stale_threshold {
                stale_actors.push(entry.key().clone());
            }
        }

        HealthReport {
            total_actors: self.heartbeats.len(),
            stale_actors,
            // 加上内存、CPU、消息队列深度等指标
        }
    }
}

/// 每个 Actor 在 tick 时上报心跳
impl PlayerActor {
    fn tick(&mut self) {
        self.health_checker.heartbeat(ActorId::Player(self.role_id));
        // ... 其他 tick 逻辑
    }
}
```

---

## 四、Rust 相比 Java/Akka 的天然优势

除了上面需要手动搭建的机制，Rust 还有一些 Java/Akka 做不到的天然优势：

| 维度 | Java + Akka | Rust |
|------|------------|------|
| 内存安全 | GC 管理，但有 NPE 风险 | 编译期保证无空指针、无数据竞争 |
| 并发安全 | 靠 Akka Actor 隔离，但 Actor 内部仍可能有竞态 | 所有权系统编译期禁止数据竞争 |
| GC 停顿 | CMS/G1/ZGC 仍有停顿 | 零 GC，延迟完全可预测 |
| 资源泄漏 | 需要手动 close，容易忘 | RAII 自动释放，编译期保证 |
| 类型安全 | 运行时 ClassCastException | 编译期类型检查，enum 穷举匹配 |
| 错误处理 | try-catch 可能遗漏 | Result<T, E> 强制处理，编译器不让你忽略 |

### 4.1 Rust 的 enum 穷举 = 不会遗漏分支

```rust
// Rust 编译器强制你处理所有消息类型，漏一个就编译不过
match msg {
    SectorMessage::PlayerCommand { .. } => { /* ... */ }
    SectorMessage::TransferTroop { .. } => { /* ... */ }
    SectorMessage::Tick => { /* ... */ }
    SectorMessage::ConfigReload(_) => { /* ... */ }
    // 如果新增了一种消息类型但忘了处理，编译直接报错
}
```

### 4.2 所有权系统 = 编译期消除数据竞争

```rust
// 每个 SectorActor 独占自己的 tiles 数据
// 编译器保证不会有两个 Actor 同时修改同一块地图数据
// 这在 Java 中需要靠 Akka 的 Actor 隔离 + 开发者自觉，Rust 是编译器强制的
pub struct MapSectorActor {
    tiles: HashMap<i32, MapTile>,  // 独占所有权，无需 Arc/Mutex
}
```

---

## 五、总结：Rust 健壮性保证的完整体系

```
┌─────────────────────────────────────────────────────────────────┐
│                    编译期保证（Rust 独有）                        │
│  ✓ 无空指针    ✓ 无数据竞争    ✓ 无内存泄漏    ✓ 穷举匹配       │
├─────────────────────────────────────────────────────────────────┤
│                    Actor 隔离层                                  │
│  ✓ 每个 Actor 独立 tokio task                                   │
│  ✓ 通过 channel 通信，无共享可变状态                              │
│  ✓ 一个 Actor panic 不影响其他 Actor                             │
├─────────────────────────────────────────────────────────────────┤
│                    监督与恢复层                                   │
│  ✓ ActorSupervisor 自动重启崩溃的 Actor                         │
│  ✓ catch_unwind 捕获 panic，紧急存盘后重启                       │
│  ✓ WAL 保证关键操作不丢失                                       │
│  ✓ 指数退避防止频繁重启                                          │
├─────────────────────────────────────────────────────────────────┤
│                    流量控制层                                     │
│  ✓ bounded channel 背压                                         │
│  ✓ 超时保护（防止 Actor 卡死阻塞调用方）                          │
│  ✓ 并发限制（防止雪崩）                                          │
│  ✓ 熔断器（跨服务调用失败时快速失败）                              │
├─────────────────────────────────────────────────────────────────┤
│                    运维保障层                                     │
│  ✓ 优雅关闭（全员存盘后退出）                                    │
│  ✓ 健康检查（检测卡死 Actor）                                    │
│  ✓ 结构化日志（tracing）                                        │
│  ✓ Metrics 指标（Prometheus）                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 六、实现优先级

### P0（必须有，否则上线就炸）
1. MapSectorActor 分区隔离
2. bounded channel 背压
3. Actor panic catch_unwind + 自动重启
4. 优雅关闭
5. 时间轮替代全表扫描

### P1（上线前应该有）
6. 跨服务调用超时 + 重试
7. 健康检查
8. AOI 事件广播
9. 跨区行军转移

### P2（可以后续迭代）
10. WAL 持久化
11. 熔断器
12. Metrics / Prometheus
13. 分布式分片（多 World 节点）
