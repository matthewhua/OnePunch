use crate::arrival::resolve_arrival;
use crate::health::HealthChecker;
use crate::map::aoi::{AoiEvent, AoiManager};
use crate::march::{arrival_action_for_troop, ArrivalAction, MARCH_STATUS_ARRIVAL};
use crate::message::SectorMessage;
use crate::outbound::{
    outbound_events_for_action, InMemoryOutboundSink, WorldOutboundEvent, WorldOutboundSink,
};
use crate::supervisor::ActorId;
use crate::timer_wheel::TimerWheel;
use crate::wal::{WalEntry, WalReplayState, WalTroop, WriteAheadLog};
use anyhow::Result;
use bytes::Bytes;
use dashmap::DashMap;
use proto::slg::{BaseEntity, BaseTroop, GarrisonTroop};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{error, info};

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
    garrison_state: Arc<crate::garrison::GarrisonState>,
    _assembly_state: Arc<crate::assembly::AssemblyState>,
    outbound_sink: Arc<dyn WorldOutboundSink>,
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
        garrison_state: Arc<crate::garrison::GarrisonState>,
        assembly_state: Arc<crate::assembly::AssemblyState>,
        wal: WriteAheadLog,
        shutdown_rx: broadcast::Receiver<()>,
        tracked_troops: Arc<DashMap<i32, Vec<BaseTroop>>>,
    ) -> Self {
        Self::new_with_outbound(
            sector_id,
            rx,
            base_time_ms,
            aoi_manager,
            health_checker,
            garrison_state,
            assembly_state,
            Arc::new(InMemoryOutboundSink::new()),
            wal,
            shutdown_rx,
            tracked_troops,
        )
    }

    fn new_with_outbound(
        sector_id: i32,
        rx: mpsc::Receiver<SectorMessage>,
        base_time_ms: i64,
        aoi_manager: Arc<AoiManager>,
        health_checker: Arc<HealthChecker>,
        garrison_state: Arc<crate::garrison::GarrisonState>,
        assembly_state: Arc<crate::assembly::AssemblyState>,
        outbound_sink: Arc<dyn WorldOutboundSink>,
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
            garrison_state,
            _assembly_state: assembly_state,
            outbound_sink,
            wal,
            shutdown_rx,
            tracked_troops,
        }
    }

    /// 启动时恢复数据
    pub async fn init_with_recovery(&mut self) -> Result<()> {
        let entries = self.wal.recover().await?;
        let state = WalReplayState::from_entries(&entries);
        self.entities = state.entities;

        for (key, troop) in state.troops {
            if troop.status == Some(MARCH_STATUS_ARRIVAL) {
                continue;
            }

            self.track_troop(troop.clone());
            if let Some(end_time) = troop.end_time {
                self.timer_wheel.schedule(end_time, key);
            }
            self.marching_troops.insert(key, troop);
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
            crate::metrics::world_metrics::record_marching_troops(
                self.sector_id,
                self.marching_troops.len(),
            );

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
            SectorMessage::PlayerCommand {
                role_id,
                cmd,
                payload,
                reply,
            } => {
                self.handle_player_command(role_id, cmd, payload, reply)
                    .await?;
            }
            SectorMessage::TransferTroop { troop_data } => {
                self.accept_transfer(troop_data).await?;
            }
            SectorMessage::UpdateTroop { troop_data } => {
                self.update_troop(troop_data).await?;
            }
            SectorMessage::RemoveTroop { troop_key } => {
                self.remove_troop(troop_key);
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
        self.wal
            .append(&WalEntry::TroopUpdated {
                troop: WalTroop::from(&troop),
            })
            .await?;

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

    async fn update_troop(&mut self, troop: BaseTroop) -> Result<()> {
        self.wal
            .append(&WalEntry::TroopUpdated {
                troop: WalTroop::from(&troop),
            })
            .await?;

        let key = troop.key;
        self.marching_troops.insert(key, troop.clone());
        self.track_troop(troop.clone());
        if let Some(end_time) = troop.end_time {
            self.timer_wheel.schedule(end_time, key);
        }

        let pos = troop.origin.or(troop.goal).unwrap_or(0);
        self.aoi_manager
            .broadcast_area(pos, AoiEvent::MarchStart { troop })
            .await;

        Ok(())
    }

    fn remove_troop(&mut self, troop_key: i32) {
        self.marching_troops.remove(&troop_key);
        self.untrack_troop(troop_key);
    }

    async fn tick(&mut self) {
        let expired_keys = self.timer_wheel.advance();
        for key in expired_keys {
            let is_due = self
                .marching_troops
                .get(&key)
                .and_then(|troop| troop.end_time)
                .map(|end_time| end_time <= self.timer_wheel.current_time_ms())
                .unwrap_or(false);
            if !is_due {
                continue;
            }

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
        let action = arrival_action_for_troop(&troop);
        let resolution = resolve_arrival(&troop, self.entities.get(&goal_pos));
        self.publish_outbound_events(outbound_events_for_action(&troop, action, goal_pos));
        troop.status = Some(crate::march::MARCH_STATUS_ARRIVAL);
        let _ = self
            .wal
            .append(&WalEntry::TroopArrived {
                troop: WalTroop::from(&troop),
            })
            .await;
        self.untrack_troop(key);

        match action {
            ArrivalAction::Battle => {
                info!(
                    troop_key = key,
                    goal_pos,
                    target_exists = resolution.target.is_some(),
                    effect = ?resolution.effect,
                    "Troop arrived and queued battle trigger"
                );
            }
            ArrivalAction::Collect => {
                info!(
                    troop_key = key,
                    goal_pos,
                    effect = ?resolution.effect,
                    "Troop arrived and started collect trigger"
                );
            }
            ArrivalAction::Scout => {
                info!(
                    troop_key = key,
                    goal_pos,
                    effect = ?resolution.effect,
                    "Troop arrived and queued scout trigger"
                );
            }
            ArrivalAction::Garrison => {
                if let Err(err) = self.garrison_state.place(
                    goal_pos,
                    GarrisonTroop {
                        troop_key_id: Some(key),
                        end_time: troop.end_time,
                        ..Default::default()
                    },
                ) {
                    error!(
                        troop_key = key,
                        goal_pos,
                        error = %err,
                        "Failed to place garrison troop"
                    );
                }
                info!(
                    troop_key = key,
                    goal_pos,
                    effect = ?resolution.effect,
                    "Troop arrived and queued garrison trigger"
                );
            }
            ArrivalAction::Return => {
                info!(
                    troop_key = key,
                    goal_pos,
                    effect = ?resolution.effect,
                    "Troop returned to origin"
                );
            }
            ArrivalAction::None => {
                info!(
                    troop_key = key,
                    goal_pos,
                    effect = ?resolution.effect,
                    "Troop arrived with no trigger action"
                );
            }
        }

        self.aoi_manager
            .broadcast_area(
                goal_pos,
                AoiEvent::MarchArrive {
                    troop_key: key,
                    pos: goal_pos,
                },
            )
            .await;
    }

    fn publish_outbound_events(&self, events: Vec<WorldOutboundEvent>) {
        for event in events {
            if let Err(err) = self.outbound_sink.publish(event) {
                error!(error = %err, "Failed to publish world outbound event");
            }
        }
    }

    async fn transfer_to_neighbor(&mut self, troop: BaseTroop) {
        let goal_pos = troop.goal.unwrap_or(0);
        let next_sector_id = crate::map::grid::pos_to_sector_id(goal_pos);

        // 记录转移日志
        let _ = self
            .wal
            .append(&WalEntry::TroopTransfer {
                key: troop.key,
                target_sector: next_sector_id,
            })
            .await;

        if let Some(neighbor_tx) = self.neighbors.get(&next_sector_id) {
            let _ = neighbor_tx
                .send(SectorMessage::TransferTroop { troop_data: troop })
                .await;
        }
    }

    fn track_troop(&self, troop: BaseTroop) {
        let mut entry = self.tracked_troops.entry(self.sector_id).or_default();
        if let Some(existing) = entry.iter_mut().find(|existing| existing.key == troop.key) {
            *existing = troop;
        } else {
            entry.push(troop);
        }
    }

    fn untrack_troop(&self, troop_key: i32) {
        if let Some(mut entry) = self.tracked_troops.get_mut(&self.sector_id) {
            entry.retain(|troop| troop.key != troop_key);
        }
    }

    async fn save_and_clean_wal(&mut self) -> Result<()> {
        info!(
            "Sector {} saving data and truncating WAL...",
            self.sector_id
        );
        // 这里执行数据库原子存盘
        // ...
        self.wal.truncate().await?;
        Ok(())
    }

    pub fn set_neighbor(&mut self, sector_id: i32, tx: mpsc::Sender<SectorMessage>) {
        self.neighbors.insert(sector_id, tx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::xy_to_pos;
    use crate::march::{MARCH_STATUS_RETREAT, MARCH_TYPE_ATK_PLAYER};
    use crate::outbound::WorldOutboundTarget;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    async fn actor_with_sink(
        sink: Arc<InMemoryOutboundSink>,
    ) -> (
        MapSectorActor,
        Arc<crate::garrison::GarrisonState>,
        std::path::PathBuf,
    ) {
        let (_tx, rx) = mpsc::channel(8);
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let _shutdown_tx = shutdown_tx;
        let tracked_troops = Arc::new(DashMap::new());
        let garrison_state = Arc::new(crate::garrison::GarrisonState::new());
        let wal_path = std::env::temp_dir().join(format!(
            "sector-arrival-outbound-test-{}-{}.wal",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let wal = WriteAheadLog::new(&wal_path).await.unwrap();
        let outbound_sink: Arc<dyn WorldOutboundSink> = sink;
        let actor = MapSectorActor::new_with_outbound(
            0,
            rx,
            0,
            Arc::new(AoiManager::new()),
            Arc::new(HealthChecker::new(Duration::from_secs(30))),
            garrison_state.clone(),
            Arc::new(crate::assembly::AssemblyState::new()),
            outbound_sink,
            wal,
            shutdown_rx,
            tracked_troops,
        );
        (actor, garrison_state, wal_path)
    }

    #[tokio::test]
    async fn arrival_publishes_battle_event() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, _garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let origin = xy_to_pos(0, 0);
        let goal = xy_to_pos(10, 10);

        actor
            .handle_arrival(BaseTroop {
                key: 1,
                r#type: Some(MARCH_TYPE_ATK_PLAYER),
                origin: Some(origin),
                goal: Some(goal),
                camp: Some(2),
                ..Default::default()
            })
            .await;

        let records = sink.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].target(), WorldOutboundTarget::Battle);
        assert_eq!(
            records[0],
            WorldOutboundEvent::BattleStartRequested {
                troop_key: 1,
                march_type: Some(MARCH_TYPE_ATK_PLAYER),
                origin: Some(origin),
                target_pos: goal,
                camp: Some(2),
            }
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn retreat_arrival_publishes_return_event_before_status_changes() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, _garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let origin = xy_to_pos(5, 5);
        let home = xy_to_pos(0, 0);

        actor
            .handle_arrival(BaseTroop {
                key: 2,
                r#type: Some(MARCH_TYPE_ATK_PLAYER),
                origin: Some(origin),
                goal: Some(home),
                status: Some(MARCH_STATUS_RETREAT),
                ..Default::default()
            })
            .await;

        assert_eq!(
            sink.records(),
            vec![WorldOutboundEvent::TroopReturned {
                troop_key: 2,
                home_pos: home,
                march_type: Some(MARCH_TYPE_ATK_PLAYER),
            }]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn garrison_arrival_places_troop_in_shared_state() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let target = xy_to_pos(12, 12);

        actor
            .handle_arrival(BaseTroop {
                key: 3,
                r#type: Some(crate::march::MARCH_TYPE_GARRISON_CITY),
                origin: Some(xy_to_pos(1, 1)),
                goal: Some(target),
                end_time: Some(99_000),
                ..Default::default()
            })
            .await;

        let garrisons = garrison_state.list(Some(target));
        assert_eq!(garrisons.len(), 1);
        assert_eq!(garrisons[0].troop_key_id, Some(3));
        assert_eq!(garrisons[0].end_time, Some(99_000));
        assert_eq!(
            sink.records(),
            vec![WorldOutboundEvent::GarrisonChanged {
                troop_key: 3,
                target_pos: target,
                camp: None,
                is_arrival: true,
            }]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }
}
