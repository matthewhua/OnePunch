use dashmap::DashMap;
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::actors::player_actor::{PlayerActor, PlayerMessage};
use crate::actors::global_event_bus::GlobalEventBus;
use shared::static_config::StaticConfig;
use tokio::sync::watch;

/// 玩家管理器：维护在线玩家的 AccountID/RoleID -> Actor Sender 的映射
pub struct PlayerManager {
    // AccountID -> Sender
    account_to_actor: DashMap<i64, mpsc::UnboundedSender<PlayerMessage>>,
    // RoleID -> Sender
    role_to_actor: DashMap<i64, mpsc::UnboundedSender<PlayerMessage>>,
    // 全服事件总线
    global_event_bus: GlobalEventBus,
    // 静态配置订阅
    config_rx: watch::Receiver<Arc<StaticConfig>>,
}

impl PlayerManager {
    pub fn new(global_event_bus: GlobalEventBus, config_rx: watch::Receiver<Arc<StaticConfig>>) -> Self {
        Self {
            account_to_actor: DashMap::new(),
            role_to_actor: DashMap::new(),
            global_event_bus,
            config_rx,
        }
    }

    /// 获取玩家 Actor 的发送端
    pub fn get_by_account(&self, account_id: i64) -> Option<mpsc::UnboundedSender<PlayerMessage>> {
        self.account_to_actor.get(&account_id).map(|s| s.clone())
    }

    /// 启动新玩家 Actor (通常在 Login 成功后调用)
    pub fn spawn_actor(&self, account_id: i64, role_id: i64) -> mpsc::UnboundedSender<PlayerMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let actor = PlayerActor::new(
            account_id, 
            role_id, 
            rx, 
            self.global_event_bus.clone(),
            self.config_rx.clone()
        );
        tokio::spawn(async move {
            actor.run().await;
        });

        self.account_to_actor.insert(account_id, tx.clone());
        if role_id > 0 {
            self.role_to_actor.insert(role_id, tx.clone());
        }
        
        tx
    }

    /// 移除玩家 (下线)
    pub fn remove_player(&self, account_id: i64, role_id: i64) {
        self.account_to_actor.remove(&account_id);
        if role_id > 0 {
            self.role_to_actor.remove(&role_id);
        }
    }
}
