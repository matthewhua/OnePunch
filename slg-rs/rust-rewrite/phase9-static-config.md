# Phase 9：静态配置加载系统（Static Config）

> 状态：待实现
> 前置依赖：Phase 1（shared crate）
> 预估工期：1-2 周

---

## 一、目标

实现从 MySQL 加载游戏静态配置（s_ 表）的框架，替代 Java 版的 `StaticDataMgr` 体系。

---

## 二、Java 版配置体系

### 2.1 配置表命名规范

- `s_` 前缀：静态配置表（策划配置，运行时只读）
- `p_` 前缀：玩家数据表（运行时读写）
- `g_` 前缀：全局数据表（运行时读写）

### 2.2 核心配置管理器

| 管理器 | 职责 | 关联表 |
|--------|------|--------|
| StaticActivityConfigMgr | 活动配置 | s_activity_plan, s_activity_form_*, s_activity_cycle |
| StaticHeroMgr | 将领配置 | s_hero, s_hero_skill |
| StaticBuildingMgr | 建筑配置 | s_building |
| StaticTechMgr | 科技配置 | s_technology |
| StaticWorldMgr | 世界配置 | s_world_map, s_npc |
| StaticItemMgr | 道具配置 | s_item |
| StaticMissionMgr | 任务配置 | s_mission, s_chapter_mission |

### 2.3 加载机制

1. 服务启动时从 MySQL 批量加载所有 s_ 表
2. 解析为内存中的 HashMap/TreeMap 结构
3. 支持运行时热加载（`dealAfterConfigReload`）
4. 配置间有依赖关系（如活动配置依赖任务配置）

---

## 三、Rust 版设计

### 3.1 配置加载框架

```rust
/// 静态配置容器（全局只读，Arc 共享）
pub struct StaticConfig {
    pub activity: ActivityConfig,
    pub hero: HeroConfig,
    pub building: BuildingConfig,
    pub tech: TechConfig,
    pub world: WorldConfig,
    pub item: ItemConfig,
    pub mission: MissionConfig,
    // ...
}

impl StaticConfig {
    /// 从 MySQL 加载所有配置
    pub async fn load_from_db(pool: &MySqlPool) -> Result<Self> {
        let activity = ActivityConfig::load(pool).await?;
        let hero = HeroConfig::load(pool).await?;
        // ... 并行加载
        Ok(Self { activity, hero, ... })
    }
    
    /// 热加载（返回新的 Arc<StaticConfig>）
    pub async fn reload(pool: &MySqlPool) -> Result<Arc<Self>> {
        let config = Self::load_from_db(pool).await?;
        Ok(Arc::new(config))
    }
}
```

### 3.2 活动配置示例

```rust
/// 活动配置（对应 StaticActivityConfigMgr）
pub struct ActivityConfig {
    /// activityId → 活动计划
    pub plans: HashMap<i32, StaticActivityPlan>,
    /// formId → 玩法计划
    pub form_plans: HashMap<i32, StaticActivityFormPlan>,
    /// 活动周期
    pub cycles: Vec<StaticActivityCycle>,
    /// formType → formId 列表
    pub form_type_index: HashMap<i32, Vec<i32>>,
    /// 定时开启的活动
    pub fixed_time_plans: HashMap<i32, StaticActivityPlan>,
    /// 触发开启的活动
    pub trigger_plans: HashMap<i32, HashSet<StaticActivityPlan>>,
}

/// 活动计划（对应 s_activity_plan 表）
pub struct StaticActivityPlan {
    pub activity_id: i32,
    pub open_duration: i64,       // 开启时长，-1 表示永久
    pub display_duration: i64,    // 展示期时长
    pub form_ids: Vec<i32>,       // 关联的玩法 ID
    pub open_cond: OpenCondition, // 开启条件
    pub loop_config: Option<LoopConfig>, // 循环配置
}
```

### 3.3 配置热加载

```rust
/// 通过 watch channel 广播配置更新
pub struct ConfigWatcher {
    config: watch::Sender<Arc<StaticConfig>>,
}

// 各 Actor 持有 watch::Receiver，自动获取最新配置
impl PlayerActor {
    async fn run(mut self) {
        loop {
            tokio::select! {
                // ... 其他消息处理
                Ok(()) = self.config_rx.changed() => {
                    let new_config = self.config_rx.borrow().clone();
                    self.on_config_reload(new_config);
                }
            }
        }
    }
}
```

---

## 四、实现步骤

### Step 1：框架搭建（3 天）
- [ ] 定义 StaticConfig 容器和各子配置结构体
- [ ] 实现 sqlx 查询和映射
- [ ] 实现 ConfigWatcher（watch channel）

### Step 2：核心配置加载（1 周）
- [ ] 活动配置（s_activity_plan, s_activity_form_*, s_activity_cycle）
- [ ] 任务配置（s_mission, s_chapter_mission）
- [ ] 道具配置（s_item）
- [ ] 建筑/科技/将领配置

### Step 3：热加载集成（2 天）
- [ ] GM 命令触发配置热加载
- [ ] 各 Actor 响应配置更新
- [ ] 活动系统的 dealAfterConfigReload 逻辑
