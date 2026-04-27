# Phase 4 World Robustness 实现步骤拆解 (P2 阶)

在完成了 P0 与 P1 阶段的基础设施、分区 Actor 搭建、崩溃隔离及服务弹性处理之后，我们开始着手 **P2 的高级健壮性扩展**。以下是针对 P2 的四个功能点的设计任务拆分。

## User Review Required

请您评估这四大 P2 特性的优先级，并依然可以使用我为您预留的 **Gemini Flash 提示词** 直接发给大模型进行初始代码生成。

## Proposed Changes

### [Stage 7] WAL (Write-Ahead Log) 关键状态持久化
**目标**：当前内存里的跨区行军、派兵等关键业务，如果在存盘周期抵达前发生物理机关机往往会造成数据丢失。WAL 的作用是将关键命令立即以 `Append-Only` 的形式写入磁盘后再向客户端宣告成功。

#### [NEW] src/world/wal.rs
- **实现细节**:
  - 定义 `WalEntry` 枚举，比如包含 `MarchStart`, `BattleSettle`, `ResourceChange` 等变动。
  - 实现 `WriteAheadLog` 结构体，内部持有 `tokio::fs::File`，提供 `append` (追加写入) 与 `recover` (节点启动时重放) 功能。
  - 在 `MapSectorActor` 处理对应 `SectorMessage` 时，先 `append` 到本地 WAL 再进行内存调度。
- **Gemini Flash 提示词**:
  > "根据我们先前实现的 Rust Actor 游戏底座，现在我要实现 phase 4 文档的 3.7 节内容：WAL 预写日志持久化。请帮我编写 `src/world/wal.rs`。需求是定义 `WalEntry` 泛型或 enum（用来记录如：发兵、战斗），并实现 `WriteAheadLog` 结构，提供基于 tokio 异步文件 IO 的追加写入 (`append`) 以及能够在进程拉起时读取的重新放映 (`recover`) 功能。"

---

### [Stage 8] gRPC 客户端熔断器 (Circuit Breaker)
**目标**：P1 中通过 tower 我们实现了超时和上限控制，P2 中我们将添加更纯粹的熔断器逻辑（闭合、断开、半开状态），防止雪崩请求压垮远端服务或阻塞本地线程资源。

#### [NEW] src/world/circuit_breaker.rs
#### [MODIFY] src/world/rpc_client.rs
- **实现细节**:
  - 实现一个维护三态（Closed 正常请求, Open 拒绝请求并立即报错短路, Half-Open 漏桶试探）的状态机。
  - 包装在现有的 gRPC Request 被调用前；记录错误次数，当超过阈值，直接将后方请求抛弃，直至超时进入 Half-Open 开始试发。
- **Gemini Flash 提示词**:
  > "在此前使用 tower 中间件搭好的 Rust gRPC 客户端弹性的基础上，我要进一步实现真正的熔断器（Circuit Breaker）。请展示一个完整的熔断状态机（Closed, Open, Half-Open）的控制层，要求能统计最近 N 次的调用失败率，超过阈值则立即拦截掉外部请求，并在指定恢复时间后放出单次试探。尽量保持无状态调用或利用已有的社区轻量级库思路。"

---

### [Stage 9] 可观测性系统：Metrics 与 Prometheus
**目标**：没有观测就没有运维。我们需要对 Actor 的心跳、各分区的计算负载、行军队列长度进行监控，并暴露 HTTP 服务格式让外部采数。

#### [NEW] src/world/metrics.rs
#### [MODIFY] src/world/sector_actor.rs
- **实现细节**:
  - 引入社区常用的 `metrics` 核心与 `metrics-exporter-prometheus` crate。
  - 导出指标：`sector_actor_queue_length`（队列积压），`active_march_count` (当前大地图行进的队伍总数) 等。
  - 依托 tokio 启动 HTTP Server，暴露 `/metrics` 接口点供 Prometheus 拉取抓抓。
- **Gemini Flash 提示词**:
  > "请为我的 Rust Tokio 游戏服务端编写一套 Metrics 可观测监控方案（对应 phase4 的 3.8 节）。我想要基于能够对接 Prometheus 的 crate（比如 metrics-exporter-prometheus），请提供初始化的全套代码，并展示如何在 `MapSectorActor` 系统里，动态上报：1. 每个 Actor 处理的队列时长或积压数量 2. 本服务器活跃部队的总量 等仪表打点（Gauge/Counter）代码片段。"

---

### [Stage 10] 分布式分片路由 (Cluster Sharding)
**目标**：为了突破单机的内存和算力限制，World Server 需要演化为集群。Home 或 Gateway 服必须明确请求该发往哪个 World 节点。

#### [NEW] src/world/router.rs (或概念文档)
- **实现细节**:
  - 大地图的二维平面如果无限扩展，网关层需要有一个简易的路由表 (如 Consistence Hashing 或依 `SectorID % NodeNum`)。
  - 提供计算：根据目的地的 X, Y 计算 Sector，再由 Sector ID 决定 gRPC 的远端 Node 地址。
- **Gemini Flash 提示词**:
  > "我们在架构图中提到了要实现对标 Akka Cluster Sharding 的机制。请帮我用 Rust 编写一个简易的网关层路由表设计 `Router`。输入要求是：大地图的目标网格坐标或者是发往哪个特定的 Sector 分区。该结构要维护一张哈希环（Consistent Hashing），并在接收请求时返回指定物理节点 gRPC Client 实例 的引用，解决分布式部署的定位问题。"

## Verification Plan
1. **WAL 重放**：构造一条虚假派兵记录强行结束 World 服务器进程，验证再次启动时，内存 TimerWheel 重新接管到了那条部队行进倒计时。
2. **熔断**：切断被调用服务，World 断言能够捕获熔断异常日志且后续请求 `is_circuit_open == true`，防止雪崩。
3. **Prometheus 打点**：通过 `curl http://127.0.0.1:9000/metrics` 直观地在控制台查阅导出的 Prometheus 标准监控文本。
