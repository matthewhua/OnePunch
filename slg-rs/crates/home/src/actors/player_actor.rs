use tokio::sync::{mpsc, oneshot, watch};
use std::sync::Arc;
use tracing::{info, debug};
use proto::slg::{RoleLoginRs, GetRoleDataRs};
use shared::static_config::StaticConfig;

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
}

use crate::systems::activity::ActivitySystem;
use crate::systems::ToFunctionClientBase;
use shared::event::{EventDispatcher, PlayerContext, GameEvent, GlobalEvent};
use crate::actors::global_event_bus::GlobalEventBus;

pub struct PlayerActor {
    pub account_id: i64,
    pub role_id: i64,
    pub state: PlayerState,
    
    msg_rx: mpsc::UnboundedReceiver<PlayerMessage>,
    
    // システムモジュール
    pub activity_system: ActivitySystem,
    
    // 事件总线
    pub event_dispatcher: EventDispatcher,
    pub ctx: PlayerContext,
    pub global_event_bus: GlobalEventBus,
    /// 静态配置订阅
    pub config_rx: watch::Receiver<Arc<StaticConfig>>,
    /// 当前生效的静态配置快照
    pub current_config: Arc<StaticConfig>,
}

impl PlayerActor {
    pub fn new(
        account_id: i64, 
        role_id: i64, 
        rx: mpsc::UnboundedReceiver<PlayerMessage>,
        global_event_bus: GlobalEventBus,
        config_rx: watch::Receiver<Arc<StaticConfig>>,
    ) -> Self {
        let current_config = config_rx.borrow().clone();
        Self {
            account_id,
            role_id,
            state: PlayerState::Loading,
            msg_rx: rx,
            activity_system: ActivitySystem::new(),
            event_dispatcher: EventDispatcher::new(),
            ctx: PlayerContext { role_id, account_id },
            global_event_bus,
            config_rx,
            current_config,
        }
    }

    pub async fn run(mut self) {
        info!("Actor for role {} started", self.role_id);
        
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
                        PlayerMessage::Heartbeat => {
                            debug!("Heartbeat for role {}", self.role_id);
                        }
                        PlayerMessage::Shutdown => {
                            info!("Actor for role {} shutting down", self.role_id);
                            return;
                        }
                        PlayerMessage::DispatchGameEvent(event) => {
                            self.dispatch_event(&event);
                        }
                    }
                }
                Ok(()) = self.config_rx.changed() => {
                    info!("PlayerActor {} received static config reload", self.role_id);
                    let new_config = self.config_rx.borrow().clone();
                    self.current_config = new_config;
                }
            }
        }
    }

    async fn handle_role_login(&mut self, tx: oneshot::Sender<anyhow::Result<RoleLoginRs>>) {
        info!("Role {} logged in", self.role_id);
        self.state = PlayerState::InGame;
        
        // 触发登录事件
        let event = GameEvent::PlayerLogin { role_id: self.role_id };
        self.dispatch_event(&event);

        let _ = tx.send(Ok(RoleLoginRs { state: Some(1), ..Default::default() }));
    }

    pub fn dispatch_event(&mut self, event: &GameEvent) {
        // 1. 同步分发给各模块系统 (推荐直接调用以避免所有权冲突)
        self.activity_system.handle_event(event, &mut self.ctx);
        // self.mission_system.handle_event(event, &mut self.ctx);

        // 2. 分发给动态注册的处理器
        self.event_dispatcher.dispatch(event, &mut self.ctx);
    }

    async fn handle_get_role_data(&mut self, tx: oneshot::Sender<anyhow::Result<GetRoleDataRs>>) {
        use proto::slg::FunctionClientBase;
        use prost::Message;

        let mut function_base = Vec::new();

        // 1. 获取 Activity 数据
        match self.activity_system.to_function_base_bytes() {
            Ok(bytes) => {
                if let Ok(f_base) = FunctionClientBase::decode(&bytes[..]) {
                    function_base.push(f_base);
                }
            }
            Err(e) => tracing::error!("Failed to get activity function base: {}", e),
        }

        // TODO: 获取 Lord, Hero 等其他系统数据

        let rs = GetRoleDataRs {
            function_base,
            ..Default::default()
        };
        let _ = tx.send(Ok(rs));
    }
}

