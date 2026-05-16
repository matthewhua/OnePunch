use crate::message::SectorMessage;
use crate::sector_actor::MapSectorActor;
use crate::{health::HealthChecker, map::aoi::AoiManager, map::grid, wal::WriteAheadLog};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use proto::slg::BaseTroop;
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
}

impl WorldRuntime {
    pub fn new() -> Self {
        let aoi = Arc::new(AoiManager::new());
        let health = Arc::new(HealthChecker::new(std::time::Duration::from_secs(30)));
        let sector_troops = Arc::new(DashMap::new());
        let mut sector_senders = HashMap::new();
        let mut shutdown_txs = Vec::new();

        for sector_id in 0..SECTOR_COUNT {
            let (tx, rx) = mpsc::channel(64);
            let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
            let tracked_troops = sector_troops.clone();
            let aoi = aoi.clone();
            let health = health.clone();
            let wal_path = sector_wal_path(sector_id);

            tokio::spawn(async move {
                match WriteAheadLog::new(&wal_path).await {
                    Ok(wal) => {
                        let actor = MapSectorActor::new(
                            sector_id,
                            rx,
                            0,
                            aoi,
                            health,
                            wal,
                            shutdown_rx,
                            tracked_troops,
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

        Self {
            sector_senders,
            shutdown_txs,
            sector_troops,
        }
    }

    pub fn sector_id_for_pos(pos: i32) -> i32 {
        grid::pos_to_sector_id(pos)
    }

    pub async fn send_transfer_troop(&self, troop: BaseTroop) -> Result<()> {
        let sector_id = troop
            .goal
            .ok_or_else(|| anyhow!("troop goal is required for sector routing"))?;
        let sector_id = Self::sector_id_for_pos(sector_id);
        let sender = self
            .sector_senders
            .get(&sector_id)
            .ok_or_else(|| anyhow!("sector sender {} not found", sector_id))?;
        sender
            .send(SectorMessage::TransferTroop { troop_data: troop })
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

    fn troop(key: i32, origin: i32, goal: i32, end_time: i64) -> BaseTroop {
        BaseTroop {
            key,
            origin: Some(origin),
            goal: Some(goal),
            status: Some(crate::march::MARCH_STATUS_MARCH),
            start_time: Some(1_000),
            end_time: Some(end_time),
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

        runtime
            .send_transfer_troop(troop(11, original_origin, original_goal, 10_000))
            .await
            .unwrap();

        let old_sector = WorldRuntime::sector_id_for_pos(original_goal);
        wait_for_keys(&runtime, old_sector, vec![11]).await;

        let updated = BaseTroop {
            status: Some(crate::march::MARCH_STATUS_RETREAT),
            ..troop(11, current_pos, home, 20_000)
        };
        runtime.sync_troop_update(updated).await.unwrap();

        let current_sector = WorldRuntime::sector_id_for_pos(current_pos);
        let home_sector = WorldRuntime::sector_id_for_pos(home);
        wait_for_keys(&runtime, old_sector, vec![]).await;
        wait_for_keys(&runtime, current_sector, vec![11]).await;
        wait_for_keys(&runtime, home_sector, vec![11]).await;
    }
}
