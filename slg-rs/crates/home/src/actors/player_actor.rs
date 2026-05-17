use prost::Message;
use proto::slg::{
    AwardPb, BaseMailPb, FunctionClientBase, GetRoleDataRs, LordDataFunction, RoleLoginRs,
    WorldBattleFighterSummaryPayload, WorldBattleResultPayload, WorldCollectReturnedPayload,
    WorldCollectStartedPayload, WorldGarrisonChangedPayload, WorldOutboundRq, WorldOutboundRs,
    WorldScoutReportRequestedPayload, WorldTroopReturnedPayload,
};
use shared::persistence::{LordRow, PlayerDao};
use shared::static_config::StaticConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{error, info, warn};

use crate::actors::global_event_bus::GlobalEventBus;
use crate::systems::activity::ActivitySystem;
use crate::systems::backpack::BackpackSystem;
use crate::systems::building::BuildingSystem;
use crate::systems::chat::ChatSystem;
use crate::systems::equip::EquipSystem;
use crate::systems::hero::HeroSystem;
use crate::systems::mail::MailSystem;
use crate::systems::mission::MissionSystem;
use crate::systems::registry::{HomeSystemRegistry, route_home_command};
use crate::systems::shop::ShopSystem;
use crate::systems::skin::SkinSystem;
use crate::systems::tech::TechSystem;
use crate::systems::vip::VipSystem;
use crate::systems::world::WorldSystem;
use shared::event::{EventDispatcher, GameEvent, MissionEvent, MissionType, PlayerContext};

/// 存盘间隔（秒）
const SAVE_INTERVAL_SECS: u64 = 300;
/// 存盘超时（秒）
const SAVE_TIMEOUT_SECS: u64 = 10;
/// 紧急存盘目录
const EMERGENCY_SAVE_DIR: &str = "./emergency_saves";
const WORLD_OUTBOUND_EVENT_SCOUT_REPORT_REQUESTED: i32 = 1;
const WORLD_OUTBOUND_EVENT_COLLECT_STARTED: i32 = 2;
const WORLD_OUTBOUND_EVENT_TROOP_RETURNED: i32 = 3;
const WORLD_OUTBOUND_EVENT_GARRISON_CHANGED: i32 = 4;
const WORLD_OUTBOUND_EVENT_COLLECT_RETURNED: i32 = 5;
const WORLD_OUTBOUND_EVENT_BATTLE_RESULT: i32 = 6;
const WORLD_AWARD_TYPE_LORD_RESOURCE: i32 = 1;
const WORLD_AWARD_TYPE_ITEM: i32 = 4;
const LORD_RESOURCE_DIAMOND: i32 = 1;
const LORD_RESOURCE_GOLD: i32 = 2;
const LORD_RESOURCE_MEAT: i32 = 3;
const LORD_RESOURCE_STAMINA: i32 = 4;
const LORD_RESOURCE_FAME: i32 = 5;
const FORMATION_STATE_IDLE: i32 = 0;

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
    /// World 服务投递的到达/出站事件
    WorldOutbound {
        event: WorldOutboundRq,
        reply: oneshot::Sender<anyhow::Result<WorldOutboundRs>>,
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
    pub chat_system: ChatSystem,
    pub tech_system: TechSystem,
    pub equip_system: EquipSystem,
    pub mail_system: MailSystem,
    pub mission_system: MissionSystem,
    pub skin_system: SkinSystem,
    pub shop_system: ShopSystem,
    pub vip_system: VipSystem,
    pub world_system: WorldSystem,

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
            chat_system: ChatSystem::new(),
            tech_system: TechSystem::new(),
            equip_system: EquipSystem::new(),
            mail_system: MailSystem::new(),
            mission_system: MissionSystem::new(),
            skin_system: SkinSystem::new(),
            shop_system: ShopSystem::new(),
            vip_system: VipSystem::new(),
            world_system: WorldSystem::new(),
            event_dispatcher: EventDispatcher::new(),
            ctx: PlayerContext {
                role_id,
                account_id,
            },
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
        let mission_blob_loaded = if let Some(row) = self.dao.load_player_data(self.role_id).await?
        {
            let mission_blob_loaded = row
                .mission_func
                .as_ref()
                .map(|data| !data.is_empty())
                .unwrap_or(false);
            self.load_registered_systems_from_row(&row);
            mission_blob_loaded
        } else {
            false
        };

        if !mission_blob_loaded && self.mission_system.is_uninitialized() {
            self.mission_system
                .init_for_new_player(&self.current_config);
        }

        // 3. 更新登录信息
        self.dao.update_login(self.role_id).await?;
        self.dao.set_log_off(self.role_id, false).await?;

        info!(role_id = self.role_id, "Player data loaded");
        Ok(())
    }

    pub async fn run(mut self) {
        info!(role_id = self.role_id, "PlayerActor started");

        if let Err(e) = self.load_data().await {
            error!(role_id = self.role_id, "Failed to load player data: {}", e);
            return;
        }
        self.state = PlayerState::Loading;

        // 触发各系统 on_login
        self.on_registered_systems_login();

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
                        PlayerMessage::WorldOutbound { event, reply } => {
                            let result = self.handle_world_outbound(event).await;
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
        self.tick_registered_systems();

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
        if force_all || self.lord_dirty {
            if let Some(lord) = &self.lord {
                if let Err(e) = self.dao.save_lord(lord).await {
                    error!(role_id = self.role_id, "Failed to save p_lord: {}", e);
                } else {
                    self.lord_dirty = false;
                }
            }
        }

        // 2. 存盘 p_data（功能模块 blob）
        let (entries, saved_columns) = self.collect_registered_save_entries(force_all);

        if entries.is_empty() {
            return;
        }

        let entry_count = entries.len();
        let save_result = tokio::time::timeout(
            Duration::from_secs(SAVE_TIMEOUT_SECS),
            self.dao.save_player_data(self.role_id, &entries),
        )
        .await;

        match save_result {
            Ok(Ok(())) => {
                for column in saved_columns {
                    self.clear_system_dirty(column);
                }
                self.save_fail_count = 0;
                info!(
                    role_id = self.role_id,
                    modules = entry_count,
                    force = force_all,
                    "Save completed"
                );
            }
            Ok(Err(e)) => {
                self.save_fail_count += 1;
                error!(
                    role_id = self.role_id,
                    fail_count = self.save_fail_count,
                    "Save failed: {}",
                    e
                );
                if self.save_fail_count >= 3 {
                    warn!(role_id = self.role_id, "Emergency save to file");
                    shared::persistence::emergency_save_to_file(
                        EMERGENCY_SAVE_DIR,
                        self.role_id,
                        &entries,
                    );
                }
            }
            Err(_) => {
                self.save_fail_count += 1;
                error!(
                    role_id = self.role_id,
                    "Save timed out ({}s)", SAVE_TIMEOUT_SECS
                );
                if self.save_fail_count >= 3 {
                    shared::persistence::emergency_save_to_file(
                        EMERGENCY_SAVE_DIR,
                        self.role_id,
                        &entries,
                    );
                }
            }
        }
    }

    fn clear_system_dirty(&mut self, column: &str) {
        if !self.clear_registered_system_dirty(column) {
            warn!(
                role_id = self.role_id,
                column, "Unknown p_data column saved"
            );
        }
    }

    async fn handle_role_login(&mut self, tx: oneshot::Sender<anyhow::Result<RoleLoginRs>>) {
        info!(role_id = self.role_id, "Role logged in");
        self.state = PlayerState::InGame;
        self.login_timestamp = chrono::Utc::now().timestamp();

        let event = GameEvent::PlayerLogin {
            role_id: self.role_id,
        };
        self.dispatch_event(&event);

        let _ = tx.send(Ok(RoleLoginRs {
            state: Some(1),
            ..Default::default()
        }));
    }

    pub fn dispatch_event(&mut self, event: &GameEvent) {
        // 1. 将语义事件自动转换为 MissionEvent，驱动任务/活动进度
        if let Some(mission_event) = event.to_mission_event() {
            let me = GameEvent::Mission(mission_event);
            self.activity_system.handle_event(&me, &mut self.ctx);
            // 带 config 的任务进度更新（能查 s_task 配置）
            self.mission_system
                .handle_event_with_config(&me, &self.current_config, &mut self.ctx);
            self.event_dispatcher.dispatch(&me, &mut self.ctx);
        }

        // 2. 原始事件也分发一遍（部分 handler 需要原始语义）
        self.activity_system.handle_event(event, &mut self.ctx);
        self.mission_system.handle_event(event, &mut self.ctx);
        self.event_dispatcher.dispatch(event, &mut self.ctx);
    }

    async fn handle_get_role_data(&mut self, tx: oneshot::Sender<anyhow::Result<GetRoleDataRs>>) {
        let _ = tx.send(Ok(self.build_role_data_response()));
    }

    fn build_role_data_response(&self) -> GetRoleDataRs {
        use shared::msg::ToFunctionClientBaseBytes;

        let mut function_base = Vec::new();

        if let Some(lord) = &self.lord {
            push_function_base(
                &mut function_base,
                Self::lord_data_function(lord).to_function_base_bytes(),
            );
        }
        self.append_registered_function_bases(&mut function_base);

        GetRoleDataRs {
            function_base,
            ..Default::default()
        }
    }

    fn lord_data_function(lord: &LordRow) -> LordDataFunction {
        LordDataFunction {
            nick_name: lord.nick.clone(),
            portrait: lord
                .portrait
                .as_deref()
                .and_then(|portrait| portrait.parse::<i32>().ok()),
            diamond: lord.diamond,
            battle_fight: lord.battle_fight,
            guide_index: lord.guide_id,
            title: lord.title,
            portrait_frame: lord.portrait_frame,
            server_open_time: Some(0),
            role_create_time: lord.on_time,
            off_time: lord.off_time,
            meat: lord.meat,
            fame: lord.fame,
            gold: lord.gold,
            search_survivor_time: lord.search_survivor_time,
            stamina: lord.stamina.map(i64_to_i32_saturating),
            total_login: lord.total_login,
            current_streak: lord.current_streak,
            vip_level: lord.vip_level,
            vip_exp: lord.vip_exp,
            ..Default::default()
        }
    }

    /// 将业务命令路由到已注册 Home 系统处理，返回序列化后的响应 payload。
    ///
    /// 命令范围注册在 `systems::registry::HOME_COMMAND_ROUTES`，避免新增 Home system
    /// 继续扩大 PlayerActor 的分发 match。
    pub async fn handle_game_command(
        &mut self,
        cmd: u32,
        payload: Vec<u8>,
    ) -> anyhow::Result<Vec<u8>> {
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

        if cmd == 1109 {
            return shared::msg::GameMessage::build_response(
                1110,
                &self.build_role_data_response(),
            );
        }

        if route_home_command(cmd).is_none() {
            warn!(
                role_id = self.role_id,
                cmd, "Unhandled cmd, no system matched"
            );
        }

        // 调用对应系统，获取响应 payload 和需要分发的事件
        let result = self.dispatch_registered_command(cmd, &payload, &config)?;

        // 分发系统产生的游戏事件（驱动任务/活动进度）
        for event in result.events {
            self.dispatch_event(&event);
        }

        shared::msg::GameMessage::build_response_from_raw(cmd as i32 + 1, &result.response_payload)
    }

    async fn handle_world_outbound(
        &mut self,
        event: WorldOutboundRq,
    ) -> anyhow::Result<WorldOutboundRs> {
        if event.role_id != self.role_id {
            warn!(
                actor_role_id = self.role_id,
                event_role_id = event.role_id,
                event_type = event.event_type,
                troop_key = event.troop_key,
                "PlayerActor rejected World outbound event for a different role"
            );
            return Ok(WorldOutboundRs {
                code: 403,
                msg: format!(
                    "role_id mismatch: actor={} event={}",
                    self.role_id, event.role_id
                ),
            });
        }

        let decoded = match decode_world_outbound_payload(&event) {
            Ok(decoded) => decoded,
            Err(err) => {
                warn!(
                    role_id = self.role_id,
                    event_type = event.event_type,
                    troop_key = event.troop_key,
                    error = %err,
                    "PlayerActor rejected invalid World outbound payload"
                );
                return Ok(WorldOutboundRs {
                    code: 400,
                    msg: err.to_string(),
                });
            }
        };
        let decoded_msg = decoded.description();
        let event_token = world_outbound_event_token(&event);

        if self.world_system.has_processed_outbound(&event_token) {
            info!(
                role_id = self.role_id,
                event_type = event.event_type,
                troop_key = event.troop_key,
                world_entity_id = event.world_entity_id,
                event_id = %event.event_id,
                event_key = %event.event_key,
                decoded = %decoded_msg,
                "PlayerActor ignored duplicate World outbound event"
            );
            return Ok(WorldOutboundRs {
                code: 0,
                msg: format!("duplicate World outbound ignored: {}", decoded_msg),
            });
        }

        match self.apply_world_outbound(&decoded) {
            Ok(events) => {
                self.world_system.mark_outbound_processed(event_token);
                for event in events {
                    self.dispatch_event(&event);
                }
            }
            Err(err) => {
                warn!(
                    role_id = self.role_id,
                    event_type = event.event_type,
                    troop_key = event.troop_key,
                    error = %err,
                    "PlayerActor failed to apply World outbound event"
                );
                return Ok(WorldOutboundRs {
                    code: 500,
                    msg: err.to_string(),
                });
            }
        }

        info!(
            role_id = self.role_id,
            event_type = event.event_type,
            troop_key = event.troop_key,
            world_entity_id = event.world_entity_id,
            event_id = %event.event_id,
            event_key = %event.event_key,
            payload_len = event.payload.len(),
            decoded = %decoded_msg,
            context = %event.context,
            "PlayerActor received World outbound event"
        );

        Ok(WorldOutboundRs {
            code: 0,
            msg: decoded_msg,
        })
    }

    fn apply_world_outbound(
        &mut self,
        event: &DecodedWorldOutbound,
    ) -> anyhow::Result<Vec<GameEvent>> {
        match event {
            DecodedWorldOutbound::ScoutReportRequested(payload) => {
                self.create_scout_report_mail(payload)?;
                Ok(Vec::new())
            }
            DecodedWorldOutbound::CollectReturned(payload) => {
                if let Some(formation_id) = payload.formation_id.filter(|id| *id > 0) {
                    self.hero_system
                        .set_formation_state(formation_id, FORMATION_STATE_IDLE);
                }
                self.apply_world_awards(&payload.awards)
            }
            DecodedWorldOutbound::BattleResult(payload) => {
                self.create_battle_report_mail(payload)?;
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn create_battle_report_mail(
        &mut self,
        payload: &WorldBattleResultPayload,
    ) -> anyhow::Result<()> {
        let now_secs = chrono::Utc::now().timestamp();
        let attacker = payload.attacker.as_ref();
        let defender = payload.defender.as_ref();

        let mut content_parts = vec![
            format!("Outcome: {}", payload.outcome),
            format!("Winner: {}", payload.winner_side),
            format!("Rounds: {}", payload.rounds),
            format!("Target Pos: {}", payload.target_pos),
        ];
        if let Some(owner_id) = payload.target_owner_id {
            content_parts.push(format!("Target Owner: {}", owner_id));
        }
        if let Some(attacker) = attacker {
            content_parts.push(format!(
                "Attacker Losses: {}/{} units, {} power",
                attacker.units_lost, attacker.initial_units, attacker.power_lost
            ));
        }
        if let Some(defender) = defender {
            content_parts.push(format!(
                "Defender Losses: {}/{} units, {} power",
                defender.units_lost, defender.initial_units, defender.power_lost
            ));
        }

        let mut c_param = vec![
            format!("battle_id:{}", payload.battle_id),
            format!("target_pos:{}", payload.target_pos),
            format!("outcome:{}", payload.outcome),
            format!("winner:{}", payload.winner_side),
            format!("rounds:{}", payload.rounds),
        ];
        push_fighter_mail_params("attacker", attacker, &mut c_param);
        push_fighter_mail_params("defender", defender, &mut c_param);

        self.mail_system.add_personal_mail(BaseMailPb {
            template_id: 201,
            r#type: 0,
            time: Some(now_secs),
            title: Some(format!("Battle Report: {}", payload.outcome)),
            content: Some(content_parts.join("\n")),
            c_param,
            ..Default::default()
        });

        Ok(())
    }

    fn create_scout_report_mail(
        &mut self,
        payload: &WorldScoutReportRequestedPayload,
    ) -> anyhow::Result<()> {
        let now_secs = chrono::Utc::now().timestamp();
        let entity_type_name = entity_type_display_name(payload.target_entity_type);

        let mut content_parts = Vec::new();
        if let Some(entity_type) = payload.target_entity_type {
            content_parts.push(format!(
                "Target Type: {} ({})",
                entity_type_name, entity_type
            ));
        }
        if let Some(owner_id) = payload.target_owner_id {
            content_parts.push(format!("Owner ID: {}", owner_id));
        }
        if let Some(camp) = payload.target_camp {
            content_parts.push(format!("Camp: {}", camp));
        }
        if let Some(is_battle) = payload.target_is_battle {
            if is_battle {
                content_parts.push("Status: In Battle".to_string());
            }
        }
        if let Some(protect_time) = payload.target_protect_time {
            if protect_time > 0 && protect_time as i64 > now_secs {
                let remaining_secs = protect_time as i64 - now_secs;
                content_parts.push(format!(
                    "Protection Shield: {}s remaining",
                    remaining_secs.max(0)
                ));
            }
        }
        if !payload.target_resources.is_empty() {
            let total: i64 = payload.target_resources.iter().map(|a| a.count).sum();
            content_parts.push(format!("Available Resources: {} units", total));
        }
        if !payload.garrison_troops.is_empty() {
            content_parts.push(format!(
                "Garrison Troops: {} stationed",
                payload.garrison_troops.len()
            ));
        }

        let content = content_parts.join("\n");
        let title = format!(
            "Scout Report: {} at pos {}",
            entity_type_name, payload.target_pos
        );

        let mut c_param = Vec::new();
        c_param.push(format!("pos:{}", payload.target_pos));
        if let Some(t) = payload.target_entity_type {
            c_param.push(format!("entity_type:{}", t));
        }
        if let Some(owner) = payload.target_owner_id {
            c_param.push(format!("owner:{}", owner));
        }
        if !payload.target_resources.is_empty() {
            for (i, res) in payload.target_resources.iter().enumerate() {
                c_param.push(format!("resource_{}:id={},count={}", i, res.id, res.count));
            }
        }
        c_param.push(format!("garrison_count:{}", payload.garrison_troops.len()));

        self.mail_system.add_personal_mail(BaseMailPb {
            template_id: 200,
            r#type: 0,
            time: Some(now_secs),
            title: Some(title),
            content: Some(content),
            c_param,
            ..Default::default()
        });

        Ok(())
    }

    fn apply_world_awards(&mut self, awards: &[AwardPb]) -> anyhow::Result<Vec<GameEvent>> {
        let mut events = Vec::new();

        for award in awards {
            if award.count <= 0 {
                continue;
            }

            match award.r#type {
                WORLD_AWARD_TYPE_LORD_RESOURCE => {
                    self.add_lord_resource(award.id, award.count)?;
                    events.push(GameEvent::Mission(MissionEvent::new(
                        self.role_id,
                        MissionType::GatherResource,
                        vec![award.id as i64, award.count],
                    )));
                }
                WORLD_AWARD_TYPE_ITEM => {
                    self.backpack_system.add_award(award.clone());
                    events.push(GameEvent::ItemGain {
                        role_id: self.role_id,
                        prop_id: award.id,
                        count: award.count,
                    });
                }
                other => {
                    warn!(
                        role_id = self.role_id,
                        award_type = other,
                        award_id = award.id,
                        "Ignoring unsupported World award type"
                    );
                }
            }
        }

        Ok(events)
    }

    pub(crate) fn lord_resource_amount(&self, resource_id: i32) -> anyhow::Result<i64> {
        let lord = self
            .lord
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("lord data not loaded"))?;
        match resource_id {
            LORD_RESOURCE_DIAMOND => Ok(lord.diamond.unwrap_or_default()),
            LORD_RESOURCE_GOLD => Ok(lord.gold.unwrap_or_default()),
            LORD_RESOURCE_MEAT => Ok(lord.meat.unwrap_or_default()),
            LORD_RESOURCE_STAMINA => Ok(lord.stamina.unwrap_or_default()),
            LORD_RESOURCE_FAME => Ok(i64::from(lord.fame.unwrap_or_default())),
            other => anyhow::bail!("unsupported lord resource id={}", other),
        }
    }

    pub(crate) fn try_consume_lord_resource(
        &mut self,
        resource_id: i32,
        amount: i64,
    ) -> anyhow::Result<()> {
        if amount <= 0 {
            return Ok(());
        }
        let have = self.lord_resource_amount(resource_id)?;
        if have < amount {
            anyhow::bail!(
                "not enough lord resource id={}: have={}, need={}",
                resource_id,
                have,
                amount
            );
        }
        self.add_lord_resource(resource_id, -amount)
    }

    pub(crate) fn grant_lord_resource(
        &mut self,
        resource_id: i32,
        amount: i64,
    ) -> anyhow::Result<()> {
        self.add_lord_resource(resource_id, amount)
    }

    fn add_lord_resource(&mut self, resource_id: i32, delta: i64) -> anyhow::Result<()> {
        if delta == 0 {
            return Ok(());
        }

        let lord = self
            .lord
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("lord data not loaded"))?;

        match resource_id {
            LORD_RESOURCE_DIAMOND => {
                add_i64_option(&mut lord.diamond, delta);
                if delta < 0 {
                    add_i64_option(&mut lord.diamond_cost, -delta);
                }
            }
            LORD_RESOURCE_GOLD => add_i64_option(&mut lord.gold, delta),
            LORD_RESOURCE_MEAT => add_i64_option(&mut lord.meat, delta),
            LORD_RESOURCE_STAMINA => add_i64_option(&mut lord.stamina, delta),
            LORD_RESOURCE_FAME => {
                let current = lord.fame.unwrap_or_default() as i64;
                lord.fame = Some((current + delta).clamp(i32::MIN as i64, i32::MAX as i64) as i32);
            }
            other => anyhow::bail!("unsupported lord resource id={}", other),
        }

        self.lord_dirty = true;
        Ok(())
    }
}

fn entity_type_display_name(entity_type: Option<i32>) -> &'static str {
    match entity_type {
        Some(1) => "Player",
        Some(2) => "Bandit",
        Some(3) => "Mine",
        Some(4) => "City",
        Some(5) => "Multiplayer Bandit",
        Some(6) => "Pirate Fleet",
        Some(7) => "Rebels NPC",
        Some(99) => "Decoration",
        Some(200) => "Hunting Trap",
        _ => "Unknown Entity",
    }
}

enum DecodedWorldOutbound {
    ScoutReportRequested(WorldScoutReportRequestedPayload),
    CollectStarted(WorldCollectStartedPayload),
    TroopReturned(WorldTroopReturnedPayload),
    GarrisonChanged(WorldGarrisonChangedPayload),
    CollectReturned(WorldCollectReturnedPayload),
    BattleResult(WorldBattleResultPayload),
}

impl DecodedWorldOutbound {
    fn description(&self) -> String {
        match self {
            Self::ScoutReportRequested(payload) => format!(
                "scout_report_requested target_pos={} origin={} camp={} entity_type={} owner={}",
                payload.target_pos,
                optional_i32(payload.origin),
                optional_i32(payload.camp),
                optional_i32(payload.target_entity_type),
                optional_i64(payload.target_owner_id),
            ),
            Self::CollectStarted(payload) => format!(
                "collect_started target_pos={} march_type={} start_time_ms={}",
                payload.target_pos,
                optional_i32(payload.march_type),
                payload.start_time_ms
            ),
            Self::TroopReturned(payload) => format!(
                "troop_returned home_pos={} march_type={}",
                payload.home_pos,
                optional_i32(payload.march_type)
            ),
            Self::GarrisonChanged(payload) => format!(
                "garrison_changed target_pos={} camp={} is_arrival={}",
                payload.target_pos,
                optional_i32(payload.camp),
                payload.is_arrival
            ),
            Self::CollectReturned(payload) => format!(
                "collect_returned target_pos={} home_pos={} awards={}",
                payload.target_pos,
                payload.home_pos,
                payload.awards.len()
            ),
            Self::BattleResult(payload) => format!(
                "battle_result battle_id={} target_pos={} outcome={} winner={} rounds={} attacker_lost={} defender_lost={}",
                payload.battle_id,
                payload.target_pos,
                payload.outcome,
                payload.winner_side,
                payload.rounds,
                payload
                    .attacker
                    .as_ref()
                    .map(|fighter| fighter.units_lost)
                    .unwrap_or_default(),
                payload
                    .defender
                    .as_ref()
                    .map(|fighter| fighter.units_lost)
                    .unwrap_or_default()
            ),
        }
    }
}

fn decode_world_outbound_payload(event: &WorldOutboundRq) -> anyhow::Result<DecodedWorldOutbound> {
    match event.event_type {
        WORLD_OUTBOUND_EVENT_SCOUT_REPORT_REQUESTED => {
            let payload = WorldScoutReportRequestedPayload::decode(event.payload.as_slice())?;
            Ok(DecodedWorldOutbound::ScoutReportRequested(payload))
        }
        WORLD_OUTBOUND_EVENT_COLLECT_STARTED => {
            let payload = WorldCollectStartedPayload::decode(event.payload.as_slice())?;
            Ok(DecodedWorldOutbound::CollectStarted(payload))
        }
        WORLD_OUTBOUND_EVENT_TROOP_RETURNED => {
            let payload = WorldTroopReturnedPayload::decode(event.payload.as_slice())?;
            Ok(DecodedWorldOutbound::TroopReturned(payload))
        }
        WORLD_OUTBOUND_EVENT_GARRISON_CHANGED => {
            let payload = WorldGarrisonChangedPayload::decode(event.payload.as_slice())?;
            Ok(DecodedWorldOutbound::GarrisonChanged(payload))
        }
        WORLD_OUTBOUND_EVENT_COLLECT_RETURNED => {
            let payload = WorldCollectReturnedPayload::decode(event.payload.as_slice())?;
            Ok(DecodedWorldOutbound::CollectReturned(payload))
        }
        WORLD_OUTBOUND_EVENT_BATTLE_RESULT => {
            let payload = WorldBattleResultPayload::decode(event.payload.as_slice())?;
            Ok(DecodedWorldOutbound::BattleResult(payload))
        }
        other => Err(anyhow::anyhow!(
            "unknown World outbound event_type={}",
            other
        )),
    }
}

fn world_outbound_event_token(event: &WorldOutboundRq) -> String {
    if !event.event_id.is_empty() {
        return format!("id:{}", event.event_id);
    }
    if !event.event_key.is_empty() {
        return format!("key:{}", event.event_key);
    }

    format!(
        "legacy:role={}:type={}:entity={}:troop={}:payload={}",
        event.role_id,
        event.event_type,
        event.world_entity_id,
        event.troop_key,
        stable_bytes_hash(&event.payload)
    )
}

fn stable_bytes_hash(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:016x}", hash)
}

fn add_i64_option(value: &mut Option<i64>, delta: i64) {
    *value = Some(value.unwrap_or_default().saturating_add(delta));
}

fn optional_i32(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn push_fighter_mail_params(
    prefix: &str,
    fighter: Option<&WorldBattleFighterSummaryPayload>,
    c_param: &mut Vec<String>,
) {
    let Some(fighter) = fighter else {
        return;
    };
    c_param.push(format!("{}_fighter_id:{}", prefix, fighter.fighter_id));
    c_param.push(format!(
        "{}_initial_units:{}",
        prefix, fighter.initial_units
    ));
    c_param.push(format!(
        "{}_remaining_units:{}",
        prefix, fighter.remaining_units
    ));
    c_param.push(format!("{}_units_lost:{}", prefix, fighter.units_lost));
    c_param.push(format!("{}_power_lost:{}", prefix, fighter.power_lost));
    c_param.push(format!(
        "{}_loss_rate_bps:{}",
        prefix, fighter.loss_rate_bps
    ));
}

fn push_function_base(function_base: &mut Vec<FunctionClientBase>, bytes: Vec<u8>) {
    if let Ok(f_base) = FunctionClientBase::decode(bytes.as_slice()) {
        function_base.push(f_base);
    }
}

fn i64_to_i32_saturating(value: i64) -> i32 {
    value.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::PlayerSystem;
    use proto::slg::{
        BaseMailPb, GetMailListRq, GetMailListRs, GetRoleDataRq, ShopBuyRq, ShopBuyRs,
    };
    use shared::msg::{GameMessage, func_type};
    use shared::static_config::shop::{StaticShop, StaticShopProp};
    use sqlx::mysql::MySqlPoolOptions;
    use std::collections::HashMap;

    fn test_lord(role_id: i64) -> LordRow {
        LordRow {
            role_id,
            nick: Some("tester".to_string()),
            portrait: Some("1001".to_string()),
            portrait_frame: Some(2),
            top_up: None,
            diamond: Some(100),
            diamond_cost: Some(0),
            guide_id: Some(3),
            on_time: Some(1_700_000_000),
            ol_time: Some(0),
            off_time: Some(0),
            ol_month: Some(0),
            title: Some(0),
            max_key: Some(0),
            role_status: None,
            across_day_deal_time: Some(0),
            battle_fight: Some(12_345),
            meat: Some(200),
            fame: Some(7),
            gold: Some(300),
            search_survivor_time: Some(0),
            stamina: Some(120),
            start_ad_time: Some(0),
            start_ad_id: Some(0),
            is_add_login: Some(1),
            total_login: Some(1),
            current_streak: Some(1),
            vip_level: Some(0),
            vip_exp: Some(0),
            camp_id: Some(1),
            last_periodic_task_time: None,
            lord_system_setting: None,
            pay_amount: Some(0),
            language: None,
            push_switch: None,
        }
    }

    fn test_actor(account_id: i64, role_id: i64) -> PlayerActor {
        let (_msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (global_tx, _global_rx) = mpsc::channel(8);
        let (_config_tx, config_rx) = watch::channel(Arc::new(StaticConfig::default()));
        let pool = MySqlPoolOptions::new()
            .connect_lazy("mysql://root:password@127.0.0.1/slg_test")
            .expect("lazy mysql pool should not connect during this test");
        let mut actor = PlayerActor::new(
            account_id,
            role_id,
            msg_rx,
            GlobalEventBus::new(global_tx),
            config_rx,
            Arc::new(PlayerDao::new(pool)),
        );
        actor.lord = Some(test_lord(role_id));
        actor
    }

    #[tokio::test]
    async fn role_login_then_dispatch_1109_returns_get_role_data() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);

        let (login_tx, login_rx) = oneshot::channel();
        actor.handle_role_login(login_tx).await;
        let login_rs = login_rx.await.unwrap().unwrap();
        assert_eq!(login_rs.state, Some(1));
        assert_eq!(actor.state, PlayerState::InGame);

        let request_payload = GameMessage::build_response(1109, &GetRoleDataRq::default()).unwrap();
        let response_payload = actor
            .handle_game_command(1109, request_payload)
            .await
            .unwrap();

        let response = GameMessage::decode(response_payload).unwrap();
        assert_eq!(response.base.cmd, 1110);
        let body: GetRoleDataRs = response.get_payload().unwrap();

        let mut function_types: Vec<i32> = body
            .function_base
            .iter()
            .filter_map(|base| base.r#type)
            .collect();
        function_types.sort_unstable();

        assert!(function_types.contains(&func_type::LORD));
        assert!(function_types.contains(&func_type::BAG));
        assert!(function_types.contains(&func_type::SIM));
        assert!(function_types.contains(&func_type::MAIL));
        assert!(function_types.contains(&func_type::CHAT));
        assert!(function_types.contains(&func_type::MISSION));
        assert!(function_types.contains(&func_type::SHOP));
        assert!(function_types.contains(&func_type::VIP));
    }

    #[tokio::test]
    async fn mail_command_route_returns_mail_list() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        actor.mail_system.add_personal_mail(BaseMailPb {
            template_id: 100,
            r#type: 0,
            time: Some(1_700_000_001),
            title: Some("Scout report".to_string()),
            content: Some("pending scout report body".to_string()),
            ..Default::default()
        });

        let request_payload = GameMessage::build_response(6001, &GetMailListRq::default()).unwrap();
        let response_payload = actor
            .handle_game_command(6001, request_payload)
            .await
            .unwrap();

        let response = GameMessage::decode(response_payload).unwrap();
        assert_eq!(response.base.cmd, 6002);
        let body: GetMailListRs = response.get_payload().unwrap();
        assert_eq!(body.mail_pb.len(), 1);
        assert_eq!(body.mail_pb[0].title.as_deref(), Some("Scout report"));
        assert!(actor.mail_system.is_dirty());
    }

    fn test_shop_config(price: &str) -> shared::static_config::ShopConfig {
        let mut shops = HashMap::new();
        shops.insert(
            1,
            StaticShop {
                id: 1,
                name: Some("basic".to_string()),
                shop_type: Some(3),
                show_type: None,
                show_res: None,
                limited_time: None,
                manual_refresh: None,
                free_refresh: None,
                refresh_need: None,
                slot_num: Some(1),
                refresh_type: None,
                sort: Some(1),
                field_configuration: None,
                group_str: None,
                function_open: None,
                form_id: None,
            },
        );
        shared::static_config::ShopConfig {
            shops,
            shop_props: vec![StaticShopProp {
                id: 1001,
                shop_id: Some(1),
                dsc: None,
                show_type: None,
                prop: Some("[4,2001,2]".to_string()),
                group_val: None,
                weight: None,
                price: Some(price.to_string()),
                count: None,
                single_limit: Some(10),
                discount: None,
                unlock_time: None,
                feature_lv: None,
                sort: Some(1),
            }],
            props_by_shop_idx: HashMap::from([(1, vec![0])]),
        }
    }

    #[tokio::test]
    async fn shop_buy_deducts_lord_resource_and_grants_item() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        actor.current_config = Arc::new(StaticConfig {
            shop: test_shop_config("[1,3,50]"),
            ..Default::default()
        });
        actor
            .shop_system
            .ensure_configured(&actor.current_config.shop);

        let request_payload = GameMessage::build_response(
            7651,
            &ShopBuyRq {
                shop_id: 1,
                slot: 1,
                buy_count: 1,
            },
        )
        .unwrap();
        let response_payload = actor
            .handle_game_command(7651, request_payload)
            .await
            .unwrap();

        let response = GameMessage::decode(response_payload).unwrap();
        assert_eq!(response.base.cmd, 7652);
        let body: ShopBuyRs = response.get_payload().unwrap();
        assert_eq!(body.shop_id, Some(1));
        assert_eq!(body.item.as_ref().unwrap().purchased_count, Some(1));
        assert_eq!(actor.lord.as_ref().unwrap().meat, Some(150));
        assert_eq!(actor.backpack_system.get_item_count(2001), 2);
        assert_eq!(
            actor.shop_system.find_item(1, 1).unwrap().purchased_count,
            Some(1)
        );
        assert!(actor.lord_dirty);
        assert!(actor.backpack_system.is_dirty());
        assert!(actor.shop_system.is_dirty());
    }

    #[tokio::test]
    async fn shop_buy_rejects_when_lord_resource_is_not_enough() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        actor.current_config = Arc::new(StaticConfig {
            shop: test_shop_config("[1,3,500]"),
            ..Default::default()
        });
        actor
            .shop_system
            .ensure_configured(&actor.current_config.shop);

        let request_payload = GameMessage::build_response(
            7651,
            &ShopBuyRq {
                shop_id: 1,
                slot: 1,
                buy_count: 1,
            },
        )
        .unwrap();
        let result = actor.handle_game_command(7651, request_payload).await;

        assert!(result.is_err());
        assert_eq!(actor.lord.as_ref().unwrap().meat, Some(200));
        assert_eq!(actor.backpack_system.get_item_count(2001), 0);
        assert_eq!(
            actor.shop_system.find_item(1, 1).unwrap().purchased_count,
            Some(0)
        );
        assert!(!actor.lord_dirty);
        assert!(!actor.backpack_system.is_dirty());
        assert!(!actor.shop_system.is_dirty());
    }

    #[tokio::test]
    async fn shop_buy_accepts_legacy_lord_resource_award_type() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        actor.current_config = Arc::new(StaticConfig {
            shop: test_shop_config("[2,3,50]"),
            ..Default::default()
        });
        actor
            .shop_system
            .ensure_configured(&actor.current_config.shop);

        let request_payload = GameMessage::build_response(
            7651,
            &ShopBuyRq {
                shop_id: 1,
                slot: 1,
                buy_count: 1,
            },
        )
        .unwrap();
        actor
            .handle_game_command(7651, request_payload)
            .await
            .unwrap();

        assert_eq!(actor.lord.as_ref().unwrap().meat, Some(150));
        assert_eq!(actor.backpack_system.get_item_count(2001), 2);
    }

    #[tokio::test]
    async fn world_outbound_decodes_typed_collect_payload() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldCollectStartedPayload {
            target_pos: 202,
            march_type: Some(3),
            start_time_ms: 12_000,
        }
        .encode_to_vec();

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id,
                event_type: WORLD_OUTBOUND_EVENT_COLLECT_STARTED,
                world_entity_id: 202,
                troop_key: 44,
                payload,
                context: "test".to_string(),
                event_id: "test-collect-started-44".to_string(),
                event_key: "test:collect_started:44".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 0);
        assert!(rs.msg.contains("collect_started"));
        assert!(rs.msg.contains("target_pos=202"));
    }

    #[tokio::test]
    async fn world_outbound_rejects_invalid_payload() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id,
                event_type: WORLD_OUTBOUND_EVENT_TROOP_RETURNED,
                world_entity_id: 101,
                troop_key: 45,
                payload: vec![0x80],
                context: "test".to_string(),
                event_id: "test-invalid-45".to_string(),
                event_key: "test:invalid:45".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 400);
        assert!(rs.msg.contains("invalid"));
    }

    #[tokio::test]
    async fn world_outbound_rejects_role_id_mismatch_before_applying_rewards() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldCollectReturnedPayload {
            target_pos: 202,
            home_pos: 101,
            march_type: Some(3),
            formation_id: None,
            awards: vec![AwardPb {
                r#type: WORLD_AWARD_TYPE_LORD_RESOURCE,
                id: LORD_RESOURCE_MEAT,
                count: 50,
                safe: Some(true),
                ..Default::default()
            }],
            collect_start_time_ms: 12_000,
            collect_end_time_ms: 12_500,
        }
        .encode_to_vec();

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id: role_id + 1,
                event_type: WORLD_OUTBOUND_EVENT_COLLECT_RETURNED,
                world_entity_id: 202,
                troop_key: 46,
                payload,
                context: "test".to_string(),
                event_id: "test-collect-returned-mismatch-46".to_string(),
                event_key: "test:collect_returned:mismatch:46".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 403);
        assert!(rs.msg.contains("role_id mismatch"));
        assert_eq!(actor.lord.as_ref().unwrap().meat, Some(200));
        assert!(!actor.lord_dirty);
    }

    #[tokio::test]
    async fn world_outbound_collect_returned_applies_lord_resource_and_formation() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        actor.hero_system.formations.push(proto::slg::Formation {
            id: 7,
            state: 1,
            hero_id: vec![101],
            ..Default::default()
        });
        let payload = WorldCollectReturnedPayload {
            target_pos: 202,
            home_pos: 101,
            march_type: Some(3),
            formation_id: Some(7),
            awards: vec![AwardPb {
                r#type: WORLD_AWARD_TYPE_LORD_RESOURCE,
                id: LORD_RESOURCE_MEAT,
                count: 50,
                safe: Some(true),
                ..Default::default()
            }],
            collect_start_time_ms: 12_000,
            collect_end_time_ms: 12_500,
        }
        .encode_to_vec();

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id,
                event_type: WORLD_OUTBOUND_EVENT_COLLECT_RETURNED,
                world_entity_id: 202,
                troop_key: 46,
                payload,
                context: "test".to_string(),
                event_id: "test-collect-returned-46".to_string(),
                event_key: "test:collect_returned:46".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 0);
        assert!(rs.msg.contains("collect_returned"));
        assert_eq!(actor.lord.as_ref().unwrap().meat, Some(250));
        assert!(actor.lord_dirty);
        assert_eq!(actor.hero_system.formations[0].state, FORMATION_STATE_IDLE);
        assert!(actor.hero_system.is_dirty());
    }

    #[tokio::test]
    async fn world_outbound_collect_returned_duplicate_does_not_apply_twice() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        actor.hero_system.formations.push(proto::slg::Formation {
            id: 7,
            state: 1,
            hero_id: vec![101],
            ..Default::default()
        });
        let payload = WorldCollectReturnedPayload {
            target_pos: 202,
            home_pos: 101,
            march_type: Some(3),
            formation_id: Some(7),
            awards: vec![AwardPb {
                r#type: WORLD_AWARD_TYPE_LORD_RESOURCE,
                id: LORD_RESOURCE_MEAT,
                count: 50,
                safe: Some(true),
                ..Default::default()
            }],
            collect_start_time_ms: 12_000,
            collect_end_time_ms: 12_500,
        }
        .encode_to_vec();
        let request = WorldOutboundRq {
            role_id,
            event_type: WORLD_OUTBOUND_EVENT_COLLECT_RETURNED,
            world_entity_id: 202,
            troop_key: 46,
            payload,
            context: "test".to_string(),
            event_id: "test-collect-returned-dedup-46".to_string(),
            event_key: "test:collect_returned:dedup:46".to_string(),
        };

        let first = actor.handle_world_outbound(request.clone()).await.unwrap();
        let duplicate = actor.handle_world_outbound(request).await.unwrap();

        assert_eq!(first.code, 0);
        assert_eq!(duplicate.code, 0);
        assert!(duplicate.msg.contains("duplicate"));
        assert_eq!(actor.lord.as_ref().unwrap().meat, Some(250));
        assert_eq!(actor.hero_system.formations[0].state, FORMATION_STATE_IDLE);
        assert!(
            actor
                .world_system
                .has_processed_outbound("id:test-collect-returned-dedup-46")
        );
    }

    #[tokio::test]
    async fn world_outbound_collect_returned_applies_item_award() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldCollectReturnedPayload {
            target_pos: 202,
            home_pos: 101,
            march_type: Some(3),
            formation_id: None,
            awards: vec![AwardPb {
                r#type: WORLD_AWARD_TYPE_ITEM,
                id: 1001,
                count: 2,
                safe: Some(true),
                ..Default::default()
            }],
            collect_start_time_ms: 12_000,
            collect_end_time_ms: 12_500,
        }
        .encode_to_vec();

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id,
                event_type: WORLD_OUTBOUND_EVENT_COLLECT_RETURNED,
                world_entity_id: 202,
                troop_key: 47,
                payload,
                context: "test".to_string(),
                event_id: "test-collect-returned-47".to_string(),
                event_key: "test:collect_returned:47".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 0);
        assert_eq!(actor.backpack_system.get_item_count(1001), 2);
        assert!(actor.backpack_system.is_dirty());
    }

    #[tokio::test]
    async fn world_outbound_scout_report_creates_mail() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldScoutReportRequestedPayload {
            origin: Some(1),
            target_pos: 303,
            camp: Some(1),
            target_entity_type: Some(1), // Player
            target_owner_id: Some(900_002),
            target_camp: Some(2),
            target_conf_id: Some(7),
            target_is_battle: Some(true),
            target_protect_time: Some(1_700_000_500),
            scout_time_ms: Some(1_700_000_000),
            target_resources: vec![],
            garrison_troops: vec![],
        }
        .encode_to_vec();

        let mail_count_before = actor.mail_system.mails.len();

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id,
                event_type: WORLD_OUTBOUND_EVENT_SCOUT_REPORT_REQUESTED,
                world_entity_id: 303,
                troop_key: 100,
                payload,
                context: "scout test".to_string(),
                event_id: "test-scout-report-100".to_string(),
                event_key: "test:scout_report:100".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 0);
        assert!(rs.msg.contains("scout_report_requested"));
        assert_eq!(actor.mail_system.mails.len(), mail_count_before + 1);

        let mail = actor.mail_system.mails.last().unwrap();
        assert_eq!(mail.template_id, 200);
        assert_eq!(mail.r#type, 0);
        assert!(mail.title.as_deref().unwrap().contains("Scout Report"));
        assert!(mail.title.as_deref().unwrap().contains("pos 303"));
        assert!(mail.content.as_deref().unwrap().contains("Player"));
        assert!(
            mail.content
                .as_deref()
                .unwrap()
                .contains("Owner ID: 900002")
        );
        assert!(mail.c_param.iter().any(|p| p.contains("pos:303")));
        assert!(actor.mail_system.is_dirty());
    }

    #[tokio::test]
    async fn world_outbound_scout_report_duplicate_does_not_create_duplicate_mail() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldScoutReportRequestedPayload {
            origin: Some(1),
            target_pos: 404,
            camp: Some(1),
            target_entity_type: Some(3), // Mine
            target_owner_id: None,
            target_camp: None,
            target_conf_id: Some(201),
            target_is_battle: None,
            target_protect_time: None,
            scout_time_ms: Some(1_700_000_000),
            target_resources: vec![],
            garrison_troops: vec![],
        }
        .encode_to_vec();
        let request = WorldOutboundRq {
            role_id,
            event_type: WORLD_OUTBOUND_EVENT_SCOUT_REPORT_REQUESTED,
            world_entity_id: 404,
            troop_key: 200,
            payload,
            context: "scout test".to_string(),
            event_id: "test-scout-dedup-200".to_string(),
            event_key: "test:scout_dedup:200".to_string(),
        };

        let first = actor.handle_world_outbound(request.clone()).await.unwrap();
        let duplicate = actor.handle_world_outbound(request).await.unwrap();

        assert_eq!(first.code, 0);
        assert_eq!(duplicate.code, 0);
        assert!(duplicate.msg.contains("duplicate"));
        // Only one mail should be created
        assert_eq!(actor.mail_system.mails.len(), 1);
        assert!(
            actor
                .world_system
                .has_processed_outbound("id:test-scout-dedup-200")
        );
    }

    #[tokio::test]
    async fn world_outbound_battle_result_creates_report_mail() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldBattleResultPayload {
            battle_id: 99,
            target_pos: 606,
            origin: Some(101),
            march_type: Some(1),
            camp: Some(1),
            outcome: "AttackerWin".to_string(),
            winner_side: "Attacker".to_string(),
            rounds: 3,
            total_events: 5,
            attacker: Some(WorldBattleFighterSummaryPayload {
                fighter_id: 11,
                initial_units: 100,
                remaining_units: 80,
                units_lost: 20,
                initial_power: 3_600,
                remaining_power: 2_880,
                power_lost: 720,
                damage_dealt: 1_200,
                damage_taken: 600,
                loss_rate_bps: 2_000,
            }),
            defender: Some(WorldBattleFighterSummaryPayload {
                fighter_id: 22,
                initial_units: 90,
                remaining_units: 0,
                units_lost: 90,
                initial_power: 2_700,
                remaining_power: 0,
                power_lost: 2_700,
                damage_dealt: 600,
                damage_taken: 1_200,
                loss_rate_bps: 10_000,
            }),
            target_owner_id: Some(900_002),
            target_entity_type: Some(1),
        }
        .encode_to_vec();

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id,
                event_type: WORLD_OUTBOUND_EVENT_BATTLE_RESULT,
                world_entity_id: 606,
                troop_key: 400,
                payload,
                context: "battle test".to_string(),
                event_id: "test-battle-result-400".to_string(),
                event_key: "test:battle_result:400".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 0);
        assert!(rs.msg.contains("battle_result"));
        assert_eq!(actor.mail_system.mails.len(), 1);
        let mail = actor.mail_system.mails.last().unwrap();
        assert_eq!(mail.template_id, 201);
        assert!(mail.title.as_deref().unwrap().contains("Battle Report"));
        assert!(mail.content.as_deref().unwrap().contains("AttackerWin"));
        assert!(mail.content.as_deref().unwrap().contains("Rounds: 3"));
        assert!(
            mail.content
                .as_deref()
                .unwrap()
                .contains("Attacker Losses: 20/100")
        );
        assert!(
            mail.content
                .as_deref()
                .unwrap()
                .contains("Defender Losses: 90/90")
        );
        assert!(mail.c_param.iter().any(|p| p == "battle_id:99"));
        assert!(mail.c_param.iter().any(|p| p == "attacker_units_lost:20"));
        assert!(actor.mail_system.is_dirty());
    }

    #[tokio::test]
    async fn world_outbound_battle_result_duplicate_does_not_create_duplicate_mail() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldBattleResultPayload {
            battle_id: 100,
            target_pos: 707,
            origin: Some(101),
            march_type: Some(1),
            camp: Some(1),
            outcome: "DefenderWin".to_string(),
            winner_side: "Defender".to_string(),
            rounds: 2,
            total_events: 4,
            attacker: Some(WorldBattleFighterSummaryPayload {
                fighter_id: 11,
                initial_units: 100,
                remaining_units: 0,
                units_lost: 100,
                initial_power: 3_600,
                remaining_power: 0,
                power_lost: 3_600,
                damage_dealt: 500,
                damage_taken: 1_000,
                loss_rate_bps: 10_000,
            }),
            defender: Some(WorldBattleFighterSummaryPayload {
                fighter_id: 22,
                initial_units: 90,
                remaining_units: 50,
                units_lost: 40,
                initial_power: 2_700,
                remaining_power: 1_500,
                power_lost: 1_200,
                damage_dealt: 1_000,
                damage_taken: 500,
                loss_rate_bps: 4_444,
            }),
            target_owner_id: Some(900_002),
            target_entity_type: Some(1),
        }
        .encode_to_vec();
        let request = WorldOutboundRq {
            role_id,
            event_type: WORLD_OUTBOUND_EVENT_BATTLE_RESULT,
            world_entity_id: 707,
            troop_key: 401,
            payload,
            context: "battle test".to_string(),
            event_id: "test-battle-dedup-401".to_string(),
            event_key: "test:battle_dedup:401".to_string(),
        };

        let first = actor.handle_world_outbound(request.clone()).await.unwrap();
        let duplicate = actor.handle_world_outbound(request).await.unwrap();

        assert_eq!(first.code, 0);
        assert_eq!(duplicate.code, 0);
        assert!(duplicate.msg.contains("duplicate"));
        assert_eq!(actor.mail_system.mails.len(), 1);
        assert!(
            actor
                .world_system
                .has_processed_outbound("id:test-battle-dedup-401")
        );
    }

    #[tokio::test]
    async fn world_outbound_scout_report_with_garrison_includes_garrison_in_mail() {
        let role_id = 900_001;
        let mut actor = test_actor(700_001, role_id);
        let payload = WorldScoutReportRequestedPayload {
            origin: Some(1),
            target_pos: 505,
            camp: Some(1),
            target_entity_type: Some(1), // Player
            target_owner_id: Some(900_003),
            target_camp: Some(2),
            target_conf_id: None,
            target_is_battle: Some(false),
            target_protect_time: None,
            scout_time_ms: Some(1_700_000_000),
            target_resources: vec![],
            garrison_troops: vec![proto::slg::GarrisonTroop {
                troop_key_id: Some(301),
                role_id: Some(900_004),
                end_time: Some(99_000),
                ..Default::default()
            }],
        }
        .encode_to_vec();

        let rs = actor
            .handle_world_outbound(WorldOutboundRq {
                role_id,
                event_type: WORLD_OUTBOUND_EVENT_SCOUT_REPORT_REQUESTED,
                world_entity_id: 505,
                troop_key: 300,
                payload,
                context: "scout test".to_string(),
                event_id: "test-scout-garrison-300".to_string(),
                event_key: "test:scout_garrison:300".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(rs.code, 0);
        assert_eq!(actor.mail_system.mails.len(), 1);
        let mail = actor.mail_system.mails.last().unwrap();
        assert!(
            mail.content
                .as_deref()
                .unwrap()
                .contains("Garrison Troops: 1")
        );
        assert!(mail.c_param.iter().any(|p| p == "garrison_count:1"));
    }
}
