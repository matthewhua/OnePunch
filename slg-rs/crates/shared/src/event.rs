//! 游戏事件系统
//!
//! `GameEvent` 在 PlayerActor 线程内同步分发，驱动任务/活动进度更新。
//! `GlobalEvent` 通过 channel 发往全局 ActivityActor，处理跨玩家逻辑。

// ── MissionType ───────────────────────────────────────────────────────────────

/// 任务触发类型，与 Java 版 MissionType 枚举及 s_task.mission_type 字段对齐。
///
/// 数值与数据库 mission_type 列一一对应，用于任务进度匹配。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum MissionType {
    // ── 登录 / 基础 ──────────────────────────────────────────────────────────
    LoginGame        = 1,   // 登录游戏
    OnlineTime       = 2,   // 在线时长（分钟）

    // ── 将领 ─────────────────────────────────────────────────────────────────
    HeroLevelUp      = 10,  // 将领升级（params[0]=heroId, params[1]=newLevel）
    HeroTierUp       = 11,  // 将领升星（params[0]=heroId, params[1]=newTier）
    HeroSkillLevelUp = 12,  // 将领技能升级
    HeroRecruit      = 13,  // 招募将领（params[0]=recruitType）
    HeroComposite    = 14,  // 合成将领

    // ── 建筑 ─────────────────────────────────────────────────────────────────
    BuildingUpgrade  = 20,  // 建筑升级（params[0]=buildType, params[1]=newLevel）
    BuildingBuild    = 21,  // 建造建筑（params[0]=buildType）

    // ── 科技 ─────────────────────────────────────────────────────────────────
    TechResearch     = 30,  // 科技研究完成（params[0]=techId, params[1]=newLevel）

    // ── 装备 ─────────────────────────────────────────────────────────────────
    EquipStrengthen  = 40,  // 装备强化（params[0]=equipConfigId, params[1]=newLevel）
    EquipWear        = 41,  // 穿戴装备
    EquipForging     = 42,  // 装备专精锻造

    // ── 背包 / 道具 ──────────────────────────────────────────────────────────
    UseItem          = 50,  // 使用道具（params[0]=propId, params[1]=count）
    GainItem         = 51,  // 获得道具（params[0]=propId, params[1]=count）

    // ── 资源 ─────────────────────────────────────────────────────────────────
    DiamondConsume   = 60,  // 消耗钻石（params[0]=amount）
    GoldConsume      = 61,  // 消耗金币（params[0]=amount）
    MeatConsume      = 62,  // 消耗肉（params[0]=amount）
    GatherResource   = 63,  // 采集资源（params[0]=resType, params[1]=amount）

    // ── 战斗 ─────────────────────────────────────────────────────────────────
    KillEnemy        = 70,  // 击杀敌人（params[0]=count）
    WinBattle        = 71,  // 赢得战斗（params[0]=count）
    AttackPlayer     = 72,  // 攻击玩家

    // ── 兵营 / 医院 ──────────────────────────────────────────────────────────
    TroopTrain       = 80,  // 训练士兵（params[0]=count）
    TroopHeal        = 81,  // 治疗伤兵（params[0]=count）

    // ── 任务 ─────────────────────────────────────────────────────────────────
    FinishMission    = 90,  // 完成任务（params[0]=missionDefine）
    FinishDailyTask  = 91,  // 完成日常任务（params[0]=count）

    // ── 活动 ─────────────────────────────────────────────────────────────────
    ActivitySign     = 100, // 活动签到（params[0]=activityId）
    ActivityScore    = 101, // 活动积分（params[0]=activityId, params[1]=score）

    // ── 充值 ─────────────────────────────────────────────────────────────────
    Recharge         = 110, // 充值（params[0]=amount）

    // ── 其他 ─────────────────────────────────────────────────────────────────
    Unknown          = 0,
}

impl MissionType {
    /// 从数据库 mission_type 整数值转换
    pub fn from_i32(v: i32) -> Self {
        match v {
            1  => Self::LoginGame,
            2  => Self::OnlineTime,
            10 => Self::HeroLevelUp,
            11 => Self::HeroTierUp,
            12 => Self::HeroSkillLevelUp,
            13 => Self::HeroRecruit,
            14 => Self::HeroComposite,
            20 => Self::BuildingUpgrade,
            21 => Self::BuildingBuild,
            30 => Self::TechResearch,
            40 => Self::EquipStrengthen,
            41 => Self::EquipWear,
            42 => Self::EquipForging,
            50 => Self::UseItem,
            51 => Self::GainItem,
            60 => Self::DiamondConsume,
            61 => Self::GoldConsume,
            62 => Self::MeatConsume,
            63 => Self::GatherResource,
            70 => Self::KillEnemy,
            71 => Self::WinBattle,
            72 => Self::AttackPlayer,
            80 => Self::TroopTrain,
            81 => Self::TroopHeal,
            90 => Self::FinishMission,
            91 => Self::FinishDailyTask,
            100 => Self::ActivitySign,
            101 => Self::ActivityScore,
            110 => Self::Recharge,
            _  => Self::Unknown,
        }
    }

    /// 转换为数据库整数值
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ── MissionEvent ──────────────────────────────────────────────────────────────

/// 任务/活动进度触发事件
#[derive(Debug, Clone)]
pub struct MissionEvent {
    pub role_id: i64,
    pub mission_type: MissionType,
    /// 扩展参数（含义由 mission_type 决定，与 s_task.params 对应）
    pub params: Vec<i64>,
}

impl MissionEvent {
    pub fn new(role_id: i64, mission_type: MissionType, params: Vec<i64>) -> Self {
        Self { role_id, mission_type, params }
    }

    /// 快捷构造：单参数
    pub fn single(role_id: i64, mission_type: MissionType, param: i64) -> Self {
        Self::new(role_id, mission_type, vec![param])
    }
}

// ── ActivityTriggerEvent ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActivityTriggerEvent {
    pub role_id: i64,
    pub trigger_type: i32,
    pub params: Vec<i64>,
}

// ── GameEvent ─────────────────────────────────────────────────────────────────

/// 玩家个人游戏事件，在 PlayerActor 线程内同步分发。
#[derive(Debug, Clone)]
pub enum GameEvent {
    // ── 生命周期 ──────────────────────────────────────────────────────────────
    PlayerLogin  { role_id: i64 },
    PlayerLogout { role_id: i64 },

    // ── 任务进度驱动 ──────────────────────────────────────────────────────────
    /// 通用任务进度事件，由各系统在关键操作后触发
    Mission(MissionEvent),

    // ── 将领 ──────────────────────────────────────────────────────────────────
    HeroLevelUp  { role_id: i64, hero_id: i32, new_level: i32 },
    HeroTierUp   { role_id: i64, hero_id: i32, new_tier: i32 },

    // ── 建筑 ──────────────────────────────────────────────────────────────────
    BuildingUpgrade { role_id: i64, build_type: i32, new_level: i32 },

    // ── 科技 ──────────────────────────────────────────────────────────────────
    TechResearchComplete { role_id: i64, tech_id: i32, new_level: i32 },

    // ── 装备 ──────────────────────────────────────────────────────────────────
    EquipStrengthen { role_id: i64, equip_config_id: i32, new_level: i32 },

    // ── 道具 ──────────────────────────────────────────────────────────────────
    ItemConsume { role_id: i64, prop_id: i32, count: i64 },
    ItemGain    { role_id: i64, prop_id: i32, count: i64 },

    // ── 资源 ──────────────────────────────────────────────────────────────────
    DiamondConsume { role_id: i64, amount: i64 },
    GoldConsume    { role_id: i64, amount: i64 },
    ResourceChange { resource_type: i32, delta: i64 },

    // ── 战斗 ──────────────────────────────────────────────────────────────────
    BattleEnd { role_id: i64, win: bool, enemy_power: i64 },

    // ── 兵营 ──────────────────────────────────────────────────────────────────
    TroopTrain { role_id: i64, count: i32 },
    TroopHeal  { role_id: i64, count: i32 },

    // ── 活动 ──────────────────────────────────────────────────────────────────
    ActivityTrigger(ActivityTriggerEvent),
}

impl GameEvent {
    /// 将语义事件自动转换为对应的 MissionEvent（用于任务/活动进度驱动）。
    ///
    /// 返回 `None` 表示该事件不需要触发任务进度。
    pub fn to_mission_event(&self) -> Option<MissionEvent> {
        match self {
            GameEvent::PlayerLogin { role_id } =>
                Some(MissionEvent::single(*role_id, MissionType::LoginGame, 1)),

            GameEvent::HeroLevelUp { role_id, hero_id, new_level } =>
                Some(MissionEvent::new(*role_id, MissionType::HeroLevelUp,
                    vec![*hero_id as i64, *new_level as i64])),

            GameEvent::HeroTierUp { role_id, hero_id, new_tier } =>
                Some(MissionEvent::new(*role_id, MissionType::HeroTierUp,
                    vec![*hero_id as i64, *new_tier as i64])),

            GameEvent::BuildingUpgrade { role_id, build_type, new_level } =>
                Some(MissionEvent::new(*role_id, MissionType::BuildingUpgrade,
                    vec![*build_type as i64, *new_level as i64])),

            GameEvent::TechResearchComplete { role_id, tech_id, new_level } =>
                Some(MissionEvent::new(*role_id, MissionType::TechResearch,
                    vec![*tech_id as i64, *new_level as i64])),

            GameEvent::EquipStrengthen { role_id, equip_config_id, new_level } =>
                Some(MissionEvent::new(*role_id, MissionType::EquipStrengthen,
                    vec![*equip_config_id as i64, *new_level as i64])),

            GameEvent::ItemConsume { role_id, prop_id, count } =>
                Some(MissionEvent::new(*role_id, MissionType::UseItem,
                    vec![*prop_id as i64, *count])),

            GameEvent::ItemGain { role_id, prop_id, count } =>
                Some(MissionEvent::new(*role_id, MissionType::GainItem,
                    vec![*prop_id as i64, *count])),

            GameEvent::DiamondConsume { role_id, amount } =>
                Some(MissionEvent::single(*role_id, MissionType::DiamondConsume, *amount)),

            GameEvent::GoldConsume { role_id, amount } =>
                Some(MissionEvent::single(*role_id, MissionType::GoldConsume, *amount)),

            GameEvent::TroopTrain { role_id, count } =>
                Some(MissionEvent::single(*role_id, MissionType::TroopTrain, *count as i64)),

            GameEvent::TroopHeal { role_id, count } =>
                Some(MissionEvent::single(*role_id, MissionType::TroopHeal, *count as i64)),

            GameEvent::BattleEnd { role_id, win, .. } if *win =>
                Some(MissionEvent::single(*role_id, MissionType::WinBattle, 1)),

            // Mission 事件本身直接透传
            GameEvent::Mission(_) => None,

            _ => None,
        }
    }
}

// ── GlobalEvent ───────────────────────────────────────────────────────────────

/// 全服/跨玩家事件，通过 channel 发往 GlobalEventBus / ActivityActor。
#[derive(Debug, Clone)]
pub enum GlobalEvent {
    WorldMilestoneMission {
        role_id: i64,
        mission_type: MissionType,
        params: Vec<i64>,
    },
    CampMilestoneMission {
        role_id: i64,
        camp_id: i32,
        mission_type: MissionType,
        params: Vec<i64>,
    },
    RankUpdate {
        rank_type: i32,
        role_id: i64,
        value: i64,
    },
}

// ── PlayerContext ─────────────────────────────────────────────────────────────

/// 供 EventHandler::handle 访问 PlayerActor 当前状态的上下文
pub struct PlayerContext {
    pub role_id: i64,
    pub account_id: i64,
    // 需要 Handler 访问的跨系统数据可以在这里追加
}

// ── EventHandler / EventDispatcher ───────────────────────────────────────────

pub trait EventHandler: Send {
    /// 判断该 Handler 是否对此事件感兴趣
    fn interested_in(&self, event: &GameEvent) -> bool;
    /// 处理事件（同步，PlayerActor 线程内运行）
    fn handle(&mut self, event: &GameEvent, ctx: &mut PlayerContext);
}

pub struct EventDispatcher {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

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
