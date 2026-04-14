use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::mpsc;
use crate::player::{PlayerActor, PlayerMessage};

/// 全局会话管理器
/// 负责维护账号 ID 与玩家 Actor 通信通道的映射
pub struct SessionManager {
    // Key: AccountId (i64), Value: Actor Sender
    players: DashMap<i64, mpsc::UnboundedSender<PlayerMessage>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            players: DashMap::new(),
        }
    }

    /// 获取或创建一个玩家 Actor
    pub fn get_or_create_player(&self, account_id: i64) -> mpsc::UnboundedSender<PlayerMessage> {
        // 如果已存在，直接返回
        if let Some(sender) = self.players.get(&account_id) {
            return sender.clone();
        }

        // 如果不存在，创建新的 Actor 并启动协程
        let (tx, rx) = mpsc::unbounded_channel();
        let actor = PlayerActor::new(account_id, rx);
        
        // 启动玩家协程
        tokio::spawn(async move {
            actor.run().await;
        });

        self.players.insert(account_id, tx.clone());
        tx
    }

    /// 移除会话 (例如玩家彻底下线且内存释放时)
    pub fn remove_session(&self, account_id: i64) {
        self.players.remove(&account_id);
    }
}
