# Phase 9：静态配置加载系统（Static Config）执行指南

本文件旨在指导你一步步在 `slg-rs` 项目中实现基于 MySQL 的游戏静态配置（s_ 表）的加载机制与热更新订阅体系。

**目标**：为各游戏逻辑功能提供全局只读的配置内存快照，并利用 `tokio::sync::watch` 实现配置热重载时 Actor 得以自动感知和刷新。

请按照以下分步策略进行推进。每完成一步，需要 `cargo check` 确认无误后再进行下一步。由于目前的数据库表环境尚未就绪，我们首先搭建空壳与基础结构。

---

## Step 1: 在 shared crate 中搭建配置加载基础框架

我们将在 `shared` 库中统一管理全局配置的生命周期。

1. 新建并注册相关的目录及模块：
   - 在 `crates/shared/src` 下创建 `static_config` 目录。
   - 创建 `crates/shared/src/static_config/mod.rs` 和 `crates/shared/src/static_config/activity.rs`。
   - 在 `crates/shared/src/lib.rs` 中注册 `pub mod static_config;`。
2. 在 `activity.rs` 中定义活动配置相关的模拟结构体：
   ```rust
   use std::collections::HashMap;

   #[derive(Debug, Clone, Default)]
   pub struct StaticActivityPlan {
       pub activity_id: i32,
       pub open_duration: i64,
       pub display_duration: i64,
       pub form_ids: Vec<i32>,
   }

   #[derive(Debug, Clone, Default)]
   pub struct ActivityConfig {
       pub plans: HashMap<i32, StaticActivityPlan>,
   }

   impl ActivityConfig {
       pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
           // TODO: 使用 sqlx::query_as 真实加载 s_activity_plan 表。当前返回空壳：
           Ok(Self::default())
       }
   }
   ```
3. 在 `mod.rs` 中定义 `StaticConfig` 顶级容器和热签发器 `ConfigWatcher`：
   ```rust
   pub mod activity;
   use activity::ActivityConfig;
   use std::sync::Arc;
   use tokio::sync::watch;

   #[derive(Debug, Clone, Default)]
   pub struct StaticConfig {
       pub activity: ActivityConfig,
       // 未来扩展 hero, building 等等
   }

   impl StaticConfig {
       pub async fn load_from_db(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
           let activity = ActivityConfig::load(pool).await?;
           Ok(Self { activity })
       }
   }

   /// 包装了配置与 watch Channel 发送端的全局单例
   pub struct ConfigWatcher {
       pub db: sqlx::MySqlPool,
       pub tx: watch::Sender<Arc<StaticConfig>>,
   }

   impl ConfigWatcher {
       pub async fn new(db: sqlx::MySqlPool) -> anyhow::Result<(Self, watch::Receiver<Arc<StaticConfig>>)> {
           let initial_config = StaticConfig::load_from_db(&db).await?;
           let (tx, rx) = watch::channel(Arc::new(initial_config));
           Ok((Self { db, tx }, rx))
       }

       pub async fn reload(&self) -> anyhow::Result<()> {
           let new_config = StaticConfig::load_from_db(&self.db).await?;
           let _ = self.tx.send(Arc::new(new_config));
           Ok(())
       }
   }
   ```

---

## Step 2: 更新 Home 启动流程 (main.rs)

将 `ConfigWatcher` 织入系统。

1. 在 `crates/home/src/main.rs` 中，紧跟数据库初始化 `let db = init_mysql(...).await?` 后，添加对静态配置的初始化：
   ```rust
   // 初始化静态配置及 Watcher
   let (config_watcher, config_rx) = shared::static_config::ConfigWatcher::new(db.clone()).await?;
   let config_watcher = Arc::new(config_watcher);
   ```
   **注意**：这要求上文的 `event_bus` 优化后一并通过所谓的 `AppContext` 注入，或者我们这里直接把 `config_rx` 往下传。为简便起见，这里请将 `config_rx` 作为参数传入 `ActivityActor::new()` 以及 `PlayerManager::new()` 中。

---

## Step 3: 更新 PlayerActor 集成配置热加载

对 `crates/home/src/actors/player_actor.rs` 进行修改以监听热更。

1. 在 `PlayerActor` 结构体中添加：
   ```rust
   use shared::static_config::StaticConfig;
   use tokio::sync::watch;
   use std::sync::Arc;

   // 内部扩充
   config_rx: watch::Receiver<Arc<StaticConfig>>,
   current_config: Arc<StaticConfig>,
   ```
2. 更改构造函数 `PlayerActor::new` 的参数，接收 `config_rx`，并用其完成初始化。
3. 在 `PlayerActor::run` 的 `tokio::select!` 中新增分支订阅配置变化：
   ```rust
   Ok(()) = self.config_rx.changed() => {
       tracing::info!("PlayerActor {} received static config reload", self.role_id);
       let new_config = self.config_rx.borrow().clone();
       self.current_config = new_config;
       // 这里如果日后需要将新配置传递给 ActivitySystem 等进行内存重建，可在此操作
   }
   ```

---

## Step 4: 更新 ActivityActor 集成配置热加载

与 PlayerActor 类似，全服 `ActivityActor` (`crates/home/src/actors/activity_actor.rs`) 往往更需要静态配置去启动全服定时活动。

1. 修改 `ActivityActor` 结构体与构造函数，接入 `config_rx: watch::Receiver<Arc<StaticConfig>>`，并保存 `current_config`。
2. 同样为其 `run` 函数的 `tokio::select!` 增加分支以实现热更新刷新：
   ```rust
   Ok(()) = self.config_rx.changed() => {
       tracing::info!("ActivityActor received static config reload");
       self.current_config = self.config_rx.borrow().clone();
   }
   ```

---

## Step 5: 验证并构建

执行以下命令确认框架已闭环跑通并不存在任何类型兼容错误：

1. `cargo check -p shared`
2. `cargo check -p home@0.1.0`

> 核心思路在于，通过引用计数的全局共享 `Arc<StaticConfig>` 和 `tokio::sync::watch` 通道，你可以使得在任何配置更新指令触发时（例如由GM工具引发 `config_watcher.reload().await`），所有挂历的在线玩家 Actors 和独立统筹 Actor 能够瞬间并在自己的上下文中安全地进行配置的无锁切换。
