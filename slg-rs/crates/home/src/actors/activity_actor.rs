use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};
use crate::systems::activity::model::GlobalActivityData;
use crate::systems::activity::lifecycle::current_timestamp;
use shared::event::GlobalEvent;
use shared::static_config::StaticConfig;
use std::sync::Arc;
use tokio::sync::watch;

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
    /// 全服事件转发
    GlobalEventReceived(GlobalEvent),
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
    /// 全服事件接收端
    global_event_rx: mpsc::Receiver<GlobalEvent>,
    /// 静态配置订阅
    config_rx: watch::Receiver<Arc<StaticConfig>>,
    /// 当前生效的静态配置快照
    current_config: Arc<StaticConfig>,
}

impl ActivityActor {
    pub fn new(
        rx: mpsc::Receiver<ActivityMessage>,
        global_event_rx: mpsc::Receiver<GlobalEvent>,
        config_rx: watch::Receiver<Arc<StaticConfig>>,
    ) -> Self {
        let current_config = config_rx.borrow().clone();
        Self {
            global_activities: HashMap::new(),
            rx,
            notify_tx: None,
            global_event_rx,
            config_rx,
            current_config,
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
                Some(event) = self.global_event_rx.recv() => {
                    self.handle_global_event(event).await;
                }
                Ok(()) = self.config_rx.changed() => {
                    info!("ActivityActor received static config reload");
                    self.current_config = self.config_rx.borrow().clone();
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
            ActivityMessage::GlobalEventReceived(event) => {
                self.handle_global_event(event).await;
            }
        }
    }

    async fn handle_global_event(&mut self, event: GlobalEvent) {
        match event {
            GlobalEvent::WorldMilestoneMission {
                role_id: _,
                mission_type: _,
                params: _,
            } => {
                // TODO: 更新全服里程碑进度
            }
            GlobalEvent::CampMilestoneMission {
                role_id: _,
                camp_id: _,
                mission_type: _,
                params: _,
            } => {
                // TODO: 更新阵营里程碑进度
            }
            GlobalEvent::RankUpdate {
                rank_type: _,
                role_id: _,
                value: _,
            } => {
                // TODO: 更新排行榜
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
