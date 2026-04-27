use std::collections::HashMap;
use std::time::Duration;
use tokio::task::{JoinSet, JoinHandle};
use anyhow::Result;
use tracing::{info, error, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorId {
    Sector(i32),
}

pub enum RestartPolicy {
    Always { max_retries: u32, within: Duration },
    ExponentialBackoff { initial: Duration, max: Duration, max_retries: u32 },
}

pub struct ActorSupervisor {
    /// 被监督的 Actor 集合 (JoinSet 管理 task)
    actors: JoinSet<Result<ActorId>>,
    /// 重启策略
    restart_policy: RestartPolicy,
    /// 重启计数和上次重启时间
    restart_counts: HashMap<ActorId, (u32, std::time::Instant)>,
}

impl ActorSupervisor {
    pub fn new(policy: RestartPolicy) -> Self {
        Self {
            actors: JoinSet::new(),
            restart_policy: policy,
            restart_counts: HashMap::new(),
        }
    }

    pub fn spawn<F>(&mut self, actor_id: ActorId, future: F)
    where
        F: std::future::Future<Output = Result<ActorId>> + Send + 'static,
    {
        info!("Spawning actor {:?}", actor_id);
        self.actors.spawn(future);
    }

    pub async fn run_supervision_loop(&mut self) {
        info!("Actor Supervisor loop started");
        
        while let Some(res) = self.actors.join_next().await {
            match res {
                Ok(Ok(id)) => {
                    info!("Actor {:?} completed normally", id);
                }
                Ok(Err(id_err)) => {
                    error!("Actor logic error: {}", id_err);
                    // 这里可以根据错误决定是否重启
                    // 暂时统一尝试重启
                }
                Err(join_err) => {
                    if join_err.is_panic() {
                        error!("Actor panic detected!");
                        // TODO: 从 panic 恢复 logic
                    } else if join_err.is_cancelled() {
                        warn!("Actor task cancelled");
                    }
                }
            }
        }
        
        info!("Actor Supervisor loop stopped");
    }
}
