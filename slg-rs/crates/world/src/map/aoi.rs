use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// AOI 管理器：维护哪些玩家在关注哪些 Grid
pub struct AoiManager {
    // GridID -> Set<AccountID>
    grid_subscriptions: DashMap<i32, HashSet<i64>>,
}

impl AoiManager {
    pub fn new() -> Self {
        Self {
            grid_subscriptions: DashMap::new(),
        }
    }

    /// 玩家进入/更新位置，更新订阅
    pub fn update_watcher(&self, account_id: i64, old_gids: &[i32], new_gids: &[i32]) {
        // 1. 移除不再关注的 Grid
        for &gid in old_gids {
            if !new_gids.iter().any(|&n| n == gid) {
                if let Some(mut set) = self.grid_subscriptions.get_mut(&gid) {
                    set.remove(&account_id);
                }
            }
        }
        
        // 2. 添加新关注的 Grid
        for &gid in new_gids {
            if !old_gids.iter().any(|&o| o == gid) {
                self.grid_subscriptions.entry(gid).or_default().insert(account_id);
            }
        }
    }

    /// 获取某个 Grid 的所有订阅者 (AccountID)
    pub fn get_subscribers(&self, gid: i32) -> Vec<i64> {
        self.grid_subscriptions
            .get(&gid)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 移除关注者 (断开连接)
    pub fn remove_watcher(&self, account_id: i64, gids: &[i32]) {
        for &gid in gids {
            if let Some(mut set) = self.grid_subscriptions.get_mut(&gid) {
                set.remove(&account_id);
            }
        }
    }
}
