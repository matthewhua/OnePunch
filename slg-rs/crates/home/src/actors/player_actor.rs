use tokio::sync::{mpsc, oneshot, watch};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};
use proto::slg::{RoleLoginRs, GetRoleDataRs};
use shared::static_config::StaticConfig;
use shared::persistence::{PlayerDao, SaveEntry};

use crate::systems::PlayerSystem;
use crate::systems::activity::ActivitySystem;
use crate::systems::building::BuildingSystem;
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
    /// 游戏内行为事件（击杀、建造等）
    DispatchGameEvent(GameEvent),
    /// 立即存盘（充值等关键操作后触发）
    ForceSave,
}

pub struct PlayerActor {
    pub account_id: i64,
    pub role_id: i64,
    pub state: PlayerState,

    msg_rx: mpsc::UnboundedReceiver<PlayerMessage>,

    // 功能系统模块
    pub activity_system: ActivitySystem,
    pub building_system: BuildingSystem,
    pub skin_system: SkinSystem,

    // 事件总线
    pub event_dispatcher: EventDispatcher,
    pub ctx: PlayerContext,
    pub global_event_bus: GlobalEventBus,

    /// 静态配置订阅
    pub config_rx: watch::Receiver<Arc<StaticConfig>>,
    /// 当前生效的静态配置快照
    pub current_config: Arc<StaticConfig>,

    /// 数据库访问
    pub dao: Arc<PlayerDao>,
    /// 连续存盘失败次数
    save_fail_count: u32,
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
            activity_system: ActivitySystem::new(),
            building_system: BuildingSystem::new(),
            skin_system: SkinSystem::new(),
            event_dispatcher: EventDispatcher::new(),
            ctx: PlayerContext { role_id, account_id },
            global_event_bus,
            config_rx,
            current_config,
            dao,
            save_fail_count: 0,
        }
    }

    /// 从数据库加载玩家数据到各功能系统
    pub async fn load_data(&mut self) -> anyhow::Result<()> {
        let rows = self.dao.load_all_data(self.role_id).await?;

        // 按 keyId 分发到对应的系统
        let data_map: HashMap<i32, Vec<u8>> = rows
            .into_iter()
            .filter_map(|r| r.data.map(|d| (r.key_id, d)))
            .collect();

        // 加载各系统
        self.load_system_mut(&data_map);

        // 更新登录时间
        self.dao.update_login_time(self.role_id).await?;

        info!(role_id = self.role_id, modules = data_map.len(), "Player data loaded");
        Ok(())
    }

    /// 将数据分发到各系统模块
    fn load_system_mut(&mut self, data_map: &HashMap<i32, Vec<u8>>) {
        let systems: Vec<(&mut dyn PlayerSystem, &'static str)> = vec![
            (&mut self.activity_system, "activity"),
            (&mut self.building_system, "building"),
            (&mut self.skin_system, "skin"),
        ];

        for (system, name) in systems {
            let kid = system.key_id();
            if let Some(data) = data_map.get(&kid) {
                if !data.is_empty() {
                    if let Err(e) = system.load_from_bin(data) {
                        warn!(
                            role_id = self.role_id, system = name, key_id = kid,
                            "Failed to deserialize system data, using default: {}", e
                        );
                    }
                }
            }
        }
    }

    pub async fn run(mut self) {
        info!(role_id = self.role_id, "PlayerActor started");

        // 加载数据
        if let Err(e) = self.load_data().await {
            error!(role_id = self.role_id, "Failed to load player data: {}", e);
            return;
        }
        self.state = PlayerState::Loading;

        // 触发各系统 on_login
        self.activity_system.on_login();
        self.building_system.on_login();
        self.skin_system.on_login();

        // 定时存盘 + 每秒 tick
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
                            info!(role_id = self.role_id, "Shutting down, saving data...");
                            self.do_save(true).await;
                            self.dao.update_logout_time(self.role_id).await.ok();
                            info!(role_id = self.role_id, "PlayerActor stopped");
                            return;
                        }
                        PlayerMessage::DispatchGameEvent(event) => {
                            self.dispatch_event(&event);
                        }
                        PlayerMessage::ForceSave => {
                            self.do_save(false).await;
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
                    let new_config = self.config_rx.borrow().clone();
                    self.current_config = new_config;
                }
            }
        }
    }

    /// 每秒 tick
    fn on_tick(&mut self) {
        self.activity_system.tick();
        self.building_system.tick();
        self.skin_system.tick();
    }

    /// 执行存盘
    ///
    /// `force_all` = true 时全量存盘（下线），否则仅存 dirty 模块。
    async fn do_save(&mut self, force_all: bool) {
        let mut entries = Vec::new();

        // 收集需要存盘的系统
        let systems: Vec<&mut dyn PlayerSystem> = vec![
            &mut self.activity_system,
            &mut self.building_system,
            &mut self.skin_system,
        ];

        for system in systems {
            if force_all || system.is_dirty() {
                match system.save_to_bin() {
                    Ok(data) => {
                        entries.push(SaveEntry {
                            key_id: system.key_id(),
                            data,
                        });
                        system.clear_dirty();
                    }
                    Err(e) => {
                        error!(
                            role_id = self.role_id,
                            system = system.column_name(),
                            "Failed to serialize system: {}", e
                        );
                    }
                }
            }
        }

        if entries.is_empty() {
            return;
        }

        let entry_count = entries.len();

        // 带超时的异步存盘
        let save_result = tokio::time::timeout(
            Duration::from_secs(SAVE_TIMEOUT_SECS),
            self.dao.save_data(self.role_id, &entries),
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
                error!(
                    role_id = self.role_id, fail_count = self.save_fail_count,
                    "Save failed: {}", e
                );
                if self.save_fail_count >= 3 {
                    warn!(role_id = self.role_id, "3 consecutive save failures, emergency save to file");
                    shared::persistence::emergency_save_to_file(
                        EMERGENCY_SAVE_DIR, self.role_id, &entries,
                    );
                }
            }
            Err(_) => {
                self.save_fail_count += 1;
                error!(
                    role_id = self.role_id, fail_count = self.save_fail_count,
                    "Save timed out ({}s)", SAVE_TIMEOUT_SECS
                );
                if self.save_fail_count >= 3 {
                    shared::persistence::emergency_save_to_file(
                        EMERGENCY_SAVE_DIR, self.role_id, &entries,
                    );
                }
            }
        }
    }

    async fn handle_role_login(&mut self, tx: oneshot::Sender<anyhow::Result<RoleLoginRs>>) {
        info!(role_id = self.role_id, "Role logged in");
        self.state = PlayerState::InGame;

        let event = GameEvent::PlayerLogin { role_id: self.role_id };
        self.dispatch_event(&event);

        let _ = tx.send(Ok(RoleLoginRs { state: Some(1), ..Default::default() }));
    }

    pub fn dispatch_event(&mut self, event: &GameEvent) {
        self.activity_system.handle_event(event, &mut self.ctx);
        self.event_dispatcher.dispatch(event, &mut self.ctx);
    }

    async fn handle_get_role_data(&mut self, tx: oneshot::Sender<anyhow::Result<GetRoleDataRs>>) {
        use proto::slg::FunctionClientBase;
        use prost::Message;
        use shared::msg::ToFunctionClientBaseBytes;

        let mut function_base = Vec::new();

        // Activity
        let act_bytes = self.activity_system.to_function_base_bytes();
        if let Ok(f_base) = FunctionClientBase::decode(act_bytes.as_slice()) {
            function_base.push(f_base);
        }

        // TODO: 其他系统的 FunctionClientBase 数据

        let rs = GetRoleDataRs {
            function_base,
            ..Default::default()
        };
        let _ = tx.send(Ok(rs));
    }
}
