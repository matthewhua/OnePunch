use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use dashmap::DashMap;
use proto::slg::BaseTroop;
use tracing::{info, debug};

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
    pub fn start_march(&self, mut troop: BaseTroop, speed: f32) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        
        let origin = troop.origin.unwrap_or(0);
        let goal = troop.goal.unwrap_or(0);
        let distance = self.calculate_distance(origin, goal);
        let duration_ms = (distance / speed * 1000.0) as i64;
        
        troop.start_time = Some(now);
        troop.end_time = Some(now + duration_ms);
        
        info!("March started: Troop {} from {:?} to {:?}. Duration: {}ms", 
            troop.key, troop.origin, troop.goal, duration_ms);
            
        self.troops.insert(troop.key, MarchingTroop { base: troop, speed });
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
            
        let mut arrived = Vec::new();
        
        // 此处为了性能，生产环境应使用 PriorityQueue 或 TimerWheel
        self.troops.retain(|key, troop| {
            if let Some(end) = troop.base.end_time {
                if now >= end {
                    arrived.push(*key);
                    return false; // 移除
                }
            }
            true
        });
        
        arrived
    }
}
