//! Gateway 会话管理
//!
//! 维护每个 TCP 连接的会话状态，对应 Java 版的 `ClientGatewayProxy`。
//!
//! # 会话状态机
//!
//! ```text
//! Connected ──DoLoginRq──▶ Authenticated ──VerifyRq──▶ Verified ──BeginGameRq──▶ InGame
//!     │                         │                          │                        │
//!     └──timeout/error──────────┴──timeout/error───────────┴──disconnect────────────┘
//!                                                                                    ▼
//!                                                                               Disconnecting
//! ```

use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// 会话状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// 已建立 TCP 连接，等待 DoLoginRq
    Connected,
    /// DoLogin 成功，持有 account_key_id 和 token，等待 VerifyRq
    Authenticated,
    /// Verify 成功，持有 server_id，等待 BeginGameRq
    Verified,
    /// 已进入游戏，可以转发业务消息
    InGame,
    /// 正在断开
    Disconnecting,
}

/// 会话数据
#[derive(Debug)]
pub struct Session {
    pub conn_id: u64,
    pub peer_addr: SocketAddr,
    pub state: SessionState,
    /// DoLogin 成功后获得的账号唯一ID
    pub account_key_id: Option<i64>,
    /// DoLogin 成功后获得的 session token
    pub token: Option<String>,
    /// Verify 成功后确定的区服号
    pub server_id: Option<i32>,
    /// 角色ID（BeginGame 后获得）
    pub role_id: Option<i64>,
    /// 最后活跃时间（用于心跳超时检测）
    pub last_active: Instant,
    /// 连接建立时间
    pub connected_at: Instant,
}

impl Session {
    pub fn new(conn_id: u64, peer_addr: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            conn_id,
            peer_addr,
            state: SessionState::Connected,
            account_key_id: None,
            token: None,
            server_id: None,
            role_id: None,
            last_active: now,
            connected_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_active = Instant::now();
    }

    pub fn is_idle(&self, timeout: Duration) -> bool {
        self.last_active.elapsed() > timeout
    }
}

/// 全局会话存储
///
/// 通过 `Arc<SessionStore>` 在 main 和各 ConnectionHandler 间共享。
pub struct SessionStore {
    /// conn_id → Session
    sessions: DashMap<u64, Session>,
    /// account_key_id → conn_id（用于踢重复登录）
    account_index: DashMap<i64, u64>,
    /// 自增连接ID
    next_conn_id: AtomicU64,
}

impl SessionStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            sessions: DashMap::new(),
            account_index: DashMap::new(),
            next_conn_id: AtomicU64::new(1),
        })
    }

    /// 分配新的连接ID
    pub fn alloc_conn_id(&self) -> u64 {
        self.next_conn_id.fetch_add(1, Ordering::Relaxed)
    }

    /// 注册新连接
    pub fn register(&self, conn_id: u64, peer_addr: SocketAddr) {
        self.sessions.insert(conn_id, Session::new(conn_id, peer_addr));
    }

    /// 移除连接
    pub fn remove(&self, conn_id: u64) {
        if let Some((_, session)) = self.sessions.remove(&conn_id) {
            if let Some(account_key_id) = session.account_key_id {
                // 只有当 account_index 指向本连接时才移除（防止踢人后误删新连接）
                self.account_index.remove_if(&account_key_id, |_, &v| v == conn_id);
            }
        }
    }

    /// 绑定账号到连接（DoLogin 成功后调用）
    ///
    /// 如果该账号已有旧连接，返回旧连接ID（调用方负责踢掉旧连接）。
    pub fn bind_account(&self, conn_id: u64, account_key_id: i64, token: String) -> Option<u64> {
        let old_conn_id = self.account_index.insert(account_key_id, conn_id);

        if let Some(mut session) = self.sessions.get_mut(&conn_id) {
            session.account_key_id = Some(account_key_id);
            session.token = Some(token);
            session.state = SessionState::Authenticated;
        }

        old_conn_id
    }

    /// 设置区服（Verify 成功后调用）
    pub fn set_verified(&self, conn_id: u64, server_id: i32) {
        if let Some(mut session) = self.sessions.get_mut(&conn_id) {
            session.server_id = Some(server_id);
            session.state = SessionState::Verified;
        }
    }

    /// 设置角色（BeginGame 后调用）
    pub fn set_in_game(&self, conn_id: u64, role_id: i64) {
        if let Some(mut session) = self.sessions.get_mut(&conn_id) {
            session.role_id = Some(role_id);
            session.state = SessionState::InGame;
        }
    }

    /// 更新最后活跃时间
    pub fn touch(&self, conn_id: u64) {
        if let Some(mut session) = self.sessions.get_mut(&conn_id) {
            session.touch();
        }
    }

    /// 获取会话快照（只读）
    pub fn get_state(&self, conn_id: u64) -> Option<SessionState> {
        self.sessions.get(&conn_id).map(|s| s.state.clone())
    }

    /// 获取 account_key_id
    pub fn get_account_key_id(&self, conn_id: u64) -> Option<i64> {
        self.sessions.get(&conn_id).and_then(|s| s.account_key_id)
    }

    /// 获取 role_id
    pub fn get_role_id(&self, conn_id: u64) -> Option<i64> {
        self.sessions.get(&conn_id).and_then(|s| s.role_id)
    }

    /// 获取 server_id
    pub fn get_server_id(&self, conn_id: u64) -> Option<i32> {
        self.sessions.get(&conn_id).and_then(|s| s.server_id)
    }

    /// 在线连接数
    pub fn online_count(&self) -> usize {
        self.sessions.len()
    }

    /// 检查空闲超时的连接（返回超时的 conn_id 列表）
    pub fn find_idle(&self, timeout: Duration) -> Vec<u64> {
        self.sessions
            .iter()
            .filter(|e| e.is_idle(timeout))
            .map(|e| e.conn_id)
            .collect()
    }
}

/// 连接断开通知（用于通知 Home Service 玩家下线）
#[derive(Debug)]
pub struct DisconnectNotice {
    pub conn_id: u64,
    pub account_key_id: i64,
    pub role_id: Option<i64>,
}

/// 断开通知发送端（由 main 创建，传给各 ConnectionHandler）
pub type DisconnectTx = mpsc::UnboundedSender<DisconnectNotice>;
/// 断开通知接收端（由 main 持有，驱动下线流程）
pub type DisconnectRx = mpsc::UnboundedReceiver<DisconnectNotice>;

pub fn disconnect_channel() -> (DisconnectTx, DisconnectRx) {
    mpsc::unbounded_channel()
}
