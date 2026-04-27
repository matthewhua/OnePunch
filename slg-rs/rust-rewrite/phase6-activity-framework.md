# Phase 6：活动大框架（Activity Framework）

> 状态：待实现
> 前置依赖：Phase 3（Home Service PlayerActor 基础框架已就绪）
> 预估工期：3-4 周

---

## 一、目标

在 Rust 版 Home Service 中实现一套可扩展的运营活动框架，支撑 Java 版已有的 20+ 种活动玩法类型，并兼容现有客户端协议（proto2 Activity.proto）。

---

## 二、Java 版活动系统架构分析

### 2.1 核心分层

```
┌─────────────────────────────────────────────────────────────────┐
│  Handler 层（协议入口）                                          │
│  AbsActivityHandler<Service, Rs>                                │
│    ├── ActivitySignHandler                                      │
│    ├── ActivitySupremeLordInfoHandler                            │
│    ├── ActivityShopBuyHandler                                   │
│    └── ... 30+ 个具体 Handler                                   │
├─────────────────────────────────────────────────────────────────┤
│  Service 层（业务逻辑）                                          │
│  BaseActivityService                                            │
│    ├── ActivitySignService                                      │
│    ├── ActivitySupremeLordService（@Subscribe 事件驱动）          │
│    ├── ActivityTaskService                                      │
│    └── ... 15 个具体 Service                                    │
├─────────────────────────────────────────────────────────────────┤
│  Form 层（数据模型，个人 + 公共）                                 │
│  BaseActivityForm                                               │
│    ├── PersonalActivityForm<Pb>  ── 每个玩家独立的活动数据        │
│    │     ├── ActivityFormSign / ActivityFormTask / ...            │
│    │     └── stage/ActivityFormSupremeLord                       │
│    └── CommonActivityForm<Pb>    ── 全服共享的活动数据            │
│          ├── ActivityFormRankCommon / ActivityFormMonopolyCommon  │
│          └── stage/BaseStageRankActivityForm                    │
├─────────────────────────────────────────────────────────────────┤
│  Actor 层（生命周期管理）                                        │
│  CommonActivityActor                                            │
│    ├── 活动开启/关闭/阶段切换的定时调度                            │
│    ├── GlobalActivity 管理（全服活动实例）                        │
│    └── 配置热加载 dealAfterConfigReload                          │
├─────────────────────────────────────────────────────────────────┤
│  持久化层                                                       │
│  ActivityFunction（玩家 p_data blob）                            │
│    ├── PersonalActivity Map<activityId, PersonalActivity>       │
│    └── ActivityPersistent（跨季保留数据）                         │
│  p_global（全服活动数据，CommonActivityActor 管理）               │
├─────────────────────────────────────────────────────────────────┤
│  配置层                                                         │
│  StaticActivityConfigMgr                                        │
│    ├── StaticActivityPlan（活动计划：开启条件、时间、循环）         │
│    ├── StaticActivityFormPlan（玩法计划：formId → formType）      │
│    └── StaticActivityCycle（活动周期/赛季）                       │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 活动玩法类型枚举（ActivityFormType）

| formType | 名称 | 有公共数据 | 迁移状态 | 说明 |
|----------|------|-----------|---------|------|
| 1 | FORM_SIGN | ✗ | ✅ 需迁移 | 签到 |
| 2 | FORM_TASK | ✗ | ✅ 需迁移 | 做任务 |
| 3 | FORM_SCORE_AWARD | ✗ | ✅ 需迁移 | 积分奖励 |
| 4 | FORM_SHOP | ✗ | ✅ 需迁移 | 兑换商店 |
| 5 | FORM_GIFTPACK | ✗ | ✅ 需迁移 | 常规礼包 |
| 6 | FORM_OPT_PACK | ✗ | ✅ 需迁移 | 自选礼包 |
| 7 | FORM_RECHARGE_AWARD | ✗ | ✅ 需迁移 | 充值领奖 |
| 8 | FORM_RANK | ✓ | ✅ 需迁移 | 排行榜 |
| 9 | FORM_TURNTABLE | ✗ | ✅ 需迁移 | 转盘抽奖 |
| 10 | FORM_BATTLEPASS | ✗ | ❌ 不迁移 | 战令（保留 Java 侧） |
| 11 | FORM_QUESTIONNAIRE | ✗ | ✅ 需迁移 | 问卷调查 |
| 12 | FORM_TASK_GROUP | ✗ | ✅ 需迁移 | 任务组 |
| 13 | FORM_SUPREME_LORD | ✓ | ✅ 需迁移 | 最强领主（分阶段排行） |
| 14 | FORM_VOYAGE | ✗ | ✅ 需迁移 | 航行 |
| 15 | FORM_MONOPOLY | ✓ | ✅ 需迁移 | 大富翁 |
| 16 | FORM_BANK | ✗ | ✅ 需迁移 | 钻石银行 |
| 17 | FORM_HERO_HALL | ✓ | ✅ 需迁移 | 英雄殿堂 |
| 18 | FORM_MILESTONE | ✓ | ✅ 需迁移 | 里程碑（世界进程） |
| 19 | FORM_MILESTONE_BOSS | ✓ | ✅ 需迁移 | 世界进程 Boss |
| 20 | FORM_COMMUNITY_JUMP | ✗ | ❌ 不迁移 | 社区跳转（保留 Java 侧） |

> **不迁移说明**：战令（FORM_BATTLEPASS）和社区跳转（FORM_COMMUNITY_JUMP）保留在 Java 侧处理，Rust 活动框架不实现这两种玩法类型。

### 2.3 活动生命周期

```
配置加载 → 定时检查 → 预显示(PRE_DISPLAY) → 开启(OPEN) → 结束展示(END_DISPLAY) → 关闭(FOREVER_CLOSE)
                                                  │
                                                  ├── 每日 tick → 阶段切换（如最强领主每天一个阶段）
                                                  ├── 阶段结算 → 排行发奖
                                                  └── 玩法结束 → formEnd 逻辑
```

### 2.4 关键设计模式

1. **个人/公共数据分离**：PersonalActivityForm 存在每个玩家的 p_data blob 中；CommonActivityForm 存在全服 p_global 中
2. **事件驱动**：通过 EventBus @Subscribe 监听任务事件（BaseMissionEvent），驱动活动积分、任务进度等
3. **定时调度**：CommonActivityActor 通过 DelayRun 机制调度活动开启/关闭/阶段切换
4. **配置热加载**：dealAfterConfigReload 支持运行时修改活动配置

---

## 三、Rust 版活动框架设计

### 3.1 模块结构

```
crates/home/src/systems/activity/
├── mod.rs                    # ActivitySystem 入口，实现 PlayerSystem trait
├── types.rs                  # ActivityFormType 枚举、ActivityStage 枚举
├── model.rs                  # PersonalActivity、ActivityForm trait 定义
├── config.rs                 # 活动静态配置加载（对应 StaticActivityConfigMgr）
├── lifecycle.rs              # 活动生命周期管理（开启/关闭/阶段切换）
├── forms/                    # 各玩法实现（共 18 种，排除战令和社区跳转）
│   ├── mod.rs
│   ├── sign.rs               # 签到
│   ├── task.rs               # 做任务
│   ├── score.rs              # 积分奖励
│   ├── shop.rs               # 兑换商店
│   ├── giftpack.rs           # 常规礼包 + 自选礼包
│   ├── recharge.rs           # 充值领奖
│   ├── rank.rs               # 排行榜
│   ├── turntable.rs          # 转盘
│   ├── questionnaire.rs      # 问卷调查
│   ├── task_group.rs         # 任务组
│   ├── supreme_lord.rs       # 最强领主
│   ├── voyage.rs             # 航行
│   ├── monopoly.rs           # 大富翁
│   ├── bank.rs               # 钻石银行
│   ├── hero_hall.rs          # 英雄殿堂
│   └── milestone.rs          # 里程碑 + 里程碑 Boss
└── settle.rs                 # 结算逻辑（排行发奖、阶段结算）
```

> **注意**：战令（FORM_BATTLEPASS）和社区跳转（FORM_COMMUNITY_JUMP）不在 Rust 侧实现，保留 Java 侧处理。

全服公共活动数据放在 World Service 或独立的 ActivityActor 中：

```
crates/home/src/actors/
├── activity_actor.rs         # 全服活动 Actor（对应 CommonActivityActor）
│   ├── GlobalActivity 管理
│   ├── 公共 Form 数据（排行榜、里程碑等）
│   ├── 定时调度（活动开启/关闭/阶段切换）
│   └── 配置热加载
```

### 3.2 核心 trait 设计

```rust
/// 活动玩法类型（排除战令和社区跳转，这两种保留 Java 侧）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ActivityFormType {
    Sign = 1,
    Task = 2,
    ScoreAward = 3,
    Shop = 4,
    Giftpack = 5,
    OptPack = 6,
    RechargeAward = 7,
    Rank = 8,
    Turntable = 9,
    // Battlepass = 10,      // 不迁移，保留 Java 侧
    Questionnaire = 11,
    TaskGroup = 12,
    SupremeLord = 13,
    Voyage = 14,
    Monopoly = 15,
    Bank = 16,
    HeroHall = 17,
    Milestone = 18,
    MilestoneBoss = 19,
    // CommunityJump = 20,   // 不迁移，保留 Java 侧
}

/// 个人活动表单 trait（对应 Java PersonalActivityForm）
pub trait PersonalForm: Send + Sync {
    fn form_type(&self) -> ActivityFormType;
    
    /// 从 protobuf 反序列化
    fn deserialize(&mut self, data: &[u8]) -> Result<()>;
    
    /// 序列化为 protobuf（save_db=true 时包含服务端专用字段）
    fn serialize(&self, save_db: bool) -> Result<Vec<u8>>;
    
    /// 构建客户端推送的 ActivityFormPb
    fn to_client_pb(&self, activity: &ActivityData) -> Result<Vec<u8>>;
    
    /// 活动初始化时调用
    fn on_activity_init(&mut self, activity: &ActivityData) {}
    
    /// 活动关闭时调用
    fn on_activity_close(&mut self, activity: &ActivityData) {}
    
    /// 每日 tick
    fn on_daily_tick(&mut self, activity: &ActivityData, day_num: i32) {}
}

/// 公共活动表单 trait（对应 Java CommonActivityForm，全服共享）
pub trait CommonForm: Send + Sync {
    fn form_type(&self) -> ActivityFormType;
    fn deserialize(&mut self, data: &[u8]) -> Result<()>;
    fn serialize(&self, save_db: bool) -> Result<Vec<u8>>;
    fn on_activity_init(&mut self, activity: &GlobalActivityData) {}
    fn on_activity_close(&mut self, activity: &GlobalActivityData) {}
    fn on_daily_tick(&mut self, activity: &GlobalActivityData, day_num: i32) {}
}

/// 玩家的活动数据（对应 Java PersonalActivity）
pub struct PersonalActivity {
    pub activity_id: i32,
    pub open_times: i32,
    pub forms: HashMap<i32, Box<dyn PersonalForm>>,  // formId -> Form
    pub entrance_closed: bool,
}

/// 全服活动数据（对应 Java GlobalActivity）
pub struct GlobalActivityData {
    pub activity_id: i32,
    pub stage: ActivityStage,
    pub begin_time: i64,
    pub end_time: i64,
    pub display_end_time: i64,
    pub open_times: i32,
    pub day_num: i32,
    pub common_forms: HashMap<i32, Box<dyn CommonForm>>,
}
```

### 3.3 ActivitySystem（玩家侧）

```rust
/// 玩家活动系统（对应 Java ActivityFunction）
pub struct ActivitySystem {
    /// 玩家的所有活动数据
    activities: HashMap<i32, PersonalActivity>,  // activityId -> PersonalActivity
    /// 跨季保留数据
    persistent: ActivityPersistent,
    /// dirty 标记
    dirty: bool,
}

impl PlayerSystem for ActivitySystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> { /* 解码 ActivityFunction pb */ }
    fn save_to_bin(&self) -> Result<Vec<u8>> { /* 编码 ActivityFunction pb */ }
}

impl ActivitySystem {
    /// 处理活动相关命令（对应 Java 各 Handler）
    pub fn handle_command(&mut self, cmd: u32, payload: &[u8], ctx: &mut PlayerContext) -> Result<Vec<u8>> {
        match cmd {
            8001 => self.get_activity_func_data(ctx),
            8007 => self.activity_sign(payload, ctx),
            8009 => self.gain_task_award(payload, ctx),
            8033 => self.supreme_lord_info(payload, ctx),
            8035 => self.supreme_lord_claim_award(payload, ctx),
            // ... 其他活动命令
            _ => Err(anyhow!("未知活动命令: {}", cmd)),
        }
    }
}
```

### 3.4 ActivityActor（全服侧）

```rust
/// 全服活动 Actor（对应 Java CommonActivityActor）
pub struct ActivityActor {
    /// 全服活动实例
    global_activities: HashMap<i32, GlobalActivityData>,
    /// 活动配置
    config: Arc<ActivityConfig>,
    /// 消息接收
    rx: mpsc::Receiver<ActivityMessage>,
    /// 通知 Home Service 的 channel
    home_notify: mpsc::Sender<ActivityNotify>,
}

pub enum ActivityMessage {
    /// 定时 tick（每秒）
    Tick,
    /// 查询全服活动数据（Home Service 调用）
    GetGlobalActivity { activity_id: i32, reply: oneshot::Sender<Option<GlobalActivityCopy>> },
    /// 更新排行榜
    UpdateRank { activity_id: i32, form_id: i32, role_id: i64, value: i64 },
    /// 配置热加载
    ReloadConfig(Arc<ActivityConfig>),
}

impl ActivityActor {
    pub async fn run(mut self) {
        let mut tick = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => self.handle(msg).await,
                _ = tick.tick() => self.on_tick().await,
            }
        }
    }
    
    /// 每秒 tick：检查活动开启/关闭/阶段切换
    async fn on_tick(&mut self) {
        let now = current_timestamp();
        // 检查预显示 → 开启
        // 检查开启 → 结束展示
        // 检查结束展示 → 关闭
        // 检查阶段切换（如最强领主每日切换）
    }
}
```

---

## 四、协议兼容方案

### 4.1 proto2 兼容

Rust 的 `prost` 默认支持 proto3，但项目中 Activity.proto 使用 proto2。需要：

1. 在 `build.rs` 中配置 prost 支持 proto2 语法（prost 本身支持 proto2）
2. proto2 的 `extensions` 机制在 prost 中不直接支持，需要手动处理扩展字段的编解码
3. 方案：保持 proto2 文件不变，在 Rust 侧实现自定义的扩展字段编解码层

### 4.2 扩展字段处理

Java 版大量使用 protobuf extensions（如 `ActivityFormPb` 的 `extensions 10 to 100`），prost 不原生支持。解决方案：

```rust
/// 方案 A：手动解析未知字段
/// ActivityFormPb 解码后，根据 formType 手动解析 extension 字段
fn decode_form_extension(form_type: ActivityFormType, raw_bytes: &[u8]) -> Result<Box<dyn PersonalForm>> {
    match form_type {
        ActivityFormType::Sign => {
            let pb = ActivityFormSignPb::decode(raw_bytes)?;
            Ok(Box::new(SignForm::from_pb(pb)))
        }
        ActivityFormType::SupremeLord => {
            let pb = ActivityFormSupremeLordPb::decode(raw_bytes)?;
            Ok(Box::new(SupremeLordForm::from_pb(pb)))
        }
        // ...
    }
}

/// 方案 B（推荐）：将 proto2 extensions 改写为 proto3 oneof
/// 新建 activity_v2.proto，使用 oneof 替代 extensions
/// 在 Gateway 层做协议转换（客户端仍用 proto2，服务端内部用 proto3）
```

### 4.3 命令号映射

活动相关命令号范围：8001-8074, 8961-8966

需要在 `shared/src/cmd.rs` 中补充这些命令号，并在 Gateway 路由表中标记为 Home 命令。

---

## 五、实现步骤

### Step 1：基础框架（1 周）
- [ ] 定义 `ActivityFormType` 枚举和 `ActivityStage` 枚举
- [ ] 定义 `PersonalForm` / `CommonForm` trait
- [ ] 实现 `PersonalActivity` / `GlobalActivityData` 数据结构
- [ ] 实现 `ActivitySystem`（PlayerSystem trait）的 load/save
- [ ] 处理 proto2 extensions 的编解码兼容层
- [ ] 在 `cmd.rs` 中注册活动命令号

### Step 2：活动生命周期（1 周）
- [ ] 实现 `ActivityActor`（全服活动管理）
- [ ] 实现活动配置加载（StaticActivityPlan / StaticActivityFormPlan）
- [ ] 实现活动开启/关闭/阶段切换的定时调度
- [ ] 实现 ActivityActor ↔ PlayerActor 的通信（活动开启通知、排行更新）

### Step 3：首批玩法实现（1 周）
- [ ] 签到（FORM_SIGN）— 最简单，验证框架可用性
- [ ] 做任务（FORM_TASK）— 验证事件驱动机制
- [ ] 积分奖励（FORM_SCORE_AWARD）— 验证积分累计
- [ ] 最强领主（FORM_SUPREME_LORD）— 验证阶段排行机制

### Step 4：剩余玩法 + 结算（1 周）
- [ ] 排行榜、商店、礼包、战令、转盘等
- [ ] 阶段结算发奖（SupremeLordStageSettleAward）
- [ ] 里程碑系统（与 World Service 交互）
- [ ] GetActivityFuncData 协议（8001/8002）完整实现

---

## 六、给 AI 的实现提示词

```
你是一个 Rust 游戏服务器开发者。请在 crates/home/src/systems/activity/ 下实现活动框架。

技术栈：tokio, prost, serde, dashmap
核心要求：
1. 定义 PersonalForm trait 和 CommonForm trait，所有玩法类型实现这两个 trait
2. ActivitySystem 实现 PlayerSystem trait，支持从 proto2 ActivityFunction 消息加载/保存
3. 活动命令分发：根据 cmd 号路由到对应玩法的处理方法
4. proto2 extensions 兼容：ActivityFormPb 的扩展字段需要手动编解码

参考 Java 实现：
- AbsActivityHandler：活动 Handler 基类，封装了 checkAndGetActivityContext 通用验证
- ActivityFormType：20 种玩法类型枚举，部分有公共数据（isGlobal）
- PersonalActivityForm：个人活动表单，每个玩家独立
- CommonActivityForm：公共活动表单，全服共享（如排行榜）
- CommonActivityActor：全服活动生命周期管理
- ActivityFunction：玩家活动数据持久化（p_data blob）

协议兼容：
- Activity.proto 使用 proto2 语法，大量使用 extensions
- 命令号范围：8001-8074, 8961-8966
- 客户端协议不变，Rust 侧需要兼容 proto2 编解码
```
