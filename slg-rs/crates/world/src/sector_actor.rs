use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, broadcast, oneshot};
use crate::message::SectorMessage;
use crate::timer_wheel::TimerWheel;
use crate::supervisor::ActorId;
use crate::health::HealthChecker;
use crate::map::aoi::{AoiManager, AoiEvent};
use crate::march::{arrival_action_for_troop, ArrivalAction};
use crate::wal::{WriteAheadLog, WalEntry};
use proto::slg::{BaseEntity, BaseTroop};
use anyhow::Result;
use tracing::{info, error};
use bytes::Bytes;
use dashmap::DashMap;

pub struct MapSectorActor {
    pub sector_id: i32,
    /// 实体数据
    entities: HashMap<i32, BaseEntity>,
    /// 行军部队
    marching_troops: HashMap<i32, BaseTroop>,
    /// 定时任务
    timer_wheel: TimerWheel<i32>,
    
    /// 消息与通讯
    rx: mpsc::Receiver<SectorMessage>,
    neighbors: HashMap<i32, mpsc::Sender<SectorMessage>>,
    
    /// 外部组件
    aoi_manager: Arc<AoiManager>,
    health_checker: Arc<HealthChecker>,
    wal: WriteAheadLog,
    
    /// 关闭信号
    shutdown_rx: broadcast::Receiver<()>,
    tracked_troops: Arc<DashMap<i32, Vec<BaseTroop>>>,
}

impl MapSectorActor {
    pub fn new(
        sector_id: i32,
        rx: mpsc::Receiver<SectorMessage>,
        base_time_ms: i64,
        aoi_manager: Arc<AoiManager>,
        health_checker: Arc<HealthChecker>,
        wal: WriteAheadLog,
        shutdown_rx: broadcast::Receiver<()>,
        tracked_troops: Arc<DashMap<i32, Vec<BaseTroop>>>,
    ) -> Self {
        Self {
            sector_id,
            entities: HashMap::new(),
            marching_troops: HashMap::new(),
            timer_wheel: TimerWheel::new(base_time_ms),
            rx,
            neighbors: HashMap::new(),
            aoi_manager,
            health_checker,
            wal,
            shutdown_rx,
            tracked_troops,
        }
    }

    /// 启动时恢复数据
    pub async fn init_with_recovery(&mut self) -> Result<()> {
        let entries = self.wal.recover().await?;
        for entry in entries {
            match entry {
                WalEntry::MarchStart { key, origin, goal, start_time, end_time } => {
                    let troop = BaseTroop {
                        key,
                        origin: Some(origin),
                        goal: Some(goal),
                        start_time: Some(start_time),
                        end_time: Some(end_time),
                        ..Default::default()
                    };
                    self.track_troop(troop.clone());
                    self.marching_troops.insert(key, troop);
                    self.timer_wheel.schedule(end_time, key);
                }
                _ => {
                    // 处理其他恢复逻辑
                }
            }
        }
        Ok(())
    }

    pub async fn run(mut self) -> Result<ActorId> {
        info!("Sector Actor {} started", self.sector_id);
        let actor_id = ActorId::Sector(self.sector_id);
        
        // 执行恢复
        if let Err(e) = self.init_with_recovery().await {
            error!("Sector {} recovery failed: {}", self.sector_id, e);
        }
        
        loop {
            self.health_checker.heartbeat(actor_id);
            crate::metrics::world_metrics::record_marching_troops(self.sector_id, self.marching_troops.len());
            
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    crate::metrics::world_metrics::inc_messages_processed(self.sector_id);
                    if let Err(e) = self.handle_message(msg).await {
                        error!("Sector {} failed to handle message: {}", self.sector_id, e);
                    }
                }
                _ = self.shutdown_rx.recv() => {
                    info!("Sector Actor {} shutting down...", self.sector_id);
                    let _ = self.save_and_clean_wal().await;
                    break;
                }
                else => break,
            }
        }
        
        Ok(actor_id)
    }

    async fn handle_message(&mut self, msg: SectorMessage) -> Result<()> {
        match msg {
            SectorMessage::PlayerCommand { role_id, cmd, payload, reply } => {
                self.handle_player_command(role_id, cmd, payload, reply).await?;
            }
            SectorMessage::TransferTroop { troop_data } => {
                self.accept_transfer(troop_data).await?;
            }
            SectorMessage::Tick => {
                self.tick().await;
            }
            SectorMessage::ConfigReload => {
                // ...
            }
        }
        Ok(())
    }

    async fn handle_player_command(
        &mut self,
        _role_id: i64,
        _cmd: u32,
        _payload: Bytes,
        reply: oneshot::Sender<Result<Bytes>>,
    ) -> Result<()> {
        // TODO: 处理玩家请求
        let _ = reply.send(Ok(Bytes::new()));
        Ok(())
    }

    async fn accept_transfer(&mut self, troop: BaseTroop) -> Result<()> {
        // 先写日志
        self.wal.append(&WalEntry::MarchStart {
            key: troop.key,
            origin: troop.origin.unwrap_or(0),
            goal: troop.goal.unwrap_or(0),
            start_time: troop.start_time.unwrap_or(0),
            end_time: troop.end_time.unwrap_or(0),
        }).await?;
        
        // 内存处理
        let key = troop.key;
        self.marching_troops.insert(key, troop.clone());
        self.track_troop(troop.clone());
        if let Some(end_time) = troop.end_time {
            self.timer_wheel.schedule(end_time, key);
        }
        
        // AOI
        let pos = troop.origin.unwrap_or(0);
        self.aoi_manager
            .broadcast_area(pos, AoiEvent::MarchStart { troop })
            .await;
        
        Ok(())
    }

    async fn tick(&mut self) {
        let expired_keys = self.timer_wheel.advance();
        for key in expired_keys {
            if let Some(troop) = self.marching_troops.remove(&key) {
                self.untrack_troop(troop.key);
                let goal_pos = troop.goal.unwrap_or(0);
                if crate::map::grid::pos_to_sector_id(goal_pos) == self.sector_id {
                    self.handle_arrival(troop).await;
                } else {
                    self.transfer_to_neighbor(troop).await;
                }
            }
        }
    }

    async fn handle_arrival(&mut self, mut troop: BaseTroop) {
        let key = troop.key;
        let goal_pos = troop.goal.unwrap_or(0);
        troop.status = Some(crate::march::MARCH_STATUS_ARRIVAL);
        self.untrack_troop(key);

        let action = arrival_action_for_troop(&troop);
        match action {
            ArrivalAction::Battle => {
                let target = self.entities.get(&goal_pos);
                info!(
                    troop_key = key,
                    goal_pos,
                    target_exists = target.is_some(),
                    "Troop arrived and queued battle trigger"
                );
            }
            ArrivalAction::Collect => {
                info!(troop_key = key, goal_pos, "Troop arrived and started collect trigger");
            }
            ArrivalAction::Scout => {
                info!(troop_key = key, goal_pos, "Troop arrived and queued scout trigger");
            }
            ArrivalAction::Garrison => {
                info!(troop_key = key, goal_pos, "Troop arrived and queued garrison trigger");
            }
            ArrivalAction::Return => {
                info!(troop_key = key, goal_pos, "Troop returned to origin");
            }
            ArrivalAction::None => {
                info!(troop_key = key, goal_pos, "Troop arrived with no trigger action");
            }
        }

        self.aoi_manager
            .broadcast_area(goal_pos, AoiEvent::MarchArrive { troop_key: key, pos: goal_pos })
            .await;
    }

    async fn transfer_to_neighbor(&mut self, troop: BaseTroop) {
        let goal_pos = troop.goal.unwrap_or(0);
        let next_sector_id = crate::map::grid::pos_to_sector_id(goal_pos);
        
        // 记录转移日志
        let _ = self.wal.append(&WalEntry::TroopTransfer { 
            key: troop.key, 
            target_sector: next_sector_id 
        }).await;
        
        if let Some(neighbor_tx) = self.neighbors.get(&next_sector_id) {
            let _ = neighbor_tx.send(SectorMessage::TransferTroop { troop_data: troop }).await;
        }
    }

    fn track_troop(&self, troop: BaseTroop) {
        self.tracked_troops
            .entry(self.sector_id)
            .or_default()
            .push(troop);
    }

    fn untrack_troop(&self, troop_key: i32) {
        if let Some(mut entry) = self.tracked_troops.get_mut(&self.sector_id) {
            entry.retain(|troop| troop.key != troop_key);
        }
    }

    async fn save_and_clean_wal(&mut self) -> Result<()> {
        info!("Sector {} saving data and truncating WAL...", self.sector_id);
        // 这里执行数据库原子存盘
        // ...
        self.wal.truncate().await?;
        Ok(())
    }

    pub fn set_neighbor(&mut self, sector_id: i32, tx: mpsc::Sender<SectorMessage>) {
        self.neighbors.insert(sector_id, tx);
    }
}
