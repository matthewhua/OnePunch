use dashmap::DashMap;
use proto::slg::{BaseEntity, BaseTroop};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

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
    /// GridID -> AccountID -> Sender (用于即时广播和按玩家取消订阅)
    broadcasters: DashMap<i32, HashMap<i64, mpsc::Sender<AoiEvent>>>,
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
        let mut closed_accounts = Vec::new();
        let mut remove_broadcaster_entry = false;

        if let Some(mut senders) = self.broadcasters.get_mut(&gid) {
            senders.retain(|account_id, tx| match tx.try_send(event.clone()) {
                Ok(_) => true,
                Err(mpsc::error::TrySendError::Full(_)) => true,
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    closed_accounts.push(*account_id);
                    false
                }
            });
            remove_broadcaster_entry = senders.is_empty();
        }

        if remove_broadcaster_entry {
            self.broadcasters.remove(&gid);
        }
        for account_id in closed_accounts {
            self.remove_subscription_record(gid, account_id);
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

    pub fn subscribe(&self, gid: i32, account_id: i64, tx: mpsc::Sender<AoiEvent>) {
        self.grid_subscriptions
            .entry(gid)
            .or_default()
            .insert(account_id);
        self.broadcasters
            .entry(gid)
            .or_default()
            .insert(account_id, tx);
    }

    pub fn subscribe_area(&self, center_pos: i32, account_id: i64, tx: mpsc::Sender<AoiEvent>) {
        for gid in Self::view_grid_ids(center_pos) {
            self.subscribe(gid, account_id, tx.clone());
        }
    }

    pub fn unsubscribe(&self, gid: i32, account_id: i64) {
        let mut remove_broadcaster_entry = false;
        if let Some(mut senders) = self.broadcasters.get_mut(&gid) {
            senders.remove(&account_id);
            remove_broadcaster_entry = senders.is_empty();
        }
        if remove_broadcaster_entry {
            self.broadcasters.remove(&gid);
        }

        self.remove_subscription_record(gid, account_id);
    }

    pub fn unsubscribe_area(&self, center_pos: i32, account_id: i64) {
        for gid in Self::view_grid_ids(center_pos) {
            self.unsubscribe(gid, account_id);
        }
    }

    pub fn move_subscription(
        &self,
        old_pos: i32,
        new_pos: i32,
        account_id: i64,
        tx: mpsc::Sender<AoiEvent>,
    ) {
        let old_gids = Self::view_grid_ids(old_pos);
        let new_gids = Self::view_grid_ids(new_pos);

        for gid in &new_gids {
            self.subscribe(*gid, account_id, tx.clone());
        }
        for gid in old_gids.difference(&new_gids) {
            self.unsubscribe(*gid, account_id);
        }
    }

    pub fn subscribers(&self, gid: i32) -> Vec<i64> {
        let Some(subscriptions) = self.grid_subscriptions.get(&gid) else {
            return Vec::new();
        };
        let mut account_ids: Vec<i64> = subscriptions.iter().copied().collect();
        account_ids.sort_unstable();
        account_ids
    }

    pub fn subscription_count(&self, gid: i32) -> usize {
        self.grid_subscriptions
            .get(&gid)
            .map(|subscriptions| subscriptions.len())
            .unwrap_or_default()
    }

    fn remove_subscription_record(&self, gid: i32, account_id: i64) {
        let mut remove_subscription_entry = false;
        if let Some(mut subscriptions) = self.grid_subscriptions.get_mut(&gid) {
            subscriptions.remove(&account_id);
            remove_subscription_entry = subscriptions.is_empty();
        }
        if remove_subscription_entry {
            self.grid_subscriptions.remove(&gid);
        }
    }

    fn view_grid_ids(center_pos: i32) -> HashSet<i32> {
        use crate::map::grid::MapGrid;
        MapGrid::get_view_grid_ids(center_pos).into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::{xy_to_pos, MapGrid};

    fn sorted_view_grid_ids(pos: i32) -> Vec<i32> {
        let mut gids = MapGrid::get_view_grid_ids(pos);
        gids.sort_unstable();
        gids
    }

    #[tokio::test]
    async fn subscribe_broadcast_and_unsubscribe_close_memory_loop() {
        let aoi = AoiManager::new();
        let (tx, mut rx) = mpsc::channel(4);

        aoi.subscribe(10, 42, tx);
        assert_eq!(aoi.subscribers(10), vec![42]);

        aoi.broadcast(
            10,
            AoiEvent::MarchArrive {
                troop_key: 7,
                pos: 99,
            },
        )
        .await;
        assert!(matches!(
            rx.recv().await,
            Some(AoiEvent::MarchArrive {
                troop_key: 7,
                pos: 99
            })
        ));

        aoi.unsubscribe(10, 42);
        assert_eq!(aoi.subscription_count(10), 0);

        aoi.broadcast(
            10,
            AoiEvent::MarchArrive {
                troop_key: 8,
                pos: 100,
            },
        )
        .await;
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn broadcast_prunes_closed_subscription_sender() {
        let aoi = AoiManager::new();
        let (tx, rx) = mpsc::channel(1);

        aoi.subscribe(11, 43, tx);
        drop(rx);

        aoi.broadcast(
            11,
            AoiEvent::MarchArrive {
                troop_key: 9,
                pos: 101,
            },
        )
        .await;

        assert_eq!(aoi.subscription_count(11), 0);
    }

    #[test]
    fn subscribe_and_unsubscribe_area_cover_view_grid_ids() {
        let aoi = AoiManager::new();
        let (tx, _rx) = mpsc::channel(4);
        let pos = xy_to_pos(125, 125);
        let gids = sorted_view_grid_ids(pos);

        aoi.subscribe_area(pos, 44, tx);

        for gid in &gids {
            assert_eq!(aoi.subscribers(*gid), vec![44]);
        }

        aoi.unsubscribe_area(pos, 44);

        for gid in gids {
            assert_eq!(aoi.subscription_count(gid), 0);
        }
    }

    #[test]
    fn move_subscription_unsubscribes_old_area_and_subscribes_new_area() {
        let aoi = AoiManager::new();
        let (tx, _rx) = mpsc::channel(4);
        let old_pos = xy_to_pos(125, 125);
        let new_pos = xy_to_pos(275, 125);
        let old_gids: HashSet<i32> = MapGrid::get_view_grid_ids(old_pos).into_iter().collect();
        let new_gids: HashSet<i32> = MapGrid::get_view_grid_ids(new_pos).into_iter().collect();

        aoi.subscribe_area(old_pos, 45, tx.clone());
        aoi.move_subscription(old_pos, new_pos, 45, tx);

        for gid in old_gids.difference(&new_gids) {
            assert_eq!(aoi.subscription_count(*gid), 0);
        }
        for gid in &new_gids {
            assert_eq!(aoi.subscribers(*gid), vec![45]);
        }
    }

    #[test]
    fn repeated_move_subscription_does_not_leak_duplicate_records() {
        let aoi = AoiManager::new();
        let (tx, _rx) = mpsc::channel(4);
        let old_pos = xy_to_pos(125, 125);
        let new_pos = xy_to_pos(275, 125);
        let new_gids = sorted_view_grid_ids(new_pos);

        aoi.subscribe_area(old_pos, 46, tx.clone());
        aoi.move_subscription(old_pos, new_pos, 46, tx.clone());
        aoi.move_subscription(old_pos, new_pos, 46, tx.clone());
        aoi.move_subscription(new_pos, new_pos, 46, tx);

        for gid in new_gids {
            assert_eq!(aoi.subscribers(gid), vec![46]);
            assert_eq!(aoi.subscription_count(gid), 1);
        }
    }
}
