use dashmap::DashMap;
use std::collections::HashSet;
use tokio::sync::mpsc;
use proto::slg::{BaseTroop, BaseEntity};

/// AOI 事件类型
#[derive(Debug, Clone)]
pub enum AoiEvent {
    /// 实体进入视野
    EntityEnter { entity: BaseEntity },
    /// 实体离开视野
    EntityLeave { pos: i32 },
    /// 实体状态更新
    EntityUpdate { pos: i32, updates: Vec<u32> },
    /// 部队行军开始
    MarchStart { troop: BaseTroop },
    /// 部队到达
    MarchArrive { troop_key: i32, pos: i32 },
}

/// AOI 管理器：维护视野订阅与事件广播
pub struct AoiManager {
    /// GridID -> Set<AccountID> (用于持久化或查询谁在看哪里)
    grid_subscriptions: DashMap<i32, HashSet<i64>>,
    /// GridID -> List of senders (用于即时广播)
    /// 这里的 AccountID -> Sender 可以维护在 Gateway 侧，World 这边只管推到对应的 Grid 通道
    broadcasters: DashMap<i32, Vec<mpsc::Sender<AoiEvent>>>,
}

impl AoiManager {
    pub fn new() -> Self {
        Self {
            grid_subscriptions: DashMap::new(),
            broadcasters: DashMap::new(),
        }
    }

    /// 广播事件到指定格子
    pub async fn broadcast(&self, gid: i32, event: AoiEvent) {
        if let Some(mut senders) = self.broadcasters.get_mut(&gid) {
            // 清理已断开的连接并发送
            senders.retain(|tx| {
                match tx.try_send(event.clone()) {
                    Ok(_) => true,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        // 队列满，视具体情况丢弃或记录
                        true
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        // 连接已关闭，移除
                        false
                    }
                }
            });
        }
    }

    /// 针对 9 宫格进行广播
    pub async fn broadcast_area(&self, center_pos: i32, event: AoiEvent) {
        use crate::map::grid::MapGrid;
        let gids = MapGrid::get_view_grid_ids(center_pos);
        for gid in gids {
            self.broadcast(gid, event.clone()).await;
        }
    }

    pub fn subscribe(&self, gid: i32, tx: mpsc::Sender<AoiEvent>) {
        self.broadcasters.entry(gid).or_default().push(tx);
    }

    pub fn unsubscribe(&self, gid: i32, account_id: i64) {
        // 实现略，需要建立 Sender -> AccountID 的映射或者直接靠 Closed 自动清理
    }
}
