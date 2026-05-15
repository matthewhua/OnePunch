use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use proto::slg::BaseTroop;
use tracing::info;

pub const MARCH_TYPE_ATK_BANDIT: i32 = 1;
pub const MARCH_TYPE_ATK_PLAYER: i32 = 2;
pub const MARCH_TYPE_MINE_COLLECT: i32 = 3;
pub const MARCH_TYPE_ATK_CITY: i32 = 4;
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
}

impl MarchingManager {
    pub fn new() -> Self {
        Self {
            troops: DashMap::new(),
        }
    }

    /// 发起行军
    pub fn start_march(&self, troop: BaseTroop, speed: f32) -> Result<BaseTroop> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        self.start_march_at(troop, speed, now)
    }

    pub fn start_march_at(&self, mut troop: BaseTroop, speed: f32, now_ms: i64) -> Result<BaseTroop> {
        if !speed.is_finite() || speed <= 0.0 {
            return Err(anyhow!("march speed must be positive"));
        }
        if self.troops.contains_key(&troop.key) {
            return Err(anyhow!("troop key {} already marching", troop.key));
        }

        let origin = troop.origin.ok_or_else(|| anyhow!("march origin is required"))?;
        let goal = troop.goal.ok_or_else(|| anyhow!("march goal is required"))?;
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

        info!("March started: Troop {} from {:?} to {:?}. Duration: {}ms", 
            troop.key, troop.origin, troop.goal, duration_ms);

        self.troops.insert(troop.key, MarchingTroop { base: troop.clone(), speed });
        Ok(troop)
    }

    /// 计算两点间距离 (欧几里得距离)
    fn calculate_distance(&self, p1: i32, p2: i32) -> f32 {
        use crate::map::grid::pos_to_xy;
        let (x1, y1) = pos_to_xy(p1);
        let (x2, y2) = pos_to_xy(p2);
        
        (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f32).sqrt()
    }

    /// 扫秒并检查到达的部队
    pub fn tick(&self) -> Vec<i32> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        self.tick_at(now)
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
}
