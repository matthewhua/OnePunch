use anyhow::{anyhow, Result};
use dashmap::DashMap;
use proto::slg::BaseTroop;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub const MARCH_TYPE_ATK_BANDIT: i32 = 1;
pub const MARCH_TYPE_ATK_PLAYER: i32 = 2;
pub const MARCH_TYPE_MINE_COLLECT: i32 = 3;
pub const MARCH_TYPE_ATK_CITY: i32 = 4;
pub const MARCH_TYPE_INTEL_TASK: i32 = 5;
pub const MARCH_TYPE_GARRISON_PLAYER: i32 = 6;
pub const MARCH_TYPE_SPONSOR_ASSEMBLY: i32 = 7;
pub const MARCH_TYPE_JOIN_ASSEMBLY: i32 = 8;
pub const MARCH_TYPE_GARRISON_CITY: i32 = 9;
pub const MARCH_TYPE_SPONSOR_ASSEMBLY_CITY: i32 = 10;
pub const MARCH_TYPE_NPC: i32 = 11;
pub const MARCH_TYPE_SPONSOR_ASSEMBLY_HUNTING_TRAP: i32 = 12;
pub const MARCH_TYPE_MULTI_PLAYER_BANDIT: i32 = 13;
pub const MARCH_TYPE_PIRATE_FLEET: i32 = 14;
pub const MARCH_TYPE_SCOUT: i32 = 15;

pub const MARCH_STATUS_MARCH: i32 = 1;
pub const MARCH_STATUS_RETREAT: i32 = 2;
pub const MARCH_STATUS_ARRIVAL: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrivalAction {
    None,
    Battle,
    Collect,
    Scout,
    Garrison,
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarchArrival {
    pub troop: BaseTroop,
    pub pos: i32,
    pub action: ArrivalAction,
}

#[derive(Debug, Clone)]
pub struct MarchingTroop {
    pub base: BaseTroop,
    /// 移动速度 (像素每秒)
    pub speed: f32,
}

/// 行军管理器：处理部队位移与到达
pub struct MarchingManager {
    // TroopKey -> Troop
    pub troops: DashMap<i32, MarchingTroop>,
    troop_owners: DashMap<i32, i64>,
}

impl MarchingManager {
    pub fn new() -> Self {
        Self {
            troops: DashMap::new(),
            troop_owners: DashMap::new(),
        }
    }

    pub fn set_troop_owner(&self, troop_key: i32, role_id: i64) {
        self.troop_owners.insert(troop_key, role_id);
    }

    pub fn troop_owner(&self, troop_key: i32) -> Option<i64> {
        self.troop_owners
            .get(&troop_key)
            .map(|owner| *owner.value())
    }

    pub fn clear_troop_owner(&self, troop_key: i32) {
        self.troop_owners.remove(&troop_key);
    }

    /// 发起行军
    pub fn start_march(&self, troop: BaseTroop, speed: f32) -> Result<BaseTroop> {
        let now = now_millis();
        self.start_march_at(troop, speed, now)
    }

    pub fn start_march_at(
        &self,
        mut troop: BaseTroop,
        speed: f32,
        now_ms: i64,
    ) -> Result<BaseTroop> {
        if !speed.is_finite() || speed <= 0.0 {
            return Err(anyhow!("march speed must be positive"));
        }
        if self.troops.contains_key(&troop.key) {
            return Err(anyhow!("troop key {} already marching", troop.key));
        }

        let origin = troop
            .origin
            .ok_or_else(|| anyhow!("march origin is required"))?;
        let goal = troop
            .goal
            .ok_or_else(|| anyhow!("march goal is required"))?;
        if !crate::map::grid::is_valid_pos(origin) {
            return Err(anyhow!("invalid march origin: {}", origin));
        }
        if !crate::map::grid::is_valid_pos(goal) {
            return Err(anyhow!("invalid march goal: {}", goal));
        }

        let distance = self.calculate_distance(origin, goal);
        let duration_ms = (distance / speed * 1000.0).ceil() as i64;

        troop.status = Some(MARCH_STATUS_MARCH);
        troop.start_time = Some(now_ms);
        troop.end_time = Some(now_ms + duration_ms);

        info!(
            "March started: Troop {} from {:?} to {:?}. Duration: {}ms",
            troop.key, troop.origin, troop.goal, duration_ms
        );

        self.troops.insert(
            troop.key,
            MarchingTroop {
                base: troop.clone(),
                speed,
            },
        );
        Ok(troop)
    }

    /// 计算两点间距离 (欧几里得距离)
    fn calculate_distance(&self, p1: i32, p2: i32) -> f32 {
        use crate::map::grid::pos_to_xy;
        let (x1, y1) = pos_to_xy(p1);
        let (x2, y2) = pos_to_xy(p2);

        (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f32).sqrt()
    }

    /// 召回行军部队：部队从当前位置掉头返回原始起点。
    pub fn recall_troop(&self, troop_key: i32, recall_type: Option<i32>) -> Result<BaseTroop> {
        self.recall_troop_at(troop_key, recall_type, now_millis())
    }

    pub fn recall_troop_at(
        &self,
        troop_key: i32,
        recall_type: Option<i32>,
        now_ms: i64,
    ) -> Result<BaseTroop> {
        let mut entry = self
            .troops
            .get_mut(&troop_key)
            .ok_or_else(|| anyhow!("troop {} not found", troop_key))?;
        let troop = &mut entry.base;

        if troop.status != Some(MARCH_STATUS_MARCH) {
            return Err(anyhow!("troop {} is not marching", troop_key));
        }

        let origin = troop
            .origin
            .ok_or_else(|| anyhow!("march origin is required"))?;
        let goal = troop
            .goal
            .ok_or_else(|| anyhow!("march goal is required"))?;
        let start_time = troop
            .start_time
            .ok_or_else(|| anyhow!("march start time is required"))?;
        let end_time = troop
            .end_time
            .ok_or_else(|| anyhow!("march end time is required"))?;
        let total_ms = (end_time - start_time).max(0);
        let elapsed_ms = (now_ms - start_time).clamp(0, total_ms);
        let current_pos = interpolate_pos(origin, goal, start_time, end_time, now_ms);
        let return_ms = match recall_type.unwrap_or(1) {
            2 => 1_000,
            _ => elapsed_ms,
        };

        troop.origin = Some(current_pos);
        troop.goal = Some(origin);
        troop.status = Some(MARCH_STATUS_RETREAT);
        troop.start_time = Some(now_ms);
        troop.end_time = Some(now_ms + return_ms);

        Ok(troop.clone())
    }

    /// 加速返回中的部队。普通加速减半剩余时间，高级加速压到 1 秒内。
    pub fn accelerate_troop(
        &self,
        troop_key: i32,
        accelerate_type: Option<i32>,
    ) -> Result<BaseTroop> {
        self.accelerate_troop_at(troop_key, accelerate_type, now_millis())
    }

    pub fn accelerate_troop_at(
        &self,
        troop_key: i32,
        accelerate_type: Option<i32>,
        now_ms: i64,
    ) -> Result<BaseTroop> {
        let mut entry = self
            .troops
            .get_mut(&troop_key)
            .ok_or_else(|| anyhow!("troop {} not found", troop_key))?;
        let troop = &mut entry.base;

        if troop.status != Some(MARCH_STATUS_RETREAT) {
            return Err(anyhow!("troop {} is not retreating", troop_key));
        }

        let end_time = troop
            .end_time
            .ok_or_else(|| anyhow!("march end time is required"))?;
        let remaining_ms = (end_time - now_ms).max(0);
        let new_remaining_ms = match accelerate_type.unwrap_or(1) {
            2 => remaining_ms.min(1_000),
            _ => (remaining_ms / 2).max((remaining_ms > 0) as i64),
        };
        troop.end_time = Some(now_ms + new_remaining_ms);

        Ok(troop.clone())
    }

    /// 扫秒并检查到达的部队
    pub fn tick(&self) -> Vec<i32> {
        self.tick_at(now_millis())
            .into_iter()
            .map(|arrival| arrival.troop.key)
            .collect()
    }

    pub fn tick_at(&self, now_ms: i64) -> Vec<MarchArrival> {
        let mut arrived_keys: Vec<i32> = self
            .troops
            .iter()
            .filter_map(|entry| {
                let troop = &entry.value().base;
                troop
                    .end_time
                    .filter(|end_time| now_ms >= *end_time)
                    .map(|_| *entry.key())
            })
            .collect();
        arrived_keys.sort_unstable();

        let mut arrivals = Vec::with_capacity(arrived_keys.len());
        for key in arrived_keys {
            if let Some((_, marching)) = self.troops.remove(&key) {
                let mut troop = marching.base;
                troop.status = Some(MARCH_STATUS_ARRIVAL);
                let pos = troop.goal.unwrap_or(0);
                arrivals.push(MarchArrival {
                    action: arrival_action_for_troop(&troop),
                    troop,
                    pos,
                });
            }
        }

        arrivals
    }
}

pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn interpolate_pos(origin: i32, goal: i32, start_time: i64, end_time: i64, now_ms: i64) -> i32 {
    if end_time <= start_time {
        return goal;
    }

    let elapsed = (now_ms - start_time).clamp(0, end_time - start_time) as f64;
    let ratio = elapsed / (end_time - start_time) as f64;
    let (origin_x, origin_y) = crate::map::grid::pos_to_xy(origin);
    let (goal_x, goal_y) = crate::map::grid::pos_to_xy(goal);
    let x = origin_x as f64 + (goal_x - origin_x) as f64 * ratio;
    let y = origin_y as f64 + (goal_y - origin_y) as f64 * ratio;
    crate::map::grid::xy_to_pos(x.round() as i32, y.round() as i32)
}

pub fn arrival_action_for_troop(troop: &BaseTroop) -> ArrivalAction {
    if troop.status == Some(MARCH_STATUS_RETREAT) {
        return ArrivalAction::Return;
    }

    match troop.r#type.unwrap_or_default() {
        MARCH_TYPE_ATK_BANDIT
        | MARCH_TYPE_ATK_PLAYER
        | MARCH_TYPE_ATK_CITY
        | MARCH_TYPE_SPONSOR_ASSEMBLY
        | MARCH_TYPE_JOIN_ASSEMBLY
        | MARCH_TYPE_SPONSOR_ASSEMBLY_CITY
        | MARCH_TYPE_NPC
        | MARCH_TYPE_SPONSOR_ASSEMBLY_HUNTING_TRAP
        | MARCH_TYPE_MULTI_PLAYER_BANDIT
        | MARCH_TYPE_PIRATE_FLEET => ArrivalAction::Battle,
        MARCH_TYPE_MINE_COLLECT => ArrivalAction::Collect,
        MARCH_TYPE_SCOUT => ArrivalAction::Scout,
        MARCH_TYPE_GARRISON_PLAYER | MARCH_TYPE_GARRISON_CITY => ArrivalAction::Garrison,
        _ => ArrivalAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::xy_to_pos;

    fn troop(key: i32, troop_type: i32, origin: i32, goal: i32) -> BaseTroop {
        BaseTroop {
            key,
            r#type: Some(troop_type),
            origin: Some(origin),
            goal: Some(goal),
            ..Default::default()
        }
    }

    #[test]
    fn start_march_at_sets_status_and_arrival_time() {
        let manager = MarchingManager::new();
        let origin = xy_to_pos(0, 0);
        let goal = xy_to_pos(3, 4);

        let started = manager
            .start_march_at(troop(1, MARCH_TYPE_ATK_PLAYER, origin, goal), 2.5, 1_000)
            .unwrap();

        assert_eq!(started.status, Some(MARCH_STATUS_MARCH));
        assert_eq!(started.start_time, Some(1_000));
        assert_eq!(started.end_time, Some(3_000));
        assert!(manager.troops.contains_key(&1));
    }

    #[test]
    fn tick_at_returns_arrival_with_trigger_action() {
        let manager = MarchingManager::new();
        let origin = xy_to_pos(0, 0);
        let goal = xy_to_pos(3, 4);

        manager
            .start_march_at(troop(1, MARCH_TYPE_ATK_PLAYER, origin, goal), 5.0, 1_000)
            .unwrap();

        assert!(manager.tick_at(1_999).is_empty());
        let arrived = manager.tick_at(2_000);

        assert_eq!(arrived.len(), 1);
        assert_eq!(arrived[0].troop.key, 1);
        assert_eq!(arrived[0].pos, goal);
        assert_eq!(arrived[0].action, ArrivalAction::Battle);
        assert!(!manager.troops.contains_key(&1));
    }

    #[test]
    fn recall_troop_at_turns_march_back_from_current_position() {
        let manager = MarchingManager::new();
        let origin = xy_to_pos(0, 0);
        let goal = xy_to_pos(10, 0);

        manager
            .start_march_at(troop(1, MARCH_TYPE_ATK_PLAYER, origin, goal), 1.0, 1_000)
            .unwrap();

        let recalled = manager.recall_troop_at(1, Some(1), 6_000).unwrap();

        assert_eq!(recalled.status, Some(MARCH_STATUS_RETREAT));
        assert_eq!(recalled.origin, Some(xy_to_pos(5, 0)));
        assert_eq!(recalled.goal, Some(origin));
        assert_eq!(recalled.start_time, Some(6_000));
        assert_eq!(recalled.end_time, Some(11_000));
    }

    #[test]
    fn accelerate_troop_at_only_accepts_retreating_troops() {
        let manager = MarchingManager::new();
        let origin = xy_to_pos(0, 0);
        let goal = xy_to_pos(10, 0);

        manager
            .start_march_at(troop(1, MARCH_TYPE_ATK_PLAYER, origin, goal), 1.0, 1_000)
            .unwrap();
        assert!(manager.accelerate_troop_at(1, Some(1), 2_000).is_err());

        manager.recall_troop_at(1, Some(1), 6_000).unwrap();
        let accelerated = manager.accelerate_troop_at(1, Some(1), 7_000).unwrap();

        assert_eq!(accelerated.status, Some(MARCH_STATUS_RETREAT));
        assert_eq!(accelerated.end_time, Some(9_000));

        let accelerated = manager.accelerate_troop_at(1, Some(2), 8_000).unwrap();
        assert_eq!(accelerated.end_time, Some(9_000));
    }

    #[test]
    fn rejects_bad_speed_duplicate_key_and_invalid_positions() {
        let manager = MarchingManager::new();
        let origin = xy_to_pos(0, 0);
        let goal = xy_to_pos(3, 4);

        assert!(manager
            .start_march_at(troop(1, MARCH_TYPE_ATK_PLAYER, origin, goal), 0.0, 1_000)
            .is_err());

        manager
            .start_march_at(troop(1, MARCH_TYPE_ATK_PLAYER, origin, goal), 1.0, 1_000)
            .unwrap();

        assert!(manager
            .start_march_at(troop(1, MARCH_TYPE_ATK_PLAYER, origin, goal), 1.0, 1_000)
            .is_err());
        assert!(manager
            .start_march_at(troop(2, MARCH_TYPE_ATK_PLAYER, -1, goal), 1.0, 1_000)
            .is_err());
    }

    #[test]
    fn tracks_troop_owner_for_outbound_resolution() {
        let manager = MarchingManager::new();

        manager.set_troop_owner(7, 900_001);
        assert_eq!(manager.troop_owner(7), Some(900_001));

        manager.clear_troop_owner(7);
        assert_eq!(manager.troop_owner(7), None);
    }
}
