use crate::collect::{CollectPhase, CollectState};
use anyhow::Result;
use proto::slg::{BaseEntity, BaseTroop};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WalTroop {
    pub key: i32,
    pub troop_type: Option<i32>,
    pub origin: Option<i32>,
    pub goal: Option<i32>,
    pub status: Option<i32>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub camp: Option<i32>,
}

impl From<&BaseTroop> for WalTroop {
    fn from(troop: &BaseTroop) -> Self {
        Self {
            key: troop.key,
            troop_type: troop.r#type,
            origin: troop.origin,
            goal: troop.goal,
            status: troop.status,
            start_time: troop.start_time,
            end_time: troop.end_time,
            camp: troop.camp,
        }
    }
}

impl From<WalTroop> for BaseTroop {
    fn from(troop: WalTroop) -> Self {
        Self {
            key: troop.key,
            r#type: troop.troop_type,
            origin: troop.origin,
            goal: troop.goal,
            status: troop.status,
            start_time: troop.start_time,
            end_time: troop.end_time,
            camp: troop.camp,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WalEntity {
    pub pos: i32,
    pub entity_type: Option<i32>,
    pub key_id: Option<i32>,
    pub camp: Option<i32>,
    pub conf_id: Option<i32>,
    pub protect_time: Option<i32>,
    pub occupied_area: Vec<i32>,
    pub is_battle: Option<bool>,
}

impl From<&BaseEntity> for WalEntity {
    fn from(entity: &BaseEntity) -> Self {
        Self {
            pos: entity.pos,
            entity_type: entity.entity_type,
            key_id: entity.key_id,
            camp: entity.camp,
            conf_id: entity.conf_id,
            protect_time: entity.protect_time,
            occupied_area: entity.occupied_area.clone(),
            is_battle: entity.is_battle,
        }
    }
}

impl From<WalEntity> for BaseEntity {
    fn from(entity: WalEntity) -> Self {
        Self {
            pos: entity.pos,
            entity_type: entity.entity_type,
            key_id: entity.key_id,
            camp: entity.camp,
            conf_id: entity.conf_id,
            protect_time: entity.protect_time,
            occupied_area: entity.occupied_area,
            is_battle: entity.is_battle,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WalCollectState {
    pub troop_key: i32,
    pub origin_pos: i32,
    pub target_pos: i32,
    pub march_type: Option<i32>,
    pub formation_id: Option<i32>,
    pub start_time_ms: i64,
    pub collect_end_time_ms: i64,
    pub collected_amount: i64,
    pub resource_type: i32,
    pub camp: Option<i32>,
    pub phase: i32,
}

impl From<&CollectState> for WalCollectState {
    fn from(state: &CollectState) -> Self {
        Self {
            troop_key: state.troop_key,
            origin_pos: state.origin_pos,
            target_pos: state.target_pos,
            march_type: state.march_type,
            formation_id: state.formation_id,
            start_time_ms: state.start_time_ms,
            collect_end_time_ms: state.collect_end_time_ms,
            collected_amount: state.collected_amount,
            resource_type: state.resource_type,
            camp: state.camp,
            phase: collect_phase_to_i32(state.phase),
        }
    }
}

impl From<WalCollectState> for CollectState {
    fn from(state: WalCollectState) -> Self {
        Self {
            troop_key: state.troop_key,
            origin_pos: state.origin_pos,
            target_pos: state.target_pos,
            march_type: state.march_type,
            formation_id: state.formation_id,
            start_time_ms: state.start_time_ms,
            collect_end_time_ms: state.collect_end_time_ms,
            collected_amount: state.collected_amount,
            resource_type: state.resource_type,
            camp: state.camp,
            target: None,
            phase: collect_phase_from_i32(state.phase),
        }
    }
}

fn collect_phase_to_i32(phase: CollectPhase) -> i32 {
    match phase {
        CollectPhase::Arrived => 0,
        CollectPhase::Collecting => 1,
        CollectPhase::Returning => 2,
        CollectPhase::Completed => 3,
    }
}

fn collect_phase_from_i32(phase: i32) -> CollectPhase {
    match phase {
        0 => CollectPhase::Arrived,
        1 => CollectPhase::Collecting,
        2 => CollectPhase::Returning,
        3 => CollectPhase::Completed,
        _ => CollectPhase::Collecting,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WalEntry {
    /// 部队开始行军
    MarchStart {
        key: i32,
        origin: i32,
        goal: i32,
        start_time: i64,
        end_time: i64,
    },
    /// 部队转移到另一个 Sector
    TroopTransfer { key: i32, target_sector: i32 },
    /// 部队状态更新（召回、加速等会改变 BaseTroop 的行军状态）
    TroopUpdated { troop: WalTroop },
    /// 部队到达
    TroopArrived { troop: WalTroop },
    /// 实体新增或更新
    EntityUpsert { entity: WalEntity },
    /// 实体删除
    EntityRemove { pos: i32 },
    /// 资源变动
    ResourceUpdate {
        role_id: i64,
        pos: i32,
        res_type: i32,
        amount: i64,
    },
    /// 检查点（表示之前的日志已经安全存盘到数据库，可以截断）
    Checkpoint { sequence: u64 },
    /// 采集到达并开始采集
    CollectTroopArrived {
        troop: WalTroop,
        state: WalCollectState,
    },
    /// 采集中的部队状态更新（例如采集完成后开始返回）
    CollectTroopUpdated {
        troop: WalTroop,
        state: WalCollectState,
    },
    /// 采集返回到家，移除采集状态
    CollectReturnFinished { troop: WalTroop, troop_key: i32 },
}

#[derive(Debug, Default, Clone)]
pub struct WalReplayState {
    pub troops: HashMap<i32, BaseTroop>,
    pub entities: HashMap<i32, BaseEntity>,
    pub collect_states: HashMap<i32, CollectState>,
}

impl WalReplayState {
    pub fn from_entries(entries: &[WalEntry]) -> Self {
        let mut state = Self::default();
        for entry in entries {
            state.apply(entry);
        }
        state
    }

    pub fn apply(&mut self, entry: &WalEntry) {
        match entry {
            WalEntry::MarchStart {
                key,
                origin,
                goal,
                start_time,
                end_time,
            } => {
                self.troops.insert(
                    *key,
                    BaseTroop {
                        key: *key,
                        origin: Some(*origin),
                        goal: Some(*goal),
                        status: Some(crate::march::MARCH_STATUS_MARCH),
                        start_time: Some(*start_time),
                        end_time: Some(*end_time),
                        ..Default::default()
                    },
                );
            }
            WalEntry::TroopUpdated { troop } | WalEntry::TroopArrived { troop } => {
                self.troops.insert(troop.key, troop.clone().into());
            }
            WalEntry::CollectTroopArrived { troop, state }
            | WalEntry::CollectTroopUpdated { troop, state } => {
                self.troops.insert(troop.key, troop.clone().into());
                self.collect_states
                    .insert(state.troop_key, state.clone().into());
            }
            WalEntry::CollectReturnFinished { troop, troop_key } => {
                self.troops.insert(troop.key, troop.clone().into());
                self.collect_states.remove(troop_key);
            }
            WalEntry::TroopTransfer { key, .. } => {
                self.troops.remove(key);
                self.collect_states.remove(key);
            }
            WalEntry::EntityUpsert { entity } => {
                self.entities.insert(entity.pos, entity.clone().into());
            }
            WalEntry::EntityRemove { pos } => {
                self.entities.remove(pos);
            }
            WalEntry::ResourceUpdate { .. } | WalEntry::Checkpoint { .. } => {}
        }
    }
}

pub struct WriteAheadLog {
    file: File,
    path: String,
    sequence: u64,
}

impl WriteAheadLog {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)
            .await?;

        Ok(Self {
            file,
            path: path_str,
            sequence: 0,
        })
    }

    /// 追加一条日志
    pub async fn append(&mut self, entry: &WalEntry) -> Result<u64> {
        self.sequence += 1;
        let data = bincode::serialize(entry)?;
        let len = data.len() as u32;

        // 写入长度 + 数据
        self.file.write_all(&len.to_le_bytes()).await?;
        self.file.write_all(&data).await?;

        // 强制刷盘 (根据性能要求可以调整为定期刷盘)
        self.file.flush().await?;

        Ok(self.sequence)
    }

    /// 从日志中恢复数据
    pub async fn recover(&mut self) -> Result<Vec<WalEntry>> {
        let mut entries = Vec::new();
        let mut file = File::open(&self.path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let mut cursor = 0;
        while cursor + 4 <= buffer.len() {
            let len = u32::from_le_bytes(buffer[cursor..cursor + 4].try_into()?) as usize;
            cursor += 4;

            if cursor + len > buffer.len() {
                error!("WAL corrupted: unexpected end of file");
                break;
            }

            let entry: WalEntry = bincode::deserialize(&buffer[cursor..cursor + len])?;
            entries.push(entry);
            cursor += len;
        }

        info!("WAL recovered {} entries from {}", entries.len(), self.path);
        Ok(entries)
    }

    /// 截断日志（存盘后执行）
    pub async fn truncate(&mut self) -> Result<()> {
        // 简单处理：重新打开文件并清空
        self.file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await?;
        self.sequence = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::march::{MARCH_STATUS_ARRIVAL, MARCH_STATUS_MARCH, MARCH_STATUS_RETREAT};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn troop(
        key: i32,
        status: i32,
        origin: i32,
        goal: i32,
        start_time: i64,
        end_time: i64,
    ) -> BaseTroop {
        BaseTroop {
            key,
            r#type: Some(2),
            origin: Some(origin),
            goal: Some(goal),
            status: Some(status),
            start_time: Some(start_time),
            end_time: Some(end_time),
            camp: Some(1),
        }
    }

    #[test]
    fn replay_keeps_latest_troop_after_recall_and_accelerate() {
        let entries = vec![
            WalEntry::MarchStart {
                key: 7,
                origin: 100,
                goal: 200,
                start_time: 1_000,
                end_time: 11_000,
            },
            WalEntry::TroopUpdated {
                troop: WalTroop::from(&troop(7, MARCH_STATUS_RETREAT, 150, 100, 6_000, 11_000)),
            },
            WalEntry::TroopUpdated {
                troop: WalTroop::from(&troop(7, MARCH_STATUS_RETREAT, 150, 100, 6_000, 8_500)),
            },
        ];

        let state = WalReplayState::from_entries(&entries);
        let restored = state.troops.get(&7).expect("troop should recover");

        assert_eq!(restored.status, Some(MARCH_STATUS_RETREAT));
        assert_eq!(restored.origin, Some(150));
        assert_eq!(restored.goal, Some(100));
        assert_eq!(restored.start_time, Some(6_000));
        assert_eq!(restored.end_time, Some(8_500));
    }

    #[test]
    fn replay_records_arrival_and_entity_changes() {
        let entity = BaseEntity {
            pos: 300,
            entity_type: Some(4),
            key_id: Some(99),
            camp: Some(2),
            occupied_area: vec![300, 301],
            is_battle: Some(true),
            ..Default::default()
        };
        let entries = vec![
            WalEntry::TroopUpdated {
                troop: WalTroop::from(&troop(8, MARCH_STATUS_MARCH, 100, 300, 1_000, 2_000)),
            },
            WalEntry::TroopArrived {
                troop: WalTroop::from(&troop(8, MARCH_STATUS_ARRIVAL, 100, 300, 1_000, 2_000)),
            },
            WalEntry::EntityUpsert {
                entity: WalEntity::from(&entity),
            },
            WalEntry::EntityRemove { pos: 300 },
        ];

        let state = WalReplayState::from_entries(&entries);
        let restored = state.troops.get(&8).expect("arrival should recover");

        assert_eq!(restored.status, Some(MARCH_STATUS_ARRIVAL));
        assert!(!state.entities.contains_key(&300));
    }

    #[test]
    fn replay_tracks_collect_state_with_troop_updates() {
        let collect_state = CollectState {
            troop_key: 10,
            origin_pos: 100,
            target_pos: 300,
            march_type: Some(crate::march::MARCH_TYPE_MINE_COLLECT),
            formation_id: Some(7),
            start_time_ms: 2_000,
            collect_end_time_ms: 2_500,
            collected_amount: 100,
            resource_type: crate::collect::RESOURCE_ID_MEAT,
            camp: Some(1),
            target: None,
            phase: CollectPhase::Collecting,
        };
        let mut returning_state = collect_state.clone();
        returning_state.phase = CollectPhase::Returning;

        let entries = vec![
            WalEntry::CollectTroopArrived {
                troop: WalTroop::from(&troop(10, MARCH_STATUS_ARRIVAL, 100, 300, 1_000, 2_000)),
                state: WalCollectState::from(&collect_state),
            },
            WalEntry::CollectTroopUpdated {
                troop: WalTroop::from(&troop(10, MARCH_STATUS_RETREAT, 300, 100, 2_500, 2_600)),
                state: WalCollectState::from(&returning_state),
            },
        ];

        let state = WalReplayState::from_entries(&entries);
        let restored_troop = state.troops.get(&10).expect("troop should recover");
        let restored_collect = state
            .collect_states
            .get(&10)
            .expect("collect state should recover");

        assert_eq!(restored_troop.status, Some(MARCH_STATUS_RETREAT));
        assert_eq!(restored_collect.phase, CollectPhase::Returning);
        assert_eq!(restored_collect.formation_id, Some(7));
        assert_eq!(restored_collect.collected_amount, 100);
    }

    #[tokio::test]
    async fn recover_round_trips_new_wal_entries() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("world-wal-test-{}-{}.log", std::process::id(), now));

        let mut wal = WriteAheadLog::new(&path).await.unwrap();
        wal.append(&WalEntry::TroopUpdated {
            troop: WalTroop::from(&troop(9, MARCH_STATUS_RETREAT, 120, 10, 5_000, 6_000)),
        })
        .await
        .unwrap();
        wal.append(&WalEntry::EntityUpsert {
            entity: WalEntity::from(&BaseEntity {
                pos: 10,
                entity_type: Some(1),
                key_id: Some(2),
                ..Default::default()
            }),
        })
        .await
        .unwrap();

        let entries = wal.recover().await.unwrap();
        let state = WalReplayState::from_entries(&entries);

        assert_eq!(state.troops.get(&9).unwrap().end_time, Some(6_000));
        assert_eq!(state.entities.get(&10).unwrap().key_id, Some(2));

        let _ = tokio::fs::remove_file(path).await;
    }
}
