use shared::event::GlobalEvent;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct GlobalEventBus {
    activity_tx: mpsc::Sender<GlobalEvent>,
    // milestone_tx: mpsc::Sender<GlobalEvent>,  // 未来扩展
}

impl GlobalEventBus {
    pub fn new(activity_tx: mpsc::Sender<GlobalEvent>) -> Self {
        Self { activity_tx }
    }

    /// 发布全服事件（fire-and-forget）
    pub fn publish(&self, event: GlobalEvent) {
        let tx = self.activity_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(event).await;
        });
    }
}
