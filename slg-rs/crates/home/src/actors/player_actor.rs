use tokio::sync::{mpsc, oneshot};
use tracing::{info, debug};
use proto::slg::{RoleLoginRs, GetRoleDataRs};

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
}

use crate::systems::activity::ActivitySystem;
use crate::systems::ToFunctionClientBase;

pub struct PlayerActor {
    pub account_id: i64,
    pub role_id: i64,
    pub state: PlayerState,
    
    msg_rx: mpsc::UnboundedReceiver<PlayerMessage>,
    
    // システムモジュール
    pub activity_system: ActivitySystem,
}

impl PlayerActor {
    pub fn new(account_id: i64, role_id: i64, rx: mpsc::UnboundedReceiver<PlayerMessage>) -> Self {
        Self {
            account_id,
            role_id,
            state: PlayerState::Loading,
            msg_rx: rx,
            activity_system: ActivitySystem::new(),
        }
    }

    pub async fn run(mut self) {
        info!("Actor for role {} started", self.role_id);
        
        while let Some(msg) = self.msg_rx.recv().await {
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
                    break;
                }
            }
        }
    }

    async fn handle_role_login(&mut self, tx: oneshot::Sender<anyhow::Result<RoleLoginRs>>) {
        info!("Role {} logged in", self.role_id);
        self.state = PlayerState::InGame;
        let _ = tx.send(Ok(RoleLoginRs { state: Some(1), ..Default::default() }));
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

