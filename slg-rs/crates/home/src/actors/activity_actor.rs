use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};
use crate::systems::activity::model::GlobalActivityData;
use crate::systems::activity::lifecycle::current_timestamp;

/// 发送给 ActivityActor 的消息
pub enum ActivityMessage {
    /// 每秒 Tick
    Tick,
    /// 查询全服活动快照
    GetGlobalActivity {
        activity_id: i32,
        reply: oneshot::Sender<Option<GlobalActivityData>>,
    },
    /// 排行更新消息
    UpdateRank {
        activity_id: i32,
        form_id: i32,
        role_id: i64,
        value: i64,
    },
}

/// ActivityActor 的通知（发往 Home/PlayerActor）
pub enum ActivityNotify {
    ActivityStateChanged {
        activity_id: i32,
        new_stage: crate::systems::activity::types::ActivityStage,
    },
}

pub struct ActivityActor {
    /// 全服活动实例数据
    global_activities: HashMap<i32, GlobalActivityData>,
    /// 消息接收端
    rx: mpsc::Receiver<ActivityMessage>,
    /// 向外部（如 PlayerActor）发送通知的通道（可选）
    notify_tx: Option<mpsc::Sender<ActivityNotify>>,
}

impl ActivityActor {
    pub fn new(rx: mpsc::Receiver<ActivityMessage>) -> Self {
        Self {
            global_activities: HashMap::new(),
            rx,
            notify_tx: None,
        }
    }

    pub async fn run(mut self) {
        info!("Global ActivityActor started");
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        
        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    self.handle_message(msg).await;
                }
                _ = interval.tick() => {
                    self.on_tick().await;
                }
            }
        }
    }

    async fn handle_message(&mut self, msg: ActivityMessage) {
        match msg {
            ActivityMessage::Tick => self.on_tick().await,
            ActivityMessage::GetGlobalActivity { activity_id, reply } => {
                let _ = reply.send(self.global_activities.get(&activity_id).cloned());
            }
            ActivityMessage::UpdateRank { .. } => {
                // TODO: 更新排行榜相关的 CommonForm
                warn!("UpdateRank not implemented yet");
            }
        }
    }

    /// 每秒心跳：驱动活动状态机流转
    async fn on_tick(&mut self) {
        let now = current_timestamp();
        for (activity_id, data) in self.global_activities.iter_mut() {
            if data.update_stage(now) {
                info!("Activity {} stage changed to {:?}", activity_id, data.stage);
                // 如果有通知通道，可以通知所有在线玩家
                if let Some(ref tx) = self.notify_tx {
                    let _ = tx.send(ActivityNotify::ActivityStateChanged {
                        activity_id: *activity_id,
                        new_stage: data.stage,
                    }).await;
                }
            }
        }
    }
}
