use crate::arrival::resolve_arrival;
use crate::collect::{
    CollectCompletion, CollectPhase, CollectState, CollectStateMachine,
    DEFAULT_COLLECT_RETURN_DURATION_MS,
};
use crate::health::HealthChecker;
use crate::map::aoi::{AoiEvent, AoiManager};
use crate::march::{
    arrival_action_for_troop, ArrivalAction, MARCH_STATUS_ARRIVAL, MARCH_STATUS_RETREAT,
};
use crate::message::SectorMessage;
use crate::outbound::{
    outbound_events_for_resolution, InMemoryOutboundSink, WorldOutboundEvent, WorldOutboundSink,
};
use crate::supervisor::ActorId;
use crate::timer_wheel::TimerWheel;
use crate::wal::{WalCollectState, WalEntity, WalEntry, WalReplayState, WalTroop, WriteAheadLog};
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
    /// troop_key → formation_id
    troop_formations: HashMap<i32, i32>,
    /// 定时任务
    timer_wheel: TimerWheel<SectorTimerEvent>,

    /// 消息与通讯
    rx: mpsc::Receiver<SectorMessage>,
    neighbors: HashMap<i32, mpsc::Sender<SectorMessage>>,

    /// 外部组件
    aoi_manager: Arc<AoiManager>,
    health_checker: Arc<HealthChecker>,
    garrison_state: Arc<crate::garrison::GarrisonState>,
    _assembly_state: Arc<crate::assembly::AssemblyState>,
    collect_state: CollectStateMachine,
    outbound_sink: Arc<dyn WorldOutboundSink>,
    wal: WriteAheadLog,

    /// 关闭信号
    shutdown_rx: broadcast::Receiver<()>,
    tracked_troops: Arc<DashMap<i32, Vec<BaseTroop>>>,
    tracked_entities: Arc<DashMap<i32, Vec<BaseEntity>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SectorTimerEvent {
    MarchArrive { troop_key: i32 },
    CollectComplete { troop_key: i32 },
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
        tracked_entities: Arc<DashMap<i32, Vec<BaseEntity>>>,
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
            tracked_entities,
        )
    }

    pub(crate) fn new_with_outbound(
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
        tracked_entities: Arc<DashMap<i32, Vec<BaseEntity>>>,
    ) -> Self {
        Self {
            sector_id,
            entities: HashMap::new(),
            marching_troops: HashMap::new(),
            troop_formations: HashMap::new(),
            timer_wheel: TimerWheel::new(base_time_ms),
            rx,
            neighbors: HashMap::new(),
            aoi_manager,
            health_checker,
            garrison_state,
            _assembly_state: assembly_state,
            collect_state: CollectStateMachine::default(),
            outbound_sink,
            wal,
            shutdown_rx,
            tracked_troops,
            tracked_entities,
        }
    }

    /// 启动时恢复数据
    pub async fn init_with_recovery(&mut self) -> Result<()> {
        let entries = self.wal.recover().await?;
        let mut state = WalReplayState::from_entries(&entries);
        self.entities = std::mem::take(&mut state.entities);
        self.track_entities_snapshot();

        for (key, troop) in std::mem::take(&mut state.troops) {
            let collect_state = state.collect_states.remove(&key);
            if troop.status == Some(MARCH_STATUS_ARRIVAL) {
                match collect_state {
                    Some(state) if state.phase == CollectPhase::Collecting => {
                        self.restore_collect_state(state);
                    }
                    Some(_) => {}
                    None => {
                        let resolution =
                            resolve_arrival(&troop, self.entities.get(&troop.goal.unwrap_or(0)));
                        self.restore_arrived_troop(&troop, &resolution);
                    }
                }
                continue;
            }

            if let Some(collect_state) = collect_state {
                self.restore_collect_state(collect_state);
            }
            self.track_troop(troop.clone());
            if let Some(end_time) = troop.end_time {
                self.timer_wheel
                    .schedule(end_time, SectorTimerEvent::MarchArrive { troop_key: key });
            }
            self.marching_troops.insert(key, troop);
        }
        Ok(())
    }

    fn restore_arrived_troop(
        &mut self,
        troop: &BaseTroop,
        resolution: &crate::arrival::ArrivalResolution,
    ) {
        match resolution.effect {
            crate::arrival::ArrivalEffect::CollectStarted => {
                if let Some(state) = self
                    .collect_state
                    .on_arrival(troop, resolution, None)
                    .cloned()
                {
                    self.schedule_collect_completion(&state);
                }
            }
            crate::arrival::ArrivalEffect::GarrisonPlaced => {
                let goal_pos = troop.goal.unwrap_or(0);
                if let Err(err) = self
                    .garrison_state
                    .place(goal_pos, garrison_troop_from_arrival(troop))
                {
                    error!(
                        troop_key = troop.key,
                        goal_pos,
                        error = %err,
                        "Failed to restore arrived garrison troop"
                    );
                }
            }
            crate::arrival::ArrivalEffect::BattleRequested
            | crate::arrival::ArrivalEffect::ScoutReportRequested
            | crate::arrival::ArrivalEffect::ReturnedHome
            | crate::arrival::ArrivalEffect::Noop => {}
        }
    }

    fn restore_collect_state(&mut self, state: CollectState) {
        if let Some(formation_id) = state.formation_id.filter(|id| *id > 0) {
            self.troop_formations.insert(state.troop_key, formation_id);
        }
        self.schedule_collect_completion(&state);
        self.collect_state.insert(state);
    }

    fn schedule_collect_completion(&mut self, state: &CollectState) {
        if state.phase == CollectPhase::Collecting {
            self.timer_wheel.schedule(
                state.collect_end_time_ms,
                SectorTimerEvent::CollectComplete {
                    troop_key: state.troop_key,
                },
            );
        }
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
            SectorMessage::TransferTroop {
                troop_data,
                formation_id,
                collect_state,
            } => {
                self.accept_transfer(troop_data, formation_id, collect_state)
                    .await?;
            }
            SectorMessage::UpdateTroop { troop_data } => {
                self.update_troop(troop_data).await?;
            }
            SectorMessage::RemoveTroop { troop_key } => {
                self.remove_troop(troop_key);
            }
            SectorMessage::UpsertEntity { entity_data } => {
                self.upsert_entity(entity_data).await?;
            }
            SectorMessage::RemoveEntity { pos } => {
                self.remove_entity(pos).await?;
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

    async fn accept_transfer(
        &mut self,
        troop: BaseTroop,
        formation_id: Option<i32>,
        collect_state: Option<CollectState>,
    ) -> Result<()> {
        // 先写日志
        if let Some(collect_state) = collect_state.as_ref() {
            self.wal
                .append(&WalEntry::CollectTroopUpdated {
                    troop: WalTroop::from(&troop),
                    state: WalCollectState::from(collect_state),
                })
                .await?;
        } else {
            self.wal
                .append(&WalEntry::TroopUpdated {
                    troop: WalTroop::from(&troop),
                })
                .await?;
        }

        // 内存处理
        let key = troop.key;
        if let Some(collect_state) = collect_state {
            self.collect_state.insert(collect_state);
        }
        if let Some(formation_id) = formation_id.filter(|id| *id > 0) {
            self.troop_formations.insert(key, formation_id);
        }
        self.marching_troops.insert(key, troop.clone());
        self.track_troop(troop.clone());
        if let Some(end_time) = troop.end_time {
            self.timer_wheel
                .schedule(end_time, SectorTimerEvent::MarchArrive { troop_key: key });
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
            self.timer_wheel
                .schedule(end_time, SectorTimerEvent::MarchArrive { troop_key: key });
        }

        let pos = troop.origin.or(troop.goal).unwrap_or(0);
        self.aoi_manager
            .broadcast_area(pos, AoiEvent::MarchStart { troop })
            .await;

        Ok(())
    }

    fn remove_troop(&mut self, troop_key: i32) {
        self.marching_troops.remove(&troop_key);
        self.collect_state.remove(troop_key);
        self.troop_formations.remove(&troop_key);
        self.untrack_troop(troop_key);
    }

    async fn upsert_entity(&mut self, entity: BaseEntity) -> Result<()> {
        self.wal
            .append(&WalEntry::EntityUpsert {
                entity: WalEntity::from(&entity),
            })
            .await?;
        self.entities.insert(entity.pos, entity);
        self.track_entities_snapshot();
        Ok(())
    }

    async fn remove_entity(&mut self, pos: i32) -> Result<()> {
        self.wal.append(&WalEntry::EntityRemove { pos }).await?;
        self.entities.remove(&pos);
        self.track_entities_snapshot();
        Ok(())
    }

    async fn tick(&mut self) {
        let expired_events = self.timer_wheel.advance();
        for event in expired_events {
            match event {
                SectorTimerEvent::MarchArrive { troop_key } => {
                    self.handle_march_timer(troop_key).await;
                }
                SectorTimerEvent::CollectComplete { troop_key } => {
                    self.handle_collect_complete(troop_key).await;
                }
            }
        }
    }

    async fn handle_march_timer(&mut self, key: i32) {
        let is_due = self
            .marching_troops
            .get(&key)
            .and_then(|troop| troop.end_time)
            .map(|end_time| end_time <= self.timer_wheel.current_time_ms())
            .unwrap_or(false);
        if !is_due {
            return;
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

    async fn handle_collect_complete(&mut self, troop_key: i32) {
        let now_ms = self.timer_wheel.current_time_ms();
        let Some(completion) = self.collect_state.complete(troop_key, now_ms) else {
            return;
        };

        let troop = return_troop_from_collect_completion(&completion, now_ms);
        let wal_entry = self
            .collect_state
            .get(troop_key)
            .map(|state| WalEntry::CollectTroopUpdated {
                troop: WalTroop::from(&troop),
                state: WalCollectState::from(state),
            })
            .unwrap_or_else(|| WalEntry::TroopUpdated {
                troop: WalTroop::from(&troop),
            });
        if let Err(err) = self.wal.append(&wal_entry).await {
            error!(
                troop_key,
                error = %err,
                "Failed to persist collect return troop"
            );
        }

        self.marching_troops.insert(troop_key, troop.clone());
        self.track_troop(troop.clone());
        if let Some(formation_id) = completion.formation_id.filter(|id| *id > 0) {
            self.troop_formations.insert(troop_key, formation_id);
        }
        if let Some(end_time) = troop.end_time {
            self.timer_wheel
                .schedule(end_time, SectorTimerEvent::MarchArrive { troop_key });
        }

        self.aoi_manager
            .broadcast_area(completion.target_pos, AoiEvent::MarchStart { troop })
            .await;
    }

    async fn handle_arrival(&mut self, mut troop: BaseTroop) {
        let key = troop.key;
        let goal_pos = troop.goal.unwrap_or(0);
        let action = arrival_action_for_troop(&troop);
        let resolution = resolve_arrival(&troop, self.entities.get(&goal_pos));
        let collect_return = if action == ArrivalAction::Return {
            self.collect_state.finish_return(key)
        } else {
            None
        };
        let outbound_events = if let Some(state) = collect_return.as_ref() {
            vec![WorldOutboundEvent::CollectReturned {
                troop_key: key,
                target_pos: state.target_pos,
                home_pos: goal_pos,
                march_type: state.march_type,
                formation_id: state.formation_id,
                awards: vec![state.award()],
                collect_start_time_ms: state.start_time_ms,
                collect_end_time_ms: state.collect_end_time_ms,
            }]
        } else {
            outbound_events_for_resolution(&troop, &resolution)
        };
        let formation_id = self.troop_formations.remove(&key);
        let collect_state = self
            .collect_state
            .on_arrival(&troop, &resolution, formation_id)
            .cloned();
        if let Some(state) = collect_state.as_ref() {
            self.schedule_collect_completion(state);
        }
        troop.status = Some(crate::march::MARCH_STATUS_ARRIVAL);
        let wal_entry = if let Some(state) = collect_state.as_ref() {
            WalEntry::CollectTroopArrived {
                troop: WalTroop::from(&troop),
                state: WalCollectState::from(state),
            }
        } else if collect_return.is_some() {
            WalEntry::CollectReturnFinished {
                troop: WalTroop::from(&troop),
                troop_key: key,
            }
        } else {
            WalEntry::TroopArrived {
                troop: WalTroop::from(&troop),
            }
        };
        let _ = self.wal.append(&wal_entry).await;
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
                    collect_phase = ?collect_state.as_ref().map(|state| state.phase),
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
                if let Err(err) = self
                    .garrison_state
                    .place(goal_pos, garrison_troop_from_arrival(&troop))
                {
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
                    collect_return = collect_return.is_some(),
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

        self.publish_outbound_events(outbound_events);

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
        let collect_state = self.collect_state.remove(troop.key);
        let formation_id = self
            .troop_formations
            .remove(&troop.key)
            .or_else(|| collect_state.as_ref().and_then(|state| state.formation_id));

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
                .send(SectorMessage::TransferTroop {
                    troop_data: troop,
                    formation_id,
                    collect_state,
                })
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

    fn track_entities_snapshot(&self) {
        let mut entities: Vec<BaseEntity> = self.entities.values().cloned().collect();
        entities.sort_by_key(|entity| entity.pos);
        self.tracked_entities.insert(self.sector_id, entities);
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

fn garrison_troop_from_arrival(troop: &BaseTroop) -> GarrisonTroop {
    GarrisonTroop {
        troop_key_id: Some(troop.key),
        end_time: troop.end_time,
        ..Default::default()
    }
}

fn return_troop_from_collect_completion(completion: &CollectCompletion, now_ms: i64) -> BaseTroop {
    BaseTroop {
        key: completion.troop_key,
        r#type: completion.march_type,
        origin: Some(completion.target_pos),
        goal: Some(completion.origin_pos),
        status: Some(MARCH_STATUS_RETREAT),
        start_time: Some(now_ms),
        end_time: Some(now_ms + DEFAULT_COLLECT_RETURN_DURATION_MS),
        camp: completion.camp,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::xy_to_pos;
    use crate::march::{MARCH_STATUS_RETREAT, MARCH_TYPE_ATK_PLAYER, MARCH_TYPE_MINE_COLLECT};
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
        let tracked_entities = Arc::new(DashMap::new());
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
            tracked_entities,
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

    #[tokio::test]
    async fn collect_arrival_enters_collecting_state_and_publishes_started_event() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, _garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let target = xy_to_pos(101, 100);

        actor
            .handle_arrival(BaseTroop {
                key: 4,
                r#type: Some(MARCH_TYPE_MINE_COLLECT),
                origin: Some(xy_to_pos(1, 1)),
                goal: Some(target),
                end_time: Some(12_000),
                ..Default::default()
            })
            .await;

        let collect_state = actor.collect_state.get(4).unwrap();
        assert_eq!(collect_state.target_pos, target);
        assert_eq!(
            collect_state.phase,
            crate::collect::CollectPhase::Collecting
        );
        assert_eq!(
            sink.records(),
            vec![WorldOutboundEvent::CollectStarted {
                troop_key: 4,
                target_pos: target,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
                start_time_ms: 12_000,
            }]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn tick_drives_collect_arrival_state_change() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, _garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let target = xy_to_pos(20, 20);

        actor
            .accept_transfer(
                BaseTroop {
                    key: 5,
                    r#type: Some(MARCH_TYPE_MINE_COLLECT),
                    origin: Some(xy_to_pos(1, 1)),
                    goal: Some(target),
                    end_time: Some(100),
                    ..Default::default()
                },
                Some(7),
                None,
            )
            .await
            .unwrap();

        actor.tick().await;

        assert!(!actor.marching_troops.contains_key(&5));
        let collect_state = actor.collect_state.get(5).unwrap();
        assert_eq!(collect_state.target_pos, target);
        assert_eq!(collect_state.formation_id, Some(7));
        assert_eq!(
            collect_state.phase,
            crate::collect::CollectPhase::Collecting
        );
        assert_eq!(
            sink.records(),
            vec![WorldOutboundEvent::CollectStarted {
                troop_key: 5,
                target_pos: target,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
                start_time_ms: 100,
            }]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn tick_drives_collect_completion_return_and_result_event() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, _garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let origin = xy_to_pos(1, 1);
        let target = xy_to_pos(20, 20);

        actor
            .accept_transfer(
                BaseTroop {
                    key: 55,
                    r#type: Some(MARCH_TYPE_MINE_COLLECT),
                    origin: Some(origin),
                    goal: Some(target),
                    end_time: Some(100),
                    ..Default::default()
                },
                Some(7),
                None,
            )
            .await
            .unwrap();

        actor.tick().await;
        assert_eq!(
            actor.collect_state.get(55).map(|state| state.phase),
            Some(crate::collect::CollectPhase::Collecting)
        );

        for _ in 0..5 {
            actor.tick().await;
        }
        let return_troop = actor.marching_troops.get(&55).unwrap();
        assert_eq!(return_troop.origin, Some(target));
        assert_eq!(return_troop.goal, Some(origin));
        assert_eq!(return_troop.status, Some(MARCH_STATUS_RETREAT));
        assert_eq!(
            actor.collect_state.get(55).map(|state| state.phase),
            Some(crate::collect::CollectPhase::Returning)
        );

        actor.tick().await;
        assert!(!actor.marching_troops.contains_key(&55));
        assert!(actor.collect_state.get(55).is_none());
        assert_eq!(
            sink.records(),
            vec![
                WorldOutboundEvent::CollectStarted {
                    troop_key: 55,
                    target_pos: target,
                    march_type: Some(MARCH_TYPE_MINE_COLLECT),
                    start_time_ms: 100,
                },
                WorldOutboundEvent::CollectReturned {
                    troop_key: 55,
                    target_pos: target,
                    home_pos: origin,
                    march_type: Some(MARCH_TYPE_MINE_COLLECT),
                    formation_id: Some(7),
                    awards: vec![proto::slg::AwardPb {
                        r#type: crate::collect::AWARD_TYPE_LORD_RESOURCE,
                        id: crate::collect::RESOURCE_ID_MEAT,
                        count: crate::collect::DEFAULT_COLLECT_RESOURCE_AMOUNT,
                        safe: Some(true),
                        ..Default::default()
                    }],
                    collect_start_time_ms: 100,
                    collect_end_time_ms: 100 + crate::collect::DEFAULT_COLLECT_DURATION_MS,
                },
            ]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn tick_drives_garrison_arrival_state_change() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let target = xy_to_pos(21, 21);

        actor
            .accept_transfer(
                BaseTroop {
                    key: 6,
                    r#type: Some(crate::march::MARCH_TYPE_GARRISON_CITY),
                    origin: Some(xy_to_pos(1, 1)),
                    goal: Some(target),
                    end_time: Some(100),
                    camp: Some(4),
                    ..Default::default()
                },
                None,
                None,
            )
            .await
            .unwrap();

        actor.tick().await;

        assert!(!actor.marching_troops.contains_key(&6));
        let garrisons = garrison_state.list(Some(target));
        assert_eq!(garrisons.len(), 1);
        assert_eq!(garrisons[0].troop_key_id, Some(6));
        assert_eq!(garrisons[0].end_time, Some(100));
        assert_eq!(
            sink.records(),
            vec![WorldOutboundEvent::GarrisonChanged {
                troop_key: 6,
                target_pos: target,
                camp: Some(4),
                is_arrival: true,
            }]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn recovery_restores_arrived_garrison_without_republishing() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let target = xy_to_pos(22, 22);

        let arrived = BaseTroop {
            key: 7,
            r#type: Some(crate::march::MARCH_TYPE_GARRISON_CITY),
            origin: Some(xy_to_pos(1, 1)),
            goal: Some(target),
            status: Some(MARCH_STATUS_ARRIVAL),
            end_time: Some(200),
            ..Default::default()
        };
        actor
            .wal
            .append(&WalEntry::TroopArrived {
                troop: WalTroop::from(&arrived),
            })
            .await
            .unwrap();

        actor.init_with_recovery().await.unwrap();

        let garrisons = garrison_state.list(Some(target));
        assert_eq!(garrisons.len(), 1);
        assert_eq!(garrisons[0].troop_key_id, Some(7));
        assert_eq!(garrisons[0].end_time, Some(200));
        assert!(sink.records().is_empty());

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn recovery_restores_collecting_timer_and_returns_with_reward() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, _garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let origin = xy_to_pos(1, 1);
        let target = xy_to_pos(23, 23);
        let arrived = BaseTroop {
            key: 8,
            r#type: Some(MARCH_TYPE_MINE_COLLECT),
            origin: Some(origin),
            goal: Some(target),
            status: Some(MARCH_STATUS_ARRIVAL),
            end_time: Some(100),
            camp: Some(4),
            ..Default::default()
        };
        let collect_state = CollectState {
            troop_key: 8,
            origin_pos: origin,
            target_pos: target,
            march_type: Some(MARCH_TYPE_MINE_COLLECT),
            formation_id: Some(7),
            start_time_ms: 100,
            collect_end_time_ms: 600,
            collected_amount: crate::collect::DEFAULT_COLLECT_RESOURCE_AMOUNT,
            resource_type: crate::collect::RESOURCE_ID_MEAT,
            camp: Some(4),
            target: None,
            phase: CollectPhase::Collecting,
        };
        actor
            .wal
            .append(&WalEntry::CollectTroopArrived {
                troop: WalTroop::from(&arrived),
                state: WalCollectState::from(&collect_state),
            })
            .await
            .unwrap();

        actor.init_with_recovery().await.unwrap();
        assert_eq!(
            actor.collect_state.get(8).map(|state| state.phase),
            Some(CollectPhase::Collecting)
        );
        assert_eq!(actor.troop_formations.get(&8), Some(&7));

        for _ in 0..6 {
            actor.tick().await;
        }
        assert_eq!(
            actor.collect_state.get(8).map(|state| state.phase),
            Some(CollectPhase::Returning)
        );
        assert_eq!(
            actor.marching_troops.get(&8).and_then(|troop| troop.status),
            Some(MARCH_STATUS_RETREAT)
        );

        actor.tick().await;
        assert!(actor.collect_state.get(8).is_none());
        assert_eq!(
            sink.records(),
            vec![WorldOutboundEvent::CollectReturned {
                troop_key: 8,
                target_pos: target,
                home_pos: origin,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
                formation_id: Some(7),
                awards: vec![proto::slg::AwardPb {
                    r#type: crate::collect::AWARD_TYPE_LORD_RESOURCE,
                    id: crate::collect::RESOURCE_ID_MEAT,
                    count: crate::collect::DEFAULT_COLLECT_RESOURCE_AMOUNT,
                    safe: Some(true),
                    ..Default::default()
                }],
                collect_start_time_ms: 100,
                collect_end_time_ms: 600,
            }]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }

    #[tokio::test]
    async fn recovery_restores_collect_return_troop_with_state() {
        let sink = Arc::new(InMemoryOutboundSink::new());
        let (mut actor, _garrison_state, wal_path) = actor_with_sink(sink.clone()).await;
        let origin = xy_to_pos(1, 1);
        let target = xy_to_pos(24, 24);
        let returning_troop = BaseTroop {
            key: 9,
            r#type: Some(MARCH_TYPE_MINE_COLLECT),
            origin: Some(target),
            goal: Some(origin),
            status: Some(MARCH_STATUS_RETREAT),
            start_time: Some(600),
            end_time: Some(700),
            camp: Some(4),
            ..Default::default()
        };
        let collect_state = CollectState {
            troop_key: 9,
            origin_pos: origin,
            target_pos: target,
            march_type: Some(MARCH_TYPE_MINE_COLLECT),
            formation_id: Some(7),
            start_time_ms: 100,
            collect_end_time_ms: 600,
            collected_amount: crate::collect::DEFAULT_COLLECT_RESOURCE_AMOUNT,
            resource_type: crate::collect::RESOURCE_ID_MEAT,
            camp: Some(4),
            target: None,
            phase: CollectPhase::Returning,
        };
        actor
            .wal
            .append(&WalEntry::CollectTroopUpdated {
                troop: WalTroop::from(&returning_troop),
                state: WalCollectState::from(&collect_state),
            })
            .await
            .unwrap();

        actor.init_with_recovery().await.unwrap();
        for _ in 0..7 {
            actor.tick().await;
        }

        assert!(actor.collect_state.get(9).is_none());
        assert_eq!(
            sink.records(),
            vec![WorldOutboundEvent::CollectReturned {
                troop_key: 9,
                target_pos: target,
                home_pos: origin,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
                formation_id: Some(7),
                awards: vec![proto::slg::AwardPb {
                    r#type: crate::collect::AWARD_TYPE_LORD_RESOURCE,
                    id: crate::collect::RESOURCE_ID_MEAT,
                    count: crate::collect::DEFAULT_COLLECT_RESOURCE_AMOUNT,
                    safe: Some(true),
                    ..Default::default()
                }],
                collect_start_time_ms: 100,
                collect_end_time_ms: 600,
            }]
        );

        let _ = tokio::fs::remove_file(wal_path).await;
    }
}
