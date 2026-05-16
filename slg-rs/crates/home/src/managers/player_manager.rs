use crate::actors::global_event_bus::GlobalEventBus;
use crate::actors::player_actor::{PlayerActor, PlayerMessage};
use dashmap::DashMap;
use shared::persistence::PlayerDao;
use shared::static_config::StaticConfig;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tracing::{info, warn};

/// 玩家管理器：维护在线玩家的 AccountID/RoleID -> Actor Sender 的映射
pub struct PlayerManager {
    /// AccountID -> Sender
    account_to_actor: DashMap<i64, mpsc::UnboundedSender<PlayerMessage>>,
    /// RoleID -> Sender
    role_to_actor: DashMap<i64, mpsc::UnboundedSender<PlayerMessage>>,
    /// 全服事件总线
    global_event_bus: GlobalEventBus,
    /// 静态配置订阅
    config_rx: watch::Receiver<Arc<StaticConfig>>,
    /// 数据库访问
    dao: Arc<PlayerDao>,
}

impl PlayerManager {
    pub fn new(
        global_event_bus: GlobalEventBus,
        config_rx: watch::Receiver<Arc<StaticConfig>>,
        dao: Arc<PlayerDao>,
    ) -> Self {
        Self {
            account_to_actor: DashMap::new(),
            role_to_actor: DashMap::new(),
            global_event_bus,
            config_rx,
            dao,
        }
    }

    /// 获取玩家 Actor 的发送端（按 account_id）
    pub fn get_by_account(&self, account_id: i64) -> Option<mpsc::UnboundedSender<PlayerMessage>> {
        self.account_to_actor.get(&account_id).map(|s| s.clone())
    }

    /// 获取玩家 Actor 的发送端（按 role_id）
    pub fn get_by_role(&self, role_id: i64) -> Option<mpsc::UnboundedSender<PlayerMessage>> {
        self.role_to_actor.get(&role_id).map(|s| s.clone())
    }

    /// 在线玩家数
    pub fn online_count(&self) -> usize {
        self.role_to_actor.len()
    }

    /// 启动新玩家 Actor
    ///
    /// 如果该 account_id 已有在线 Actor，先踢掉旧的。
    pub fn spawn_actor(
        &self,
        account_id: i64,
        role_id: i64,
    ) -> mpsc::UnboundedSender<PlayerMessage> {
        if let Some(existing) = self.role_to_actor.get(&role_id) {
            return existing.clone();
        }

        // 踢掉旧 Actor（如果存在）
        if let Some(old_tx) = self.account_to_actor.get(&account_id) {
            warn!(account_id, "Kicking old actor for duplicate login");
            let _ = old_tx.send(PlayerMessage::Shutdown);
        }

        let (tx, rx) = mpsc::unbounded_channel();

        let actor = PlayerActor::new(
            account_id,
            role_id,
            rx,
            self.global_event_bus.clone(),
            self.config_rx.clone(),
            self.dao.clone(),
        );
        tokio::spawn(async move {
            actor.run().await;
        });

        self.account_to_actor.insert(account_id, tx.clone());
        if role_id > 0 {
            self.role_to_actor.insert(role_id, tx.clone());
        }

        info!(account_id, role_id, "PlayerActor spawned");
        tx
    }

    /// 移除玩家（下线后清理映射）
    pub fn remove_player(&self, account_id: i64, role_id: i64) {
        let removed_by_role = if role_id > 0 {
            self.role_to_actor.remove(&role_id).map(|(_, tx)| tx)
        } else {
            None
        };

        if account_id > 0 {
            self.account_to_actor.remove(&account_id);
        } else if let Some(role_tx) = removed_by_role {
            self.account_to_actor
                .retain(|_, account_tx| !account_tx.same_channel(&role_tx));
        }
    }

    /// 优雅关闭：通知所有在线玩家存盘下线
    pub async fn shutdown_all(&self) {
        info!(online = self.online_count(), "Shutting down all players...");
        for entry in self.account_to_actor.iter() {
            let _ = entry.value().send(PlayerMessage::Shutdown);
        }
        // 等待一段时间让 Actor 完成存盘
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        self.account_to_actor.clear();
        self.role_to_actor.clear();
        info!("All players shut down");
    }
}
