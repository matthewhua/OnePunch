use tokio::sync::{mpsc, oneshot, watch};
use std::sync::Arc;
use std::time::Duration;
use prost::Message;
use tracing::{info, warn, error};
use proto::slg::{FunctionClientBase, GetRoleDataRs, RoleLoginRs};
use shared::static_config::StaticConfig;
use shared::persistence::{PlayerDao, LordRow, SaveEntry};

use crate::systems::PlayerSystem;
use crate::systems::activity::ActivitySystem;
use crate::systems::hero::HeroSystem;
use crate::systems::backpack::BackpackSystem;
use crate::systems::building::BuildingSystem;
use crate::systems::tech::TechSystem;
use crate::systems::equip::EquipSystem;
use crate::systems::mission::MissionSystem;
use crate::systems::skin::SkinSystem;
use shared::event::{EventDispatcher, PlayerContext, GameEvent};
use crate::actors::global_event_bus::GlobalEventBus;

/// 存盘间隔（秒）
const SAVE_INTERVAL_SECS: u64 = 300;
/// 存盘超时（秒）
const SAVE_TIMEOUT_SECS: u64 = 10;
/// 紧急存盘目录
const EMERGENCY_SAVE_DIR: &str = "./emergency_saves";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Loading,
    InGame,
    Offline,
}

pub enum PlayerMessage {
    /// 登入游戏请求
    RoleLogin(oneshot::Sender<anyhow::Result<RoleLoginRs>>),
    /// 同步数据请求
    GetRoleData(oneshot::Sender<anyhow::Result<GetRoleDataRs>>),
    /// 心跳
    Heartbeat,
    /// 下线自毁
    Shutdown,
    /// 游戏内行为事件
    DispatchGameEvent(GameEvent),
    /// 立即存盘（充值等关键操作后触发）
    ForceSave,
    /// Gateway 转发的业务命令
    GameCommand {
        cmd: u32,
        payload: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<Vec<u8>>>,
    },
}

pub struct PlayerActor {
    pub account_id: i64,
    pub role_id: i64,
    pub state: PlayerState,

    msg_rx: mpsc::UnboundedReceiver<PlayerMessage>,

    // ── 领主基础数据（p_lord，直接按列读写）──
    pub lord: Option<LordRow>,
    /// 领主数据是否 dirty
    lord_dirty: bool,

    // ── 功能系统模块（p_data，blob 列）──
    pub activity_system: ActivitySystem,
    pub hero_system: HeroSystem,
    pub backpack_system: BackpackSystem,
    pub building_system: BuildingSystem,
    pub tech_system: TechSystem,
    pub equip_system: EquipSystem,
    pub mission_system: MissionSystem,
    pub skin_system: SkinSystem,

    // ── 事件 ──
    pub event_dispatcher: EventDispatcher,
    pub ctx: PlayerContext,
    pub global_event_bus: GlobalEventBus,

    /// 静态配置订阅
    pub config_rx: watch::Receiver<Arc<StaticConfig>>,
    pub current_config: Arc<StaticConfig>,

    /// 数据库访问
    pub dao: Arc<PlayerDao>,
    /// 连续存盘失败次数
    save_fail_count: u32,
    /// 上线时间戳（用于计算在线时长）
    login_timestamp: i64,
}

impl PlayerActor {
    pub fn new(
        account_id: i64,
        role_id: i64,
        rx: mpsc::UnboundedReceiver<PlayerMessage>,
        global_event_bus: GlobalEventBus,
        config_rx: watch::Receiver<Arc<StaticConfig>>,
        dao: Arc<PlayerDao>,
    ) -> Self {
        let current_config = config_rx.borrow().clone();
        Self {
            account_id,
            role_id,
            state: PlayerState::Loading,
            msg_rx: rx,
            lord: None,
            lord_dirty: false,
            activity_system: ActivitySystem::new(),
            hero_system: HeroSystem::new(),
            backpack_system: BackpackSystem::new(),
            building_system: BuildingSystem::new(),
            tech_system: TechSystem::new(),
            equip_system: EquipSystem::new(),
            mission_system: MissionSystem::new(),
            skin_system: SkinSystem::new(),
            event_dispatcher: EventDispatcher::new(),
            ctx: PlayerContext { role_id, account_id },
            global_event_bus,
            config_rx,
            current_config,
            dao,
            save_fail_count: 0,
            login_timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 从数据库加载玩家数据
    pub async fn load_data(&mut self) -> anyhow::Result<()> {
        // 1. 加载 p_lord（领主基础数据）
        self.lord = self.dao.load_lord(self.role_id).await?;

        // 2. 加载 p_data（功能模块 blob 数据）
        if let Some(row) = self.dao.load_player_data(self.role_id).await? {
            self.load_system_from_row(&row);
        }

        // 3. 更新登录信息
        self.dao.update_login(self.role_id).await?;
        self.dao.set_log_off(self.role_id, false).await?;

        info!(role_id = self.role_id, "Player data loaded");
        Ok(())
    }

    /// 将 PlayerDataRow 中的各列分发到对应系统
    fn load_system_from_row(&mut self, row: &shared::persistence::PlayerDataRow) {
        let systems: Vec<(&mut dyn PlayerSystem, Option<&Vec<u8>>)> = vec![
            (&mut self.activity_system,  row.activity_func.as_ref()),
            (&mut self.hero_system,      row.hero_func.as_ref()),
            (&mut self.backpack_system,  row.backpack_func.as_ref()),
            (&mut self.building_system,  row.sim_func.as_ref()),
            (&mut self.tech_system,      row.technology_func.as_ref()),
            (&mut self.equip_system,     row.equip_func.as_ref()),
            (&mut self.mission_system,   row.mission_func.as_ref()),
            (&mut self.skin_system,      row.skin_func.as_ref()),
        ];

        for (system, data_opt) in systems {
            if let Some(data) = data_opt {
                if !data.is_empty() {
                    if let Err(e) = system.load_from_bin(data) {
                        warn!(
                            role_id = self.role_id,
                            col = system.column_name(),
                            "Failed to deserialize, using default: {}", e
                        );
                    }
                }
            }
        }
    }

    pub async fn run(mut self) {
        info!(role_id = self.role_id, "PlayerActor started");

        if let Err(e) = self.load_data().await {
            error!(role_id = self.role_id, "Failed to load player data: {}", e);
            return;
        }
        self.state = PlayerState::Loading;

        // 触发各系统 on_login
        self.activity_system.on_login();
        self.hero_system.on_login();
        self.backpack_system.on_login();
        self.building_system.on_login();
        self.tech_system.on_login();
        self.equip_system.on_login();
        self.mission_system.on_login();
        self.skin_system.on_login();

        let mut save_interval = tokio::time::interval(Duration::from_secs(SAVE_INTERVAL_SECS));
        save_interval.tick().await; // 跳过第一次立即触发
        let mut tick_interval = tokio::time::interval(Duration::from_secs(1));
        tick_interval.tick().await;

        loop {
            tokio::select! {
                Some(msg) = self.msg_rx.recv() => {
                    match msg {
                        PlayerMessage::RoleLogin(tx) => {
                            self.handle_role_login(tx).await;
                        }
                        PlayerMessage::GetRoleData(tx) => {
                            self.handle_get_role_data(tx).await;
                        }
                        PlayerMessage::Heartbeat => {}
                        PlayerMessage::Shutdown => {
                            info!(role_id = self.role_id, "Shutting down, saving...");
                            self.do_save(true).await;
                            // 更新在线时长
                            let now = chrono::Utc::now().timestamp() as i32;
                            let delta = (now - self.login_timestamp as i32).max(0);
                            self.dao.update_lord_offline(self.role_id, delta, now).await.ok();
                            self.dao.set_log_off(self.role_id, true).await.ok();
                            info!(role_id = self.role_id, "PlayerActor stopped");
                            return;
                        }
                        PlayerMessage::DispatchGameEvent(event) => {
                            self.dispatch_event(&event);
                        }
                        PlayerMessage::ForceSave => {
                            self.do_save(false).await;
                        }
                        PlayerMessage::GameCommand { cmd, payload, reply } => {
                            let result = self.handle_game_command(cmd, payload).await;
                            let _ = reply.send(result);
                        }
                    }
                }
                _ = save_interval.tick() => {
                    self.do_save(false).await;
                }
                _ = tick_interval.tick() => {
                    self.on_tick();
                }
                Ok(()) = self.config_rx.changed() => {
                    self.current_config = self.config_rx.borrow().clone();
                }
            }
        }
    }

    fn on_tick(&mut self) {
        self.activity_system.tick();
        self.hero_system.tick();
        self.building_system.tick();
        self.skin_system.tick();

        // 科技研究完成检测
        let now = chrono::Utc::now().timestamp();
        let tech_events = self.tech_system.check_research_complete(self.role_id, now);
        for event in tech_events {
            self.dispatch_event(&event);
        }
    }

    /// 执行存盘
    ///
    /// `force_all` = true 时全量存盘（下线），否则仅存 dirty 模块。
    async fn do_save(&mut self, force_all: bool) {
        // 1. 存盘 p_lord（领主基础数据）
        if (force_all || self.lord_dirty) {
            if let Some(lord) = &self.lord {
                if let Err(e) = self.dao.save_lord(lord).await {
                    error!(role_id = self.role_id, "Failed to save p_lord: {}", e);
                } else {
                    self.lord_dirty = false;
                }
            }
        }

        // 2. 存盘 p_data（功能模块 blob）
        let mut entries: Vec<SaveEntry> = Vec::new();

        let systems: Vec<&mut dyn PlayerSystem> = vec![
            &mut self.activity_system,
            &mut self.hero_system,
            &mut self.backpack_system,
            &mut self.building_system,
            &mut self.tech_system,
            &mut self.equip_system,
            &mut self.mission_system,
            &mut self.skin_system,
        ];

        for system in systems {
            if force_all || system.is_dirty() {
                match system.save_to_bin() {
                    Ok(data) => {
                        entries.push(SaveEntry {
                            column: system.column_name(),
                            data,
                        });
                        system.clear_dirty();
                    }
                    Err(e) => {
                        error!(
                            role_id = self.role_id,
                            col = system.column_name(),
                            "Serialize failed: {}", e
                        );
                    }
                }
            }
        }

        if entries.is_empty() {
            return;
        }

        let entry_count = entries.len();
        let save_result = tokio::time::timeout(
            Duration::from_secs(SAVE_TIMEOUT_SECS),
            self.dao.save_player_data(self.role_id, &entries),
        ).await;

        match save_result {
            Ok(Ok(())) => {
                self.save_fail_count = 0;
                info!(
                    role_id = self.role_id, modules = entry_count,
                    force = force_all, "Save completed"
                );
            }
            Ok(Err(e)) => {
                self.save_fail_count += 1;
                error!(role_id = self.role_id, fail_count = self.save_fail_count, "Save failed: {}", e);
                if self.save_fail_count >= 3 {
                    warn!(role_id = self.role_id, "Emergency save to file");
                    shared::persistence::emergency_save_to_file(EMERGENCY_SAVE_DIR, self.role_id, &entries);
                }
            }
            Err(_) => {
                self.save_fail_count += 1;
                error!(role_id = self.role_id, "Save timed out ({}s)", SAVE_TIMEOUT_SECS);
                if self.save_fail_count >= 3 {
                    shared::persistence::emergency_save_to_file(EMERGENCY_SAVE_DIR, self.role_id, &entries);
                }
            }
        }
    }

    async fn handle_role_login(&mut self, tx: oneshot::Sender<anyhow::Result<RoleLoginRs>>) {
        info!(role_id = self.role_id, "Role logged in");
        self.state = PlayerState::InGame;
        self.login_timestamp = chrono::Utc::now().timestamp();

        let event = GameEvent::PlayerLogin { role_id: self.role_id };
        self.dispatch_event(&event);

        let _ = tx.send(Ok(RoleLoginRs { state: Some(1), ..Default::default() }));
    }

    pub fn dispatch_event(&mut self, event: &GameEvent) {
        // 1. 将语义事件自动转换为 MissionEvent，驱动任务/活动进度
        if let Some(mission_event) = event.to_mission_event() {
            let me = GameEvent::Mission(mission_event);
            self.activity_system.handle_event(&me, &mut self.ctx);
            // 带 config 的任务进度更新（能查 s_task 配置）
            self.mission_system.handle_event_with_config(&me, &self.current_config, &mut self.ctx);
            self.event_dispatcher.dispatch(&me, &mut self.ctx);
        }

        // 2. 原始事件也分发一遍（部分 handler 需要原始语义）
        self.activity_system.handle_event(event, &mut self.ctx);
        self.mission_system.handle_event(event, &mut self.ctx);
        self.event_dispatcher.dispatch(event, &mut self.ctx);
    }

    async fn handle_get_role_data(&mut self, tx: oneshot::Sender<anyhow::Result<GetRoleDataRs>>) {
        let function_base = collect_get_role_data_function_base(
            self.role_id,
            self.lord.as_ref(),
            &self.activity_system,
            &self.hero_system,
            &self.backpack_system,
            &self.building_system,
            &self.tech_system,
            &self.equip_system,
            &self.mission_system,
            &self.skin_system,
        );

        let rs = GetRoleDataRs { function_base, ..Default::default() };
        let _ = tx.send(Ok(rs));
    }

    /// 按 cmd 范围将业务命令路由到对应系统处理，返回序列化后的响应 payload。
    ///
    /// cmd 范围对应关系（与 proto 文件保持一致）：
    /// - 1101-1200  Game（登录/任务相关）→ mission_system
    /// - 1501-1603  Simulate（建筑/模拟经营）→ building_system
    /// - 2001-2500  Hero（将领）→ hero_system
    /// - 4001-4004  Bag（背包）→ backpack_system
    /// - 4201-4208  Technology（科技）→ tech_system
    /// - 4801-5100  Equip（装备）→ equip_system
    /// - 8001-10000 Activity（活动）→ activity_system
    pub async fn handle_game_command(
        &mut self,
        cmd: u32,
        payload: Vec<u8>,
    ) -> anyhow::Result<Vec<u8>> {
        use crate::systems::PlayerSystem;
        let config = self.current_config.clone();
        let payload = if let Ok(msg) = shared::msg::GameMessage::decode(payload.clone()) {
            if msg.base.cmd == cmd as i32 {
                msg.get_payload_bytes().unwrap_or(payload)
            } else {
                payload
            }
        } else {
            payload
        };

        // 调用对应系统，获取响应 payload 和需要分发的事件
        let (resp, events) = match cmd {
            1101..=1200 => self.mission_system.handle_command_with_events(cmd, &payload, &config),
            1501..=1603 => self.building_system.handle_command_with_events(cmd, &payload, &config),
            2001..=2500 => self.hero_system.handle_command_with_events(cmd, &payload, &config),
            4001..=4004 => self.backpack_system.handle_command_with_events(cmd, &payload, &config),
            4201..=4208 => self.tech_system.handle_command_with_events(cmd, &payload, &config),
            4801..=5100 => self.equip_system.handle_command_with_events(cmd, &payload, &config),
            8001..=10000 => self.activity_system.handle_command_with_events(cmd, &payload, &config),
            _ => {
                warn!(role_id = self.role_id, cmd, "Unhandled cmd, no system matched");
                return Err(anyhow::anyhow!("Unknown cmd: {}", cmd));
            }
        }?;

        // 分发系统产生的游戏事件（驱动任务/活动进度）
        for event in events {
            self.dispatch_event(&event);
        }

        shared::msg::GameMessage::build_response_from_raw(cmd as i32 + 1, &resp)
    }
}

fn push_function_base(
    function_base: &mut Vec<FunctionClientBase>,
    role_id: i64,
    module: &'static str,
    bytes: Vec<u8>,
) {
    match FunctionClientBase::decode(bytes.as_slice()) {
        Ok(f_base) => function_base.push(f_base),
        Err(e) => {
            warn!(role_id = role_id, module, "Failed to decode FunctionClientBase: {}", e);
        }
    }
}

fn collect_get_role_data_function_base(
    role_id: i64,
    lord: Option<&LordRow>,
    activity_system: &ActivitySystem,
    hero_system: &HeroSystem,
    backpack_system: &BackpackSystem,
    building_system: &BuildingSystem,
    tech_system: &TechSystem,
    equip_system: &EquipSystem,
    mission_system: &MissionSystem,
    skin_system: &SkinSystem,
) -> Vec<FunctionClientBase> {
    use shared::msg::ToFunctionClientBaseBytes;

    let mut function_base = Vec::new();

    if let Some(lord) = lord {
        push_function_base(
            &mut function_base,
            role_id,
            "lord",
            build_lord_function(lord).to_function_base_bytes(),
        );
    }
    push_function_base(
        &mut function_base,
        role_id,
        "activity",
        activity_system.to_function_base_bytes(),
    );
    push_function_base(
        &mut function_base,
        role_id,
        "hero",
        hero_system.to_function_base_bytes(),
    );
    push_function_base(
        &mut function_base,
        role_id,
        "backpack",
        backpack_system.to_function_base_bytes(),
    );
    push_function_base(
        &mut function_base,
        role_id,
        "building",
        building_system.to_function_base_bytes(),
    );
    push_function_base(
        &mut function_base,
        role_id,
        "tech",
        tech_system.to_function_base_bytes(),
    );
    push_function_base(
        &mut function_base,
        role_id,
        "equip",
        equip_system.to_function_base_bytes(),
    );
    push_function_base(
        &mut function_base,
        role_id,
        "mission",
        mission_system.to_function_base_bytes(),
    );
    push_function_base(
        &mut function_base,
        role_id,
        "skin",
        skin_system.to_function_base_bytes(),
    );

    function_base
}

fn build_lord_function(lord: &LordRow) -> proto::slg::LordDataFunction {
    proto::slg::LordDataFunction {
        nick_name: lord.nick.clone(),
        portrait: lord.portrait.as_ref().and_then(|value| value.parse::<i32>().ok()),
        diamond: lord.diamond,
        battle_fight: lord.battle_fight,
        guide_index: lord.guide_id,
        title: lord.title,
        portrait_frame: lord.portrait_frame,
        role_status: None,
        server_open_time: None,
        role_create_time: None,
        off_time: lord.off_time,
        meat: lord.meat,
        fame: lord.fame,
        gold: lord.gold,
        search_survivor_time: lord.search_survivor_time,
        stamina: lord.stamina.and_then(i64_to_i32),
        kill_enemy_count: None,
        union_id: lord.camp_id,
        total_login: lord.total_login,
        current_streak: lord.current_streak,
        vip_level: lord.vip_level,
        vip_exp: lord.vip_exp,
        setting: Vec::new(),
    }
}

fn i64_to_i32(value: i64) -> Option<i32> {
    i32::try_from(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::msg::func_type;

    #[test]
    fn collect_get_role_data_function_base_includes_building_and_skin() {
        let function_base = collect_get_role_data_function_base(
            42,
            None,
            &ActivitySystem::new(),
            &HeroSystem::new(),
            &BackpackSystem::new(),
            &BuildingSystem::new(),
            &TechSystem::new(),
            &EquipSystem::new(),
            &MissionSystem::new(),
            &SkinSystem::new(),
        );

        let types: Vec<i32> = function_base
            .iter()
            .map(|base| base.r#type.expect("function base missing type"))
            .collect();

        assert_eq!(
            types,
            vec![
                func_type::ACTIVITY,
                func_type::HERO,
                func_type::BAG,
                func_type::SIM,
                func_type::TECHNOLOGY,
                func_type::EQUIP,
                func_type::MISSION,
                func_type::SKIN,
            ]
        );
    }

    #[test]
    fn push_function_base_skips_invalid_payload() {
        let mut function_base = Vec::new();

        push_function_base(&mut function_base, 42, "bad", vec![0xff, 0x01]);

        assert!(function_base.is_empty());
    }

    #[test]
    fn collect_get_role_data_function_base_includes_lord_when_loaded() {
        let lord = LordRow {
            role_id: 42,
            nick: Some("tester".to_string()),
            portrait: Some("7".to_string()),
            portrait_frame: Some(8),
            top_up: None,
            diamond: Some(100),
            diamond_cost: None,
            guide_id: Some(3),
            on_time: None,
            ol_time: None,
            off_time: Some(11),
            ol_month: None,
            title: Some(2),
            max_key: None,
            role_status: None,
            across_day_deal_time: None,
            battle_fight: Some(9000),
            meat: Some(200),
            fame: Some(12),
            gold: Some(300),
            search_survivor_time: Some(44),
            stamina: Some(50),
            start_ad_time: None,
            start_ad_id: None,
            is_add_login: None,
            total_login: Some(5),
            current_streak: Some(2),
            vip_level: Some(1),
            vip_exp: Some(10),
            camp_id: Some(6),
            last_periodic_task_time: None,
            lord_system_setting: None,
            pay_amount: None,
            language: None,
            push_switch: None,
        };

        let function_base = collect_get_role_data_function_base(
            42,
            Some(&lord),
            &ActivitySystem::new(),
            &HeroSystem::new(),
            &BackpackSystem::new(),
            &BuildingSystem::new(),
            &TechSystem::new(),
            &EquipSystem::new(),
            &MissionSystem::new(),
            &SkinSystem::new(),
        );

        assert_eq!(function_base[0].r#type, Some(func_type::LORD));

        let lord_func = build_lord_function(&lord);
        assert_eq!(lord_func.nick_name.as_deref(), Some("tester"));
        assert_eq!(lord_func.portrait, Some(7));
        assert_eq!(lord_func.diamond, Some(100));
        assert_eq!(lord_func.union_id, Some(6));
    }
}
