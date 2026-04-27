use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::broadcast;
use std::time::Duration;
use tracing::{info, warn};

pub struct ShutdownCoordinator {
    /// 用于发送关机信号的广播通道
    shutdown_tx: broadcast::Sender<()>,
    /// 活跃 Actor 计数器
    actor_counter: Arc<AtomicUsize>,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self {
            shutdown_tx: tx,
            actor_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 订阅关机信号
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// 增加活跃 Actor 计数
    pub fn add_actor(&self) {
        self.actor_counter.fetch_add(1, Ordering::SeqCst);
    }

    /// 减少活跃 Actor 计数
    pub fn remove_actor(&self) {
        self.actor_counter.fetch_sub(1, Ordering::SeqCst);
    }

    /// 触发优雅关闭
    pub async fn shutdown(&self, timeout: Duration) {
        info!("Initiating graceful shutdown...");
        
        // 1. 发送广播信号
        let _ = self.shutdown_tx.send(());
        
        // 2. 等待所有 Actor 退出
        let start = std::time::Instant::now();
        while self.actor_counter.load(Ordering::SeqCst) > 0 {
            if start.elapsed() >= timeout {
                warn!("Shutdown timeout reached. Force quitting with {} actors remaining.", 
                    self.actor_counter.load(Ordering::SeqCst));
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        info!("Graceful shutdown completed.");
    }
}
