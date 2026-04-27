use std::collections::HashMap;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use crate::supervisor::ActorId;
use tracing::{warn, info};

pub struct HealthChecker {
    /// 记录每个 Actor 最后心跳时间
    heartbeats: DashMap<ActorId, Instant>,
    /// 判定为假死的阈值
    stale_threshold: Duration,
}

impl HealthChecker {
    pub fn new(stale_threshold: Duration) -> Self {
        Self {
            heartbeats: DashMap::new(),
            stale_threshold,
        }
    }

    /// Actor 上报心跳
    pub fn heartbeat(&self, id: ActorId) {
        self.heartbeats.insert(id, Instant::now());
    }

    /// 移除 Actor
    pub fn remove(&self, id: ActorId) {
        self.heartbeats.remove(&id);
    }

    /// 检查并返回假死的 Actor
    pub fn check_stale(&self) -> Vec<ActorId> {
        let now = Instant::now();
        let mut stale = Vec::new();
        for entry in self.heartbeats.iter() {
            if now.duration_since(*entry.value()) > self.stale_threshold {
                stale.push(*entry.key());
            }
        }
        stale
    }

    /// 运行健康检查循环
    pub async fn run_check_loop(&self) {
        info!("Health check loop started");
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let stale_actors = self.check_stale();
            for id in stale_actors {
                warn!("Actor {:?} seems to be stale/stuck!", id);
                // 这里可以触发告警或重启机制
            }
        }
    }
}
