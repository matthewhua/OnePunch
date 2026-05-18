//! Minimal local rank snapshot support for Step 15.
//!
//! The full design calls for a dedicated RankActor backed by `p_global.rank_data`.
//! This module keeps the first slice deliberately small and deterministic: it owns an
//! in-memory snapshot, can ingest score updates, and can answer `1193 GetRankRq`
//! from Home registry without touching World/Battle semantics.

use anyhow::Result;
use prost::Message;
use proto::slg::{GetRankRq, GetRankRs, RankItem};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const DEFAULT_PAGE_SIZE: usize = 20;
const MAX_PAGE_SIZE: usize = 20;
const DEFAULT_VERSION: i64 = 0;

static LOCAL_RANKS: LazyLock<Mutex<RankSnapshot>> = LazyLock::new(|| Mutex::new(RankSnapshot::new()));

#[derive(Debug, Clone, Default)]
pub struct RankSnapshot {
    values: HashMap<(i32, i32), HashMap<i64, RankEntry>>,
    versions: HashMap<(i32, i32), i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankEntry {
    pub role_id: i64,
    pub rank_value: i64,
    pub update_time: i64,
    pub nick: Option<String>,
    pub portrait: Option<i32>,
    pub portrait_frame: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankPage {
    pub rank_type: i32,
    pub page: i32,
    pub scope: i32,
    pub items: Vec<RankItem>,
    pub my_rank: Option<RankItem>,
    pub total_page: i32,
    pub version: i64,
}

pub struct RankSystem;

impl RankSystem {
    pub fn query_for_role(role_id: i64, payload: &[u8]) -> Result<Vec<u8>> {
        let rq = GetRankRq::decode(payload)?;
        validate_rank_request(&rq)?;
        let snapshot = LOCAL_RANKS
            .lock()
            .map_err(|_| anyhow::anyhow!("local rank lock poisoned"))?;
        let page = snapshot.query(role_id, rq.r#type, rq.page, rq.scope);
        Ok(GetRankRs {
            r#type: Some(page.rank_type),
            page: Some(page.page),
            scope: Some(page.scope),
            rank_item: page.items,
            my_rank: page.my_rank,
            total_page: Some(page.total_page),
            version: Some(page.version),
        }
        .encode_to_vec())
    }

    pub fn update(entry_type: i32, scope: i32, entry: RankEntry) -> Result<()> {
        validate_rank_type_and_scope(entry_type, scope)?;
        if entry.role_id <= 0 {
            anyhow::bail!("invalid rank role_id={}", entry.role_id);
        }
        let mut snapshot = LOCAL_RANKS
            .lock()
            .map_err(|_| anyhow::anyhow!("local rank lock poisoned"))?;
        snapshot.upsert(entry_type, scope, entry);
        Ok(())
    }

    #[cfg(test)]
    fn reset_for_test() {
        *LOCAL_RANKS.lock().unwrap() = RankSnapshot::new();
    }
}

impl RankSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, rank_type: i32, scope: i32, entry: RankEntry) {
        let key = (rank_type, scope);
        self.values
            .entry(key)
            .or_default()
            .insert(entry.role_id, entry);
        *self.versions.entry(key).or_insert(DEFAULT_VERSION) += 1;
    }

    pub fn query(&self, role_id: i64, rank_type: i32, page: i32, scope: i32) -> RankPage {
        let key = (rank_type, scope);
        let mut rows: Vec<&RankEntry> = self
            .values
            .get(&key)
            .map(|rows| rows.values().collect())
            .unwrap_or_default();
        rows.sort_by(|left, right| {
            right
                .rank_value
                .cmp(&left.rank_value)
                .then_with(|| left.update_time.cmp(&right.update_time))
                .then_with(|| left.role_id.cmp(&right.role_id))
        });

        let total_page = total_pages(rows.len());
        let page = page.max(1);
        let start = ((page as usize).saturating_sub(1)).saturating_mul(DEFAULT_PAGE_SIZE);
        let end = start.saturating_add(MAX_PAGE_SIZE).min(rows.len());

        let items = if start < rows.len() {
            rows[start..end]
                .iter()
                .enumerate()
                .map(|(offset, entry)| entry.to_rank_item((start + offset + 1) as i32))
                .collect()
        } else {
            Vec::new()
        };

        let my_rank = rows
            .iter()
            .position(|entry| entry.role_id == role_id)
            .map(|index| rows[index].to_rank_item((index + 1) as i32));

        RankPage {
            rank_type,
            page,
            scope,
            items,
            my_rank,
            total_page,
            version: *self.versions.get(&key).unwrap_or(&DEFAULT_VERSION),
        }
    }
}

impl RankEntry {
    pub fn new(role_id: i64, rank_value: i64, update_time: i64) -> Self {
        Self {
            role_id,
            rank_value,
            update_time,
            nick: None,
            portrait: None,
            portrait_frame: None,
        }
    }

    fn to_rank_item(&self, rank: i32) -> RankItem {
        RankItem {
            rank: Some(rank),
            rank_value: Some(self.rank_value),
            update_time: Some(self.update_time),
            role_id: Some(self.role_id),
            nick: self.nick.clone(),
            portrait: self.portrait,
            portrait_frame: self.portrait_frame,
            ..Default::default()
        }
    }
}

fn validate_rank_request(rq: &GetRankRq) -> Result<()> {
    validate_rank_type_and_scope(rq.r#type, rq.scope)?;
    if rq.page <= 0 {
        anyhow::bail!("invalid rank page={}", rq.page);
    }
    Ok(())
}

fn validate_rank_type_and_scope(rank_type: i32, scope: i32) -> Result<()> {
    if rank_type <= 0 {
        anyhow::bail!("invalid rank type={}", rank_type);
    }
    if !(0..=1).contains(&scope) {
        anyhow::bail!("invalid rank scope={}", scope);
    }
    Ok(())
}

fn total_pages(total_items: usize) -> i32 {
    if total_items == 0 {
        0
    } else {
        total_items.div_ceil(DEFAULT_PAGE_SIZE) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(rank_type: i32, page: i32, scope: i32) -> Vec<u8> {
        GetRankRq {
            r#type: rank_type,
            page,
            scope,
        }
        .encode_to_vec()
    }

    #[test]
    fn snapshot_orders_by_value_then_time_then_role() {
        let mut snapshot = RankSnapshot::new();
        snapshot.upsert(1, 0, RankEntry::new(30, 100, 10));
        snapshot.upsert(1, 0, RankEntry::new(20, 200, 20));
        snapshot.upsert(1, 0, RankEntry::new(10, 200, 10));

        let page = snapshot.query(20, 1, 1, 0);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].role_id, Some(10));
        assert_eq!(page.items[1].role_id, Some(20));
        assert_eq!(page.items[2].role_id, Some(30));
        assert_eq!(page.my_rank.unwrap().rank, Some(2));
        assert_eq!(page.total_page, 1);
        assert_eq!(page.version, 3);
    }

    #[test]
    fn empty_snapshot_returns_empty_page() {
        let snapshot = RankSnapshot::new();
        let page = snapshot.query(99, 1, 1, 0);
        assert!(page.items.is_empty());
        assert!(page.my_rank.is_none());
        assert_eq!(page.total_page, 0);
        assert_eq!(page.version, 0);
    }

    #[test]
    fn query_for_role_encodes_get_rank_response() {
        RankSystem::reset_for_test();
        RankSystem::update(2, 0, RankEntry::new(42, 500, 1)).unwrap();

        let bytes = RankSystem::query_for_role(42, &payload(2, 1, 0)).unwrap();
        let rs = GetRankRs::decode(bytes.as_slice()).unwrap();
        assert_eq!(rs.r#type, Some(2));
        assert_eq!(rs.page, Some(1));
        assert_eq!(rs.scope, Some(0));
        assert_eq!(rs.rank_item.len(), 1);
        assert_eq!(rs.my_rank.unwrap().role_id, Some(42));
    }

    #[test]
    fn invalid_request_is_rejected() {
        assert!(RankSystem::query_for_role(42, &payload(0, 1, 0)).is_err());
        assert!(RankSystem::query_for_role(42, &payload(1, 0, 0)).is_err());
        assert!(RankSystem::query_for_role(42, &payload(1, 1, 9)).is_err());
    }
}
