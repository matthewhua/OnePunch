use common::slg::{BeginGameRq, BeginGameRs, CreateRoleRq, CreateRoleRs};
use tokio::sync::{mpsc, oneshot};

/// 玩家状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Authenticating,    // 鉴权中 (BeginGame)
    RoleCreating,      // 创角中 (CreateRole)
    Loading,           // 正在从 DB 加载全量数据
    InGame,            // 游戏中
    OfflineRetained,   // 离线驻留 (SLG特有)
}

/// 发送给 PlayerActor 的消息
pub enum PlayerMessage {
    /// 鉴权请求 (account_id -> role_id check)
    BeginGame(BeginGameRq, oneshot::Sender<anyhow::Result<BeginGameRs>>),
    /// 创角请求
    CreateRole(CreateRoleRq, oneshot::Sender<anyhow::Result<CreateRoleRs>>),
}

/// 玩家 Actor 实体
pub struct PlayerActor {
    pub account_id: i64,
    pub role_id: i64,
    pub state: PlayerState,
    
    msg_rx: mpsc::UnboundedReceiver<PlayerMessage>,
}

impl PlayerActor {
    pub fn new(account_id: i64, rx: mpsc::UnboundedReceiver<PlayerMessage>) -> Self {
        Self {
            account_id,
            role_id: 0,
            state: PlayerState::Authenticating,
            msg_rx: rx,
        }
    }

    /// 执行 Actor 主循环
    pub async fn run(mut self) {
        tracing::info!("PlayerActor [Account:{}] started", self.account_id);
        
        while let Some(msg) = self.msg_rx.recv().await {
            match msg {
                PlayerMessage::BeginGame(req, tx) => {
                    self.handle_begin_game(req, tx).await;
                }
                PlayerMessage::CreateRole(req, tx) => {
                    self.handle_create_role(req, tx).await;
                }
            }
        }
        
        tracing::info!("PlayerActor [Account:{}] stopped", self.account_id);
    }

    async fn handle_begin_game(&mut self, req: BeginGameRq, tx: oneshot::Sender<anyhow::Result<BeginGameRs>>) {
        tracing::debug!("Handling BeginGame for account {}", self.account_id);
        // TODO: 具体的业务校验逻辑
        let _ = tx.send(Ok(BeginGameRs::default()));
    }

    async fn handle_create_role(&mut self, req: CreateRoleRq, tx: oneshot::Sender<anyhow::Result<CreateRoleRs>>) {
        tracing::debug!("Handling CreateRole for account {}", self.account_id);
        // TODO: 具体的业务校验逻辑
        let _ = tx.send(Ok(CreateRoleRs::default()));
    }
}
