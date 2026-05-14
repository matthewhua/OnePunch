use tokio::sync::{mpsc, oneshot, watch};
use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Result};
use prost::Message;
use tracing::{info, warn, error};
use proto::slg::{
    AwardPb, ChangeInfo, FunctionClientBase, GainDailyLivenessRewardRs, GetRoleDataRs, PropUseRq,
    PropUseRs, ReceiveChapterMissionRewardBatchRs, ReceiveChapterMissionRs,
    ReceiveMissionRewardRs, RoleLoginRs, TechnologyResearchRq, TechnologySpeedUpRq,
};
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
const AWARD_TYPE_DIAMOND: i32 = 1;
const AWARD_TYPE_RESOURCE: i32 = 2;
const AWARD_TYPE_ITEM: i32 = 4;
const AWARD_ID_DIAMOND: i32 = 3;
const RESOURCE_ID_MEAT: i32 = 3;
const RESOURCE_ID_GOLD: i32 = 6;
const RESOURCE_ID_FAME: i32 = 11;
const VIRTUAL_DAILY_LIVENESS_PROP_ID: i32 = 4002;

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
                        PlayerMessage::DispatchGameEvent(mut event) => {
                            bind_event_role_id(&mut event, self.role_id);
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
        let mut tech_events = self.tech_system.check_research_complete(self.role_id, now);
        for event in &mut tech_events {
            bind_event_role_id(event, self.role_id);
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
        let pending_economy = self.prepare_command_economy(cmd, &payload, &config)?;

        let (mut resp, mut events) = match cmd {
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
        let mut economy_events = self.apply_command_economy(pending_economy)?;
        economy_events.extend(self.apply_command_awards(cmd, &payload, &mut resp, &config)?);
        economy_events.append(&mut events);

        for event in &mut economy_events {
            bind_event_role_id(event, self.role_id);
            self.dispatch_event(event);
        }

        shared::msg::GameMessage::build_response_from_raw(cmd as i32 + 1, &resp)
    }

    fn prepare_command_economy(
        &self,
        cmd: u32,
        payload: &[u8],
        config: &StaticConfig,
    ) -> Result<CommandEconomy> {
        let mut costs = Vec::new();
        match cmd {
            4201 => {
                let rq = TechnologyResearchRq::decode(payload)
                    .map_err(|e| anyhow!("Decode TechnologyResearchRq for economy: {}", e))?;
                if rq.technology_id > 0 && rq.stage > 0 && matches!(rq.r#type, 1 | 2) {
                    costs.extend(self.tech_research_actor_costs(&rq, config)?);
                }
            }
            4203 => {
                let rq = TechnologySpeedUpRq::decode(payload)
                    .map_err(|e| anyhow!("Decode TechnologySpeedUpRq for economy: {}", e))?;
                if rq.r#type == 1 {
                    let prop_id = rq
                        .prop_id
                        .ok_or_else(|| anyhow!("TechnologySpeedUpRq missing propId"))?;
                    let use_number = rq
                        .use_number
                        .ok_or_else(|| anyhow!("TechnologySpeedUpRq missing useNumber"))?;
                    costs.push(AwardPb {
                        r#type: AWARD_TYPE_ITEM,
                        id: prop_id,
                        count: i64::from(use_number),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }

        self.ensure_can_consume_awards(&costs)?;
        Ok(CommandEconomy { costs })
    }

    fn apply_command_economy(&mut self, economy: CommandEconomy) -> Result<Vec<GameEvent>> {
        self.consume_awards(&economy.costs)
    }

    fn apply_command_awards(
        &mut self,
        cmd: u32,
        payload: &[u8],
        resp: &mut Vec<u8>,
        config: &StaticConfig,
    ) -> Result<Vec<GameEvent>> {
        match cmd {
            4003 => self.apply_prop_use_awards(payload, resp, config),
            1179 => {
                let rs = ReceiveMissionRewardRs::decode(resp.as_slice())
                    .map_err(|e| anyhow!("Decode ReceiveMissionRewardRs: {}", e))?;
                self.grant_change_info_awards(rs.info.as_ref())
            }
            1181 => {
                let rs = ReceiveChapterMissionRs::decode(resp.as_slice())
                    .map_err(|e| anyhow!("Decode ReceiveChapterMissionRs: {}", e))?;
                self.grant_change_info_awards(rs.info.as_ref())
            }
            1183 => {
                let rs = GainDailyLivenessRewardRs::decode(resp.as_slice())
                    .map_err(|e| anyhow!("Decode GainDailyLivenessRewardRs: {}", e))?;
                self.grant_change_info_awards(rs.info.as_ref())
            }
            1189 => {
                let rs = ReceiveChapterMissionRewardBatchRs::decode(resp.as_slice())
                    .map_err(|e| anyhow!("Decode ReceiveChapterMissionRewardBatchRs: {}", e))?;
                self.grant_change_info_awards(rs.info.as_ref())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn apply_prop_use_awards(
        &mut self,
        payload: &[u8],
        resp: &mut Vec<u8>,
        config: &StaticConfig,
    ) -> Result<Vec<GameEvent>> {
        let rq = PropUseRq::decode(payload)
            .map_err(|e| anyhow!("Decode PropUseRq for economy: {}", e))?;
        let Some(prop) = config.item.props.get(&rq.prop_id) else {
            return Ok(Vec::new());
        };
        let mut awards = parse_award_list(prop.reward_list.as_deref().unwrap_or_default());
        multiply_awards(&mut awards, i64::from(rq.use_count));
        if awards.is_empty() {
            return Ok(Vec::new());
        }

        let events = self.grant_awards(&awards)?;

        let mut rs = PropUseRs::decode(resp.as_slice())
            .map_err(|e| anyhow!("Decode PropUseRs for economy: {}", e))?;
        let change_info = rs.change_info.get_or_insert_with(ChangeInfo::default);
        change_info.show_award.extend(awards);
        *resp = rs.encode_to_vec();

        Ok(events)
    }

    fn grant_change_info_awards(&mut self, info: Option<&ChangeInfo>) -> Result<Vec<GameEvent>> {
        let Some(info) = info else {
            return Ok(Vec::new());
        };
        self.grant_awards(&info.show_award)
    }

    fn tech_research_actor_costs(
        &self,
        rq: &TechnologyResearchRq,
        config: &StaticConfig,
    ) -> Result<Vec<AwardPb>> {
        if config.tech.tech_levels.is_empty() {
            return Ok(Vec::new());
        }

        let next_level = tech_level_at_stage(&self.tech_system, rq.technology_id, rq.stage) + 1;
        let target = config
            .tech
            .tech_levels
            .values()
            .find(|tech| {
                tech.tech_id == rq.technology_id
                    && parse_i32(tech.tech_stage.trim()) == Some(rq.stage)
                    && tech.level == next_level
            })
            .ok_or_else(|| {
                anyhow!(
                    "No tech config for tech {} stage {} level {}",
                    rq.technology_id,
                    rq.stage,
                    next_level
                )
            })?;

        Ok(actor_owned_awards(
            parse_award_list(target.up_need_resource.as_deref().unwrap_or_default()),
            "tech research cost",
        ))
    }

    fn ensure_can_consume_awards(&self, awards: &[AwardPb]) -> Result<()> {
        for award in awards.iter().filter(|award| award.count > 0) {
            match (award.r#type, award.id) {
                (AWARD_TYPE_DIAMOND, AWARD_ID_DIAMOND) => {
                    let have = self.lord.as_ref().and_then(|lord| lord.diamond).unwrap_or(0);
                    ensure_enough("diamond", have, award.count)?;
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_MEAT) => {
                    let have = self.lord.as_ref().and_then(|lord| lord.meat).unwrap_or(0);
                    ensure_enough("meat", have, award.count)?;
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_GOLD) => {
                    let have = self.lord.as_ref().and_then(|lord| lord.gold).unwrap_or(0);
                    ensure_enough("gold", have, award.count)?;
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_FAME) => {
                    let have = self
                        .lord
                        .as_ref()
                        .and_then(|lord| lord.fame)
                        .map(i64::from)
                        .unwrap_or(0);
                    ensure_enough("fame", have, award.count)?;
                }
                (AWARD_TYPE_ITEM, _) => {
                    let have = self.backpack_system.get_item_count(award.id);
                    ensure_enough("item", have, award.count)?;
                }
                _ => return Err(anyhow!("Unsupported actor economy award type={} id={}", award.r#type, award.id)),
            }
        }
        Ok(())
    }

    fn consume_awards(&mut self, awards: &[AwardPb]) -> Result<Vec<GameEvent>> {
        self.ensure_can_consume_awards(awards)?;
        let mut events = Vec::new();
        for award in awards.iter().filter(|award| award.count > 0) {
            match (award.r#type, award.id) {
                (AWARD_TYPE_DIAMOND, AWARD_ID_DIAMOND) => {
                    let lord = self.lord_mut()?;
                    lord.diamond = Some(lord.diamond.unwrap_or(0) - award.count);
                    lord.diamond_cost = Some(lord.diamond_cost.unwrap_or(0) + award.count);
                    self.lord_dirty = true;
                    events.push(GameEvent::DiamondConsume {
                        role_id: self.role_id,
                        amount: award.count,
                    });
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_MEAT) => {
                    let lord = self.lord_mut()?;
                    lord.meat = Some(lord.meat.unwrap_or(0) - award.count);
                    self.lord_dirty = true;
                    events.push(GameEvent::ResourceChange {
                        role_id: self.role_id,
                        resource_type: RESOURCE_ID_MEAT,
                        delta: -award.count,
                    });
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_GOLD) => {
                    let lord = self.lord_mut()?;
                    lord.gold = Some(lord.gold.unwrap_or(0) - award.count);
                    self.lord_dirty = true;
                    events.push(GameEvent::GoldConsume {
                        role_id: self.role_id,
                        amount: award.count,
                    });
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_FAME) => {
                    let lord = self.lord_mut()?;
                    let have = i64::from(lord.fame.unwrap_or(0));
                    lord.fame = Some(i32::try_from(have - award.count)?);
                    self.lord_dirty = true;
                    events.push(GameEvent::ResourceChange {
                        role_id: self.role_id,
                        resource_type: RESOURCE_ID_FAME,
                        delta: -award.count,
                    });
                }
                (AWARD_TYPE_ITEM, _) => {
                    let (ok, mut item_events) =
                        self.backpack_system.consume_item_with_event(self.role_id, award.id, award.count);
                    if !ok {
                        return Err(anyhow!("Insufficient item {} after validation", award.id));
                    }
                    events.append(&mut item_events);
                }
                _ => {}
            }
        }
        Ok(events)
    }

    fn grant_awards(&mut self, awards: &[AwardPb]) -> Result<Vec<GameEvent>> {
        let mut events = Vec::new();
        for award in actor_owned_awards(awards.to_vec(), "award grant") {
            if award.count <= 0 {
                continue;
            }
            match (award.r#type, award.id) {
                (AWARD_TYPE_DIAMOND, AWARD_ID_DIAMOND) => {
                    let lord = self.lord_mut()?;
                    lord.diamond = Some(lord.diamond.unwrap_or(0) + award.count);
                    self.lord_dirty = true;
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_MEAT) => {
                    let lord = self.lord_mut()?;
                    lord.meat = Some(lord.meat.unwrap_or(0) + award.count);
                    self.lord_dirty = true;
                    events.push(GameEvent::ResourceChange {
                        role_id: self.role_id,
                        resource_type: RESOURCE_ID_MEAT,
                        delta: award.count,
                    });
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_GOLD) => {
                    let lord = self.lord_mut()?;
                    lord.gold = Some(lord.gold.unwrap_or(0) + award.count);
                    self.lord_dirty = true;
                    events.push(GameEvent::ResourceChange {
                        role_id: self.role_id,
                        resource_type: RESOURCE_ID_GOLD,
                        delta: award.count,
                    });
                }
                (AWARD_TYPE_RESOURCE, RESOURCE_ID_FAME) => {
                    let lord = self.lord_mut()?;
                    let have = i64::from(lord.fame.unwrap_or(0));
                    lord.fame = Some(i32::try_from(have + award.count)?);
                    self.lord_dirty = true;
                    events.push(GameEvent::ResourceChange {
                        role_id: self.role_id,
                        resource_type: RESOURCE_ID_FAME,
                        delta: award.count,
                    });
                }
                (AWARD_TYPE_ITEM, _) => {
                    events.extend(self.backpack_system.add_item_with_event(
                        self.role_id,
                        award.id,
                        award.count,
                    ));
                }
                _ => {}
            }
        }
        Ok(events)
    }

    fn lord_mut(&mut self) -> Result<&mut LordRow> {
        self.lord
            .as_mut()
            .ok_or_else(|| anyhow!("p_lord data is not loaded"))
    }
}

#[derive(Default)]
struct CommandEconomy {
    costs: Vec<AwardPb>,
}

fn bind_event_role_id(event: &mut GameEvent, role_id: i64) {
    fn fill(value: &mut i64, role_id: i64) {
        if *value == 0 {
            *value = role_id;
        }
    }

    match event {
        GameEvent::PlayerLogin { role_id: event_role_id }
        | GameEvent::PlayerLogout { role_id: event_role_id }
        | GameEvent::HeroLevelUp { role_id: event_role_id, .. }
        | GameEvent::HeroTierUp { role_id: event_role_id, .. }
        | GameEvent::BuildingUpgrade { role_id: event_role_id, .. }
        | GameEvent::TechResearchComplete { role_id: event_role_id, .. }
        | GameEvent::EquipStrengthen { role_id: event_role_id, .. }
        | GameEvent::ItemConsume { role_id: event_role_id, .. }
        | GameEvent::ItemGain { role_id: event_role_id, .. }
        | GameEvent::DiamondConsume { role_id: event_role_id, .. }
        | GameEvent::GoldConsume { role_id: event_role_id, .. }
        | GameEvent::ResourceChange { role_id: event_role_id, .. }
        | GameEvent::BattleEnd { role_id: event_role_id, .. }
        | GameEvent::TroopTrain { role_id: event_role_id, .. }
        | GameEvent::TroopHeal { role_id: event_role_id, .. } => fill(event_role_id, role_id),
        GameEvent::Mission(mission) => fill(&mut mission.role_id, role_id),
        GameEvent::ActivityTrigger(activity) => fill(&mut activity.role_id, role_id),
    }
}

fn tech_level_at_stage(system: &TechSystem, tech_id: i32, stage: i32) -> i32 {
    system
        .nodes
        .iter()
        .find(|node| node.technology_id == Some(tech_id) && node.stage == Some(stage))
        .and_then(|node| node.level)
        .unwrap_or(0)
}

fn actor_owned_awards(awards: Vec<AwardPb>, context: &'static str) -> Vec<AwardPb> {
    awards
        .into_iter()
        .filter(|award| {
            if award.r#type == AWARD_TYPE_ITEM && award.id == VIRTUAL_DAILY_LIVENESS_PROP_ID {
                return false;
            }
            let supported = matches!(
                (award.r#type, award.id),
                (AWARD_TYPE_DIAMOND, AWARD_ID_DIAMOND)
                    | (AWARD_TYPE_RESOURCE, RESOURCE_ID_MEAT)
                    | (AWARD_TYPE_RESOURCE, RESOURCE_ID_GOLD)
                    | (AWARD_TYPE_RESOURCE, RESOURCE_ID_FAME)
            ) || award.r#type == AWARD_TYPE_ITEM;
            if !supported && award.count > 0 {
                warn!(
                    context = context,
                    award_type = award.r#type,
                    award_id = award.id,
                    count = award.count,
                    "Skipping award outside PlayerActor economy"
                );
            }
            supported
        })
        .collect()
}

fn ensure_enough(name: &str, have: i64, need: i64) -> Result<()> {
    if have < need {
        return Err(anyhow!("Insufficient {}: have {}, need {}", name, have, need));
    }
    Ok(())
}

fn parse_i32(raw: &str) -> Option<i32> {
    raw.trim().parse::<i32>().ok()
}

fn parse_award_list(award_str: &str) -> Vec<AwardPb> {
    let raw = award_str.trim();
    if raw.is_empty() || raw == "0" || raw.eq_ignore_ascii_case("null") {
        return Vec::new();
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        let mut awards = Vec::new();
        collect_awards_from_json(&value, &mut awards);
        if !awards.is_empty() {
            return awards;
        }
    }

    raw.split(';')
        .filter_map(|segment| {
            let parts: Vec<&str> = segment.split(',').collect();
            if parts.len() < 3 {
                return None;
            }
            Some(AwardPb {
                r#type: parts[0].trim().parse().ok()?,
                id: parts[1].trim().parse().ok()?,
                count: parts[2].trim().parse().ok()?,
                ..Default::default()
            })
        })
        .collect()
}

fn collect_awards_from_json(value: &serde_json::Value, awards: &mut Vec<AwardPb>) {
    let serde_json::Value::Array(items) = value else {
        return;
    };

    if items.len() >= 3 && items.iter().take(3).all(serde_json::Value::is_number) {
        let Some(award_type) = json_i32(&items[0]) else {
            return;
        };
        let Some(id) = json_i32(&items[1]) else {
            return;
        };
        let Some(count) = json_i64(&items[2]) else {
            return;
        };
        awards.push(AwardPb {
            r#type: award_type,
            id,
            count,
            ..Default::default()
        });
        return;
    }

    for item in items {
        collect_awards_from_json(item, awards);
    }
}

fn json_i32(value: &serde_json::Value) -> Option<i32> {
    value.as_i64().and_then(|v| i32::try_from(v).ok())
}

fn json_i64(value: &serde_json::Value) -> Option<i64> {
    value.as_i64()
}

fn multiply_awards(awards: &mut Vec<AwardPb>, multiplier: i64) {
    if multiplier <= 0 {
        awards.clear();
        return;
    }
    for award in awards {
        award.count = award.count.saturating_mul(multiplier);
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
    use proto::slg::{
        PropUseRq, PropUseRs, TechnologyResearchQueue, TechnologyResearchRq, TechnologySpeedUpRq,
    };
    use shared::event::{ActivityTriggerEvent, MissionEvent, MissionType};
    use shared::msg::func_type;
    use shared::static_config::item::StaticPropConf;
    use shared::static_config::tech::StaticTechLv;
    use sqlx::mysql::MySqlPoolOptions;

    fn test_actor(config: StaticConfig) -> PlayerActor {
        let (_msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (_config_tx, config_rx) = watch::channel(Arc::new(config));
        let (activity_tx, _activity_rx) = mpsc::channel(1);
        let dao = Arc::new(PlayerDao::new(
            MySqlPoolOptions::new()
                .connect_lazy("mysql://root:pass@localhost/test")
                .expect("lazy mysql pool"),
        ));

        PlayerActor::new(
            7,
            42,
            msg_rx,
            GlobalEventBus::new(activity_tx),
            config_rx,
            dao,
        )
    }

    fn lord_row() -> LordRow {
        LordRow {
            role_id: 42,
            nick: Some("tester".to_string()),
            portrait: Some("7".to_string()),
            portrait_frame: Some(8),
            top_up: None,
            diamond: Some(100),
            diamond_cost: Some(0),
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
            meat: Some(50),
            fame: Some(12),
            gold: Some(30),
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
        }
    }

    fn prop_conf(prop_id: i32, reward_list: &str) -> StaticPropConf {
        StaticPropConf {
            prop_id,
            description: "prop".to_string(),
            desc2: String::new(),
            asset: None,
            asset_base: None,
            badge: None,
            get_way: None,
            prop_type: 1,
            quality: 1,
            order: 1,
            reward_list: Some(reward_list.to_string()),
            back_type: None,
            can_sell: None,
            duration: None,
            attrs: None,
            jump: None,
            num_display: None,
            dis_position: None,
            can_use: Some(1),
            cli_button: None,
            batch_use: None,
            shop_prop_id: None,
            function_open: None,
            access: None,
            effect: None,
            show_type: None,
            buff_effect_id: None,
            effect_tips_type: 0,
        }
    }

    fn award(award_type: i32, id: i32, count: i64) -> AwardPb {
        AwardPb {
            r#type: award_type,
            id,
            count,
            ..Default::default()
        }
    }

    fn tech_row(up_need_resource: &str) -> StaticTechLv {
        StaticTechLv {
            id: 100101,
            tech_id: 10,
            tech_type: 1,
            tech_name: "tech".to_string(),
            tech_stage: "1".to_string(),
            level: 1,
            max_lv: "3".to_string(),
            cnt: 1,
            up_time: 60,
            up_need_resource: Some(up_need_resource.to_string()),
            reputation_require: Some("0".to_string()),
            need_tech: None,
            need_building: None,
            next_id: None,
            buff_effect_id: None,
            fight: None,
            icon: None,
            cond: None,
            description: None,
        }
    }

    fn event_role_id(event: &GameEvent) -> i64 {
        match event {
            GameEvent::PlayerLogin { role_id }
            | GameEvent::PlayerLogout { role_id }
            | GameEvent::HeroLevelUp { role_id, .. }
            | GameEvent::HeroTierUp { role_id, .. }
            | GameEvent::BuildingUpgrade { role_id, .. }
            | GameEvent::TechResearchComplete { role_id, .. }
            | GameEvent::EquipStrengthen { role_id, .. }
            | GameEvent::ItemConsume { role_id, .. }
            | GameEvent::ItemGain { role_id, .. }
            | GameEvent::DiamondConsume { role_id, .. }
            | GameEvent::GoldConsume { role_id, .. }
            | GameEvent::ResourceChange { role_id, .. }
            | GameEvent::BattleEnd { role_id, .. }
            | GameEvent::TroopTrain { role_id, .. }
            | GameEvent::TroopHeal { role_id, .. } => *role_id,
            GameEvent::Mission(mission) => mission.role_id,
            GameEvent::ActivityTrigger(activity) => activity.role_id,
        }
    }

    #[test]
    fn bind_event_role_id_fills_zero_placeholders_only() {
        let mut event = GameEvent::HeroLevelUp {
            role_id: 0,
            hero_id: 1001,
            new_level: 2,
        };
        bind_event_role_id(&mut event, 42);
        assert_eq!(event_role_id(&event), 42);

        let mut mission =
            GameEvent::Mission(MissionEvent::single(0, MissionType::TechResearch, 1001));
        bind_event_role_id(&mut mission, 42);
        assert_eq!(event_role_id(&mission), 42);

        let mut activity = GameEvent::ActivityTrigger(ActivityTriggerEvent {
            role_id: 7,
            trigger_type: 1,
            params: vec![],
        });
        bind_event_role_id(&mut activity, 42);
        assert_eq!(event_role_id(&activity), 7);
    }

    #[tokio::test]
    async fn actor_economy_consumes_and_grants_owned_resources() {
        let mut actor = test_actor(StaticConfig::default());
        actor.lord = Some(lord_row());
        actor.backpack_system.add_item(900, 5);
        actor.backpack_system.clear_dirty();

        let events = actor
            .consume_awards(&[
                award(AWARD_TYPE_DIAMOND, AWARD_ID_DIAMOND, 10),
                award(AWARD_TYPE_RESOURCE, RESOURCE_ID_MEAT, 5),
                award(AWARD_TYPE_RESOURCE, RESOURCE_ID_GOLD, 6),
                award(AWARD_TYPE_RESOURCE, RESOURCE_ID_FAME, 4),
                award(AWARD_TYPE_ITEM, 900, 2),
            ])
            .expect("consume awards");

        let lord = actor.lord.as_ref().unwrap();
        assert_eq!(lord.diamond, Some(90));
        assert_eq!(lord.diamond_cost, Some(10));
        assert_eq!(lord.meat, Some(45));
        assert_eq!(lord.gold, Some(24));
        assert_eq!(lord.fame, Some(8));
        assert_eq!(actor.backpack_system.get_item_count(900), 3);
        assert!(events.iter().all(|event| event_role_id(event) == 42));

        actor
            .grant_awards(&[
                award(AWARD_TYPE_DIAMOND, AWARD_ID_DIAMOND, 7),
                award(AWARD_TYPE_RESOURCE, RESOURCE_ID_MEAT, 8),
                award(AWARD_TYPE_RESOURCE, RESOURCE_ID_GOLD, 9),
                award(AWARD_TYPE_RESOURCE, RESOURCE_ID_FAME, 3),
                award(AWARD_TYPE_ITEM, 901, 4),
                award(AWARD_TYPE_ITEM, VIRTUAL_DAILY_LIVENESS_PROP_ID, 99),
                award(AWARD_TYPE_RESOURCE, 2, 10),
            ])
            .expect("grant awards");

        let lord = actor.lord.as_ref().unwrap();
        assert_eq!(lord.diamond, Some(97));
        assert_eq!(lord.meat, Some(53));
        assert_eq!(lord.gold, Some(33));
        assert_eq!(lord.fame, Some(11));
        assert_eq!(actor.backpack_system.get_item_count(901), 4);
        assert_eq!(
            actor
                .backpack_system
                .get_item_count(VIRTUAL_DAILY_LIVENESS_PROP_ID),
            0
        );
    }

    #[tokio::test]
    async fn prop_use_grants_configured_rewards_in_player_actor() {
        let mut config = StaticConfig::default();
        config.item.props.insert(
            501,
            prop_conf(501, "[[1,3,100],[4,901,2],[4,4002,10]]"),
        );
        let mut actor = test_actor(config);
        actor.lord = Some(lord_row());
        actor.lord.as_mut().unwrap().diamond = Some(0);
        actor.backpack_system.add_item(501, 2);
        actor.backpack_system.clear_dirty();

        let resp = actor
            .handle_game_command(
                4003,
                PropUseRq {
                    prop_id: 501,
                    use_count: 2,
                    show: Some(0),
                    ext_param: vec![],
                }
                .encode_to_vec(),
            )
            .await
            .expect("prop use");

        let msg = shared::msg::GameMessage::decode(resp).expect("decode response");
        assert_eq!(msg.base.cmd, 4004);
        let rs: PropUseRs = msg.get_payload().expect("decode prop use response");
        let show_award = rs.change_info.unwrap().show_award;
        assert_eq!(show_award.iter().find(|a| a.id == 3).unwrap().count, 200);
        assert_eq!(show_award.iter().find(|a| a.id == 901).unwrap().count, 4);
        assert_eq!(show_award.iter().find(|a| a.id == 4002).unwrap().count, 20);

        assert_eq!(actor.lord.as_ref().unwrap().diamond, Some(200));
        assert_eq!(actor.backpack_system.get_item_count(501), 0);
        assert_eq!(actor.backpack_system.get_item_count(901), 4);
        assert_eq!(
            actor
                .backpack_system
                .get_item_count(VIRTUAL_DAILY_LIVENESS_PROP_ID),
            0
        );
    }

    #[tokio::test]
    async fn tech_item_speedup_consumes_item_after_successful_command() {
        let mut actor = test_actor(StaticConfig::default());
        actor.lord = Some(lord_row());
        actor.backpack_system.add_item(700, 3);
        actor.tech_system.queue.push(TechnologyResearchQueue {
            technology_id: Some(10),
            research_level: Some(1),
            complete_time: Some(1_000),
            research_stage: Some(1),
            ..Default::default()
        });

        actor
            .handle_game_command(
                4203,
                TechnologySpeedUpRq {
                    technology_id: 10,
                    stage: 1,
                    r#type: 1,
                    prop_id: Some(700),
                    use_number: Some(2),
                }
                .encode_to_vec(),
            )
            .await
            .expect("speed up");

        assert_eq!(actor.backpack_system.get_item_count(700), 1);
        assert!(
            actor
                .tech_system
                .queue
                .first()
                .unwrap()
                .complete_time
                .unwrap()
            < 1_000
        );
    }

    #[tokio::test]
    async fn tech_research_consumes_actor_owned_config_costs() {
        let mut config = StaticConfig::default();
        config
            .tech
            .tech_levels
            .insert(100101, tech_row("[[2,11,5],[2,3,7],[2,2,9]]"));
        let mut actor = test_actor(config);
        actor.lord = Some(lord_row());

        actor
            .handle_game_command(
                4201,
                TechnologyResearchRq {
                    technology_id: 10,
                    stage: 1,
                    r#type: 1,
                }
                .encode_to_vec(),
            )
            .await
            .expect("research");

        let lord = actor.lord.as_ref().unwrap();
        assert_eq!(lord.fame, Some(7));
        assert_eq!(lord.meat, Some(43));
        assert_eq!(actor.tech_system.queue.len(), 1);
    }

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
