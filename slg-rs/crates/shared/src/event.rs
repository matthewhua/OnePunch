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

#[derive(Debug, Clone)]
pub struct MissionEvent {
    pub role_id: i64,
    pub mission_type: MissionType,
    pub params: Vec<i64>, // 参数（如击杀数量等）
}

#[derive(Debug, Clone)]
pub struct ActivityTriggerEvent {
    pub role_id: i64,
    pub trigger_type: i32,
    pub params: Vec<i64>,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    Mission(MissionEvent),
    PlayerLogin {
        role_id: i64,
    },
    PlayerLogout {
        role_id: i64,
    },
    ActivityTrigger(ActivityTriggerEvent),
    ResourceChange {
        resource_type: i32,
        delta: i64,
    },
    BattleEnd {
        role_id: i64,
        win: bool,
        enemy_power: i64,
    },
}

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

/// 供 EventHandler::handle 访问 PlayerActor 当前状态的上下文
pub struct PlayerContext {
    pub role_id: i64,
    pub account_id: i64,
    // 需要 Handler 访问的跨系统数据可以在这里追加
}

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
        Self {
            handlers: Vec::new(),
        }
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
