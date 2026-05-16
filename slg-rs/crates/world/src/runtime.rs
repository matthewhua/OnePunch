use crate::message::SectorMessage;
use crate::sector_actor::MapSectorActor;
use crate::{health::HealthChecker, map::aoi::AoiManager, map::grid, wal::WriteAheadLog};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use proto::slg::BaseTroop;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
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

fn sector_wal_path(sector_id: i32) -> PathBuf {
    let run_id = WAL_RUN_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "slg-rs-world-sector-{}-{}-{}.wal",
        std::process::id(),
        run_id,
        sector_id
    ))
}
