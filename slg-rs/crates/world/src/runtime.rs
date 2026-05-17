use crate::message::SectorMessage;
use crate::outbound::{InMemoryOutboundSink, WorldOutboundSink};
use crate::sector_actor::MapSectorActor;
use crate::{health::HealthChecker, map::aoi::AoiManager, map::grid, wal::WriteAheadLog};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use proto::slg::{BaseEntity, BaseTroop};
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

const SECTOR_COUNT: i32 = grid::SECTOR_COUNT_X * grid::SECTOR_COUNT_Y;
static WAL_RUN_ID: AtomicU64 = AtomicU64::new(1);

pub struct WorldRuntime {
    sector_senders: HashMap<i32, mpsc::Sender<SectorMessage>>,
    shutdown_txs: Vec<broadcast::Sender<()>>,
    sector_troops: Arc<DashMap<i32, Vec<BaseTroop>>>,
    sector_entities: Arc<DashMap<i32, Vec<BaseEntity>>>,
    aoi: Arc<AoiManager>,
    garrison_state: Arc<crate::garrison::GarrisonState>,
    assembly_state: Arc<crate::assembly::AssemblyState>,
}

impl WorldRuntime {
    pub fn new() -> Self {
        Self::new_with_outbound(Arc::new(InMemoryOutboundSink::new()))
    }

    pub fn new_with_outbound(outbound_sink: Arc<dyn WorldOutboundSink>) -> Self {
        let base_time_ms = crate::march::now_millis();
        let aoi = Arc::new(AoiManager::new());
        let health = Arc::new(HealthChecker::new(std::time::Duration::from_secs(30)));
        let sector_troops = Arc::new(DashMap::new());
        let sector_entities = Arc::new(DashMap::new());
        let garrison_state = Arc::new(crate::garrison::GarrisonState::new());
        let assembly_state = Arc::new(crate::assembly::AssemblyState::new());
        let mut sector_senders = HashMap::new();
        let mut shutdown_txs = Vec::new();

        for sector_id in 0..SECTOR_COUNT {
            let (tx, rx) = mpsc::channel(64);
            let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
            let tracked_troops = sector_troops.clone();
            let tracked_entities = sector_entities.clone();
            let aoi = aoi.clone();
            let health = health.clone();
            let garrison_state = garrison_state.clone();
            let assembly_state = assembly_state.clone();
            let outbound_sink = outbound_sink.clone();
            let wal_path = sector_wal_path(sector_id);

            tokio::spawn(async move {
                match WriteAheadLog::new(&wal_path).await {
                    Ok(wal) => {
                        let actor = MapSectorActor::new_with_outbound(
                            sector_id,
                            rx,
                            base_time_ms,
                            aoi,
                            health,
                            garrison_state,
                            assembly_state,
                            outbound_sink,
                            wal,
                            shutdown_rx,
                            tracked_troops,
                            tracked_entities,
                        );
                        if let Err(err) = actor.run().await {
                            tracing::error!(sector_id, error = %err, "sector actor exited with error");
                        }
                    }
                    Err(err) => {
                        tracing::error!(sector_id, error = %err, "failed to open sector WAL");
                    }
                }
            });

            sector_senders.insert(sector_id, tx);
            shutdown_txs.push(shutdown_tx);
        }

        spawn_sector_tick_loop(sector_senders.values().cloned().collect());

        Self {
            sector_senders,
            shutdown_txs,
            sector_troops,
            sector_entities,
            aoi,
            garrison_state,
            assembly_state,
        }
    }

    pub fn sector_id_for_pos(pos: i32) -> i32 {
        grid::pos_to_sector_id(pos)
    }

    pub fn garrison_state(&self) -> Arc<crate::garrison::GarrisonState> {
        self.garrison_state.clone()
    }

    pub fn assembly_state(&self) -> Arc<crate::assembly::AssemblyState> {
        self.assembly_state.clone()
    }

    pub fn aoi_manager(&self) -> Arc<AoiManager> {
        self.aoi.clone()
    }

    pub async fn send_transfer_troop(&self, troop: BaseTroop) -> Result<()> {
        self.send_transfer_troop_with_formation(troop, None).await
    }

    pub async fn send_transfer_troop_with_formation(
        &self,
        troop: BaseTroop,
        formation_id: Option<i32>,
    ) -> Result<()> {
        let sector_id = troop
            .goal
            .ok_or_else(|| anyhow!("troop goal is required for sector routing"))?;
        let sector_id = Self::sector_id_for_pos(sector_id);
        let sender = self
            .sector_senders
            .get(&sector_id)
            .ok_or_else(|| anyhow!("sector sender {} not found", sector_id))?;
        sender
            .send(SectorMessage::TransferTroop {
                troop_data: troop,
                formation_id,
                collect_state: None,
            })
            .await
            .map_err(|_| anyhow!("sector {} receiver dropped", sector_id))
    }

    pub async fn sync_troop_update(&self, troop: BaseTroop) -> Result<()> {
        let troop_key = troop.key;
        let target_sectors = troop_sector_ids(&troop)?;
        let existing_sectors = self.sectors_tracking_troop(troop_key);

        for sector_id in existing_sectors.difference(&target_sectors) {
            self.send_to_sector(*sector_id, SectorMessage::RemoveTroop { troop_key })
                .await?;
        }

        for sector_id in target_sectors {
            self.send_to_sector(
                sector_id,
                SectorMessage::UpdateTroop {
                    troop_data: troop.clone(),
                },
            )
            .await?;
        }

        Ok(())
    }

    pub async fn sync_entity_upsert(&self, entity: BaseEntity) -> Result<()> {
        if !grid::is_valid_pos(entity.pos) {
            return Err(anyhow!("invalid world entity position: {}", entity.pos));
        }

        let sector_id = Self::sector_id_for_pos(entity.pos);
        self.send_to_sector(
            sector_id,
            SectorMessage::UpsertEntity {
                entity_data: entity,
            },
        )
        .await
    }

    pub fn sync_entity_upsert_now(&self, entity: BaseEntity) -> Result<()> {
        if !grid::is_valid_pos(entity.pos) {
            return Err(anyhow!("invalid world entity position: {}", entity.pos));
        }

        let sector_id = Self::sector_id_for_pos(entity.pos);
        self.try_send_to_sector(
            sector_id,
            SectorMessage::UpsertEntity {
                entity_data: entity,
            },
        )
    }

    pub async fn sync_entity_remove(&self, pos: i32) -> Result<()> {
        if !grid::is_valid_pos(pos) {
            return Err(anyhow!("invalid world entity position: {}", pos));
        }

        let sector_id = Self::sector_id_for_pos(pos);
        self.send_to_sector(sector_id, SectorMessage::RemoveEntity { pos })
            .await
    }

    pub fn sync_entity_remove_now(&self, pos: i32) -> Result<()> {
        if !grid::is_valid_pos(pos) {
            return Err(anyhow!("invalid world entity position: {}", pos));
        }

        let sector_id = Self::sector_id_for_pos(pos);
        self.try_send_to_sector(sector_id, SectorMessage::RemoveEntity { pos })
    }

    pub async fn sync_entity_snapshot(&self, entities: Vec<BaseEntity>) -> Result<()> {
        for entity in entities {
            self.sync_entity_upsert(entity).await?;
        }
        Ok(())
    }

    pub fn sync_entity_snapshot_now(&self, entities: Vec<BaseEntity>) -> Result<()> {
        for entity in entities {
            self.sync_entity_upsert_now(entity)?;
        }
        Ok(())
    }

    async fn send_to_sector(&self, sector_id: i32, msg: SectorMessage) -> Result<()> {
        let sender = self
            .sector_senders
            .get(&sector_id)
            .ok_or_else(|| anyhow!("sector sender {} not found", sector_id))?;
        sender
            .send(msg)
            .await
            .map_err(|_| anyhow!("sector {} receiver dropped", sector_id))
    }

    fn try_send_to_sector(&self, sector_id: i32, msg: SectorMessage) -> Result<()> {
        let sender = self
            .sector_senders
            .get(&sector_id)
            .ok_or_else(|| anyhow!("sector sender {} not found", sector_id))?;
        match sender.try_send(msg) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(msg)) => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let _ = sender.send(msg).await;
                });
                Ok(())
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(anyhow!("sector {} receiver dropped", sector_id))
            }
        }
    }

    fn sectors_tracking_troop(&self, troop_key: i32) -> BTreeSet<i32> {
        self.sector_troops
            .iter()
            .filter_map(|entry| {
                entry
                    .value()
                    .iter()
                    .any(|troop| troop.key == troop_key)
                    .then_some(*entry.key())
            })
            .collect()
    }

    pub fn sector_troop_count(&self, sector_id: i32) -> usize {
        self.sector_troops
            .get(&sector_id)
            .map(|troops| troops.len())
            .unwrap_or(0)
    }

    pub fn sector_troop_keys(&self, sector_id: i32) -> Vec<i32> {
        self.sector_troops
            .get(&sector_id)
            .map(|troops| troops.iter().map(|troop| troop.key).collect())
            .unwrap_or_default()
    }

    pub fn sector_entity_positions(&self, sector_id: i32) -> Vec<i32> {
        self.sector_entities
            .get(&sector_id)
            .map(|entities| entities.iter().map(|entity| entity.pos).collect())
            .unwrap_or_default()
    }
}

fn spawn_sector_tick_loop(sector_senders: Vec<mpsc::Sender<SectorMessage>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            for sender in &sector_senders {
                let _ = sender.send(SectorMessage::Tick).await;
            }
        }
    });
}

fn troop_sector_ids(troop: &BaseTroop) -> Result<BTreeSet<i32>> {
    let mut sector_ids = BTreeSet::new();
    if let Some(origin) = troop.origin {
        sector_ids.insert(WorldRuntime::sector_id_for_pos(origin));
    }
    if let Some(goal) = troop.goal {
        sector_ids.insert(WorldRuntime::sector_id_for_pos(goal));
    }
    if sector_ids.is_empty() {
        return Err(anyhow!(
            "troop {} requires origin or goal for sector routing",
            troop.key
        ));
    }
    Ok(sector_ids)
}

fn sector_wal_path(sector_id: i32) -> PathBuf {
    let run_id = WAL_RUN_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "slg-rs-world-sector-{}-{}-{}.wal",
        std::process::id(),
        run_id,
        sector_id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::xy_to_pos;
    use std::time::Duration;

    async fn wait_for_keys(runtime: &WorldRuntime, sector_id: i32, expected: Vec<i32>) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
        loop {
            if runtime.sector_troop_keys(sector_id) == expected {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "sector {} keys did not reach {:?}",
                sector_id,
                expected
            );
            tokio::task::yield_now().await;
        }
    }

    async fn wait_for_entity_positions(runtime: &WorldRuntime, sector_id: i32, expected: Vec<i32>) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
        loop {
            if runtime.sector_entity_positions(sector_id) == expected {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "sector {} entity positions did not reach {:?}",
                sector_id,
                expected
            );
            tokio::task::yield_now().await;
        }
    }

    fn troop(key: i32, origin: i32, goal: i32, end_time: i64) -> BaseTroop {
        BaseTroop {
            key,
            origin: Some(origin),
            goal: Some(goal),
            status: Some(crate::march::MARCH_STATUS_MARCH),
            start_time: Some(crate::march::now_millis()),
            end_time: Some(end_time),
            ..Default::default()
        }
    }

    fn entity(pos: i32, entity_type: i32, key_id: i32) -> BaseEntity {
        BaseEntity {
            pos,
            entity_type: Some(entity_type),
            key_id: Some(key_id),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn sync_troop_update_moves_tracked_view_between_sectors() {
        let runtime = WorldRuntime::new();
        let original_origin = xy_to_pos(0, 0);
        let original_goal = xy_to_pos(1000, 1000);
        let current_pos = xy_to_pos(500, 500);
        let home = original_origin;
        let now = crate::march::now_millis();

        runtime
            .send_transfer_troop(troop(11, original_origin, original_goal, now + 10_000))
            .await
            .unwrap();

        let old_sector = WorldRuntime::sector_id_for_pos(original_goal);
        wait_for_keys(&runtime, old_sector, vec![11]).await;

        let updated = BaseTroop {
            status: Some(crate::march::MARCH_STATUS_RETREAT),
            ..troop(11, current_pos, home, now + 20_000)
        };
        runtime.sync_troop_update(updated).await.unwrap();

        let current_sector = WorldRuntime::sector_id_for_pos(current_pos);
        let home_sector = WorldRuntime::sector_id_for_pos(home);
        wait_for_keys(&runtime, old_sector, vec![]).await;
        wait_for_keys(&runtime, current_sector, vec![11]).await;
        wait_for_keys(&runtime, home_sector, vec![11]).await;
    }

    #[tokio::test]
    async fn runtime_tick_loop_drives_sector_arrival() {
        let runtime = WorldRuntime::new();
        let origin = xy_to_pos(0, 0);
        let goal = xy_to_pos(10, 10);
        let sector_id = WorldRuntime::sector_id_for_pos(goal);

        runtime
            .send_transfer_troop(troop(12, origin, goal, crate::march::now_millis() + 150))
            .await
            .unwrap();

        wait_for_keys(&runtime, sector_id, vec![12]).await;
        wait_for_keys(&runtime, sector_id, vec![]).await;
    }

    #[tokio::test]
    async fn sync_entity_snapshot_routes_entities_to_owning_sectors() {
        let runtime = WorldRuntime::new();
        let first_pos = xy_to_pos(10, 10);
        let second_pos = xy_to_pos(400, 400);
        let first_sector = WorldRuntime::sector_id_for_pos(first_pos);
        let second_sector = WorldRuntime::sector_id_for_pos(second_pos);

        runtime
            .sync_entity_snapshot(vec![entity(second_pos, 3, 2), entity(first_pos, 4, 1)])
            .await
            .unwrap();

        wait_for_entity_positions(&runtime, first_sector, vec![first_pos]).await;
        wait_for_entity_positions(&runtime, second_sector, vec![second_pos]).await;

        runtime.sync_entity_remove(second_pos).await.unwrap();
        wait_for_entity_positions(&runtime, second_sector, vec![]).await;
    }
}
