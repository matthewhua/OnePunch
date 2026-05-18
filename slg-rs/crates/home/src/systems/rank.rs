//! Minimal local rank snapshot support for Step 15.
//!
//! `p_global.rank_data` is designed to store `proto::slg::LocalRankData` directly.
//! A `LocalRankData` contains multiple `RankCollect` buckets, and every bucket is
//! keyed by rank type (`Common.RankTypeDefine`) with the on-list entries serialized
//! as protobuf `RankItem`. This keeps the wire/query shape and persistence shape
//! aligned while leaving a future dedicated RankActor free to own loading/saving.

use anyhow::Result;
use prost::Message;
use proto::slg::{GetRankRq, GetRankRs, LocalRankData, RankCollect, RankItem};
use std::sync::{LazyLock, Mutex};

const DEFAULT_PAGE_SIZE: usize = 20;
const MAX_PAGE_SIZE: usize = 20;
const DEFAULT_VERSION: i64 = 0;

static LOCAL_RANKS: LazyLock<Mutex<RankSnapshot>> = LazyLock::new(|| Mutex::new(RankSnapshot::new()));

#[derive(Debug, Clone, Default)]
pub struct RankSnapshot {
    data: LocalRankData,
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

    pub fn update(rank_type: i32, scope: i32, entry: RankEntry) -> Result<()> {
        validate_rank_type_and_scope(rank_type, scope)?;
        if entry.role_id <= 0 {
            anyhow::bail!("invalid rank role_id={}", entry.role_id);
        }
        let mut snapshot = LOCAL_RANKS
            .lock()
            .map_err(|_| anyhow::anyhow!("local rank lock poisoned"))?;
        snapshot.upsert(rank_type, scope, entry);
        Ok(())
    }

    /// Encode the current rank state in the same protobuf shape intended for
    /// `p_global.rank_data`.
    pub fn save_to_global_rank_data() -> Result<Vec<u8>> {
        let snapshot = LOCAL_RANKS
            .lock()
            .map_err(|_| anyhow::anyhow!("local rank lock poisoned"))?;
        snapshot.save_to_bin()
    }

    /// Load a protobuf `LocalRankData` blob, e.g. from `p_global.rank_data`.
    pub fn load_from_global_rank_data(data: &[u8]) -> Result<()> {
        let snapshot = RankSnapshot::load_from_bin(data)?;
        let mut current = LOCAL_RANKS
            .lock()
            .map_err(|_| anyhow::anyhow!("local rank lock poisoned"))?;
        *current = snapshot;
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

    pub fn from_proto(mut data: LocalRankData) -> Self {
        for collect in &mut data.rank_collect {
            normalize_rank_items(&mut collect.rank_item);
        }
        Self { data }
    }

    pub fn to_proto(&self) -> LocalRankData {
        self.data.clone()
    }

    pub fn load_from_bin(data: &[u8]) -> Result<Self> {
        Ok(Self::from_proto(LocalRankData::decode(data)?))
    }

    pub fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.data.encode_to_vec())
    }

    pub fn upsert(&mut self, rank_type: i32, scope: i32, entry: RankEntry) {
        let collect = self.ensure_collect(rank_type, scope);
        upsert_rank_item(&mut collect.rank_item, entry.to_unranked_item());
        normalize_rank_items(&mut collect.rank_item);
        self.data.unique_id = Some(self.data.unique_id.unwrap_or(DEFAULT_VERSION).saturating_add(1));
    }

    pub fn query(&self, role_id: i64, rank_type: i32, page: i32, scope: i32) -> RankPage {
        let rank_key = rank_key(rank_type, scope);
        let rows: &[RankItem] = self
            .data
            .rank_collect
            .iter()
            .find(|collect| collect.r#type == Some(rank_key))
            .map(|collect| collect.rank_item.as_slice())
            .unwrap_or(&[]);

        let total_page = total_pages(rows.len());
        let page = page.max(1);
        let start = ((page as usize).saturating_sub(1)).saturating_mul(DEFAULT_PAGE_SIZE);
        let end = start.saturating_add(MAX_PAGE_SIZE).min(rows.len());

        let items = if start < rows.len() {
            rows[start..end].to_vec()
        } else {
            Vec::new()
        };

        let my_rank = rows
            .iter()
            .find(|entry| entry.role_id == Some(role_id))
            .cloned();

        RankPage {
            rank_type,
            page,
            scope,
            items,
            my_rank,
            total_page,
            version: self.data.unique_id.unwrap_or(DEFAULT_VERSION),
        }
    }

    fn ensure_collect(&mut self, rank_type: i32, scope: i32) -> &mut RankCollect {
        let rank_key = rank_key(rank_type, scope);
        if let Some(index) = self
            .data
            .rank_collect
            .iter()
            .position(|collect| collect.r#type == Some(rank_key))
        {
            return &mut self.data.rank_collect[index];
        }

        self.data.rank_collect.push(RankCollect {
            r#type: Some(rank_key),
            rank_item: Vec::new(),
        });
        self.data
            .rank_collect
            .last_mut()
            .expect("rank collect just inserted")
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

    fn to_unranked_item(&self) -> RankItem {
        RankItem {
            rank: None,
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

/// Fold `scope` into the stored collect key while keeping global scope compatible
/// with the legacy `RankCollect.type == RankTypeDefine` format.
fn rank_key(rank_type: i32, scope: i32) -> i32 {
    if scope == 0 {
        rank_type
    } else {
        rank_type.saturating_mul(10).saturating_add(scope)
    }
}

fn upsert_rank_item(items: &mut Vec<RankItem>, item: RankItem) {
    if let Some(role_id) = item.role_id {
        if let Some(existing) = items
            .iter_mut()
            .find(|existing| existing.role_id == Some(role_id))
        {
            *existing = item;
            return;
        }
    }
    items.push(item);
}

fn normalize_rank_items(items: &mut Vec<RankItem>) {
    items.sort_by(|left, right| {
        rank_value(right)
            .cmp(&rank_value(left))
            .then_with(|| update_time(left).cmp(&update_time(right)))
            .then_with(|| role_id(left).cmp(&role_id(right)))
    });
    for (index, item) in items.iter_mut().enumerate() {
        item.rank = Some((index + 1) as i32);
    }
}

fn rank_value(item: &RankItem) -> i64 {
    item.rank_value.unwrap_or_default()
}

fn update_time(item: &RankItem) -> i64 {
    item.update_time.unwrap_or_default()
}

fn role_id(item: &RankItem) -> i64 {
    item.role_id.unwrap_or_default()
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
        assert_eq!(page.items[0].rank, Some(1));
        assert_eq!(page.items[1].role_id, Some(20));
        assert_eq!(page.items[2].role_id, Some(30));
        assert_eq!(page.my_rank.unwrap().rank, Some(2));
        assert_eq!(page.total_page, 1);
        assert_eq!(page.version, 3);
    }

    #[test]
    fn supports_multiple_rank_types_and_scope_buckets() {
        let mut snapshot = RankSnapshot::new();
        snapshot.upsert(1, 0, RankEntry::new(10, 100, 1));
        snapshot.upsert(2, 0, RankEntry::new(20, 200, 1));
        snapshot.upsert(1, 1, RankEntry::new(30, 300, 1));

        assert_eq!(snapshot.query(10, 1, 1, 0).items[0].role_id, Some(10));
        assert_eq!(snapshot.query(20, 2, 1, 0).items[0].role_id, Some(20));
        assert_eq!(snapshot.query(30, 1, 1, 1).items[0].role_id, Some(30));
        assert_eq!(snapshot.to_proto().rank_collect.len(), 3);
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
    fn global_rank_data_roundtrip_uses_local_rank_data_pb() {
        RankSystem::reset_for_test();
        RankSystem::update(7, 0, RankEntry::new(77, 700, 1)).unwrap();

        let bytes = RankSystem::save_to_global_rank_data().unwrap();
        let decoded = LocalRankData::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.rank_collect.len(), 1);
        assert_eq!(decoded.rank_collect[0].r#type, Some(7));
        assert_eq!(decoded.rank_collect[0].rank_item[0].role_id, Some(77));

        RankSystem::reset_for_test();
        RankSystem::load_from_global_rank_data(&bytes).unwrap();
        let rs = GetRankRs::decode(
            RankSystem::query_for_role(77, &payload(7, 1, 0))
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert_eq!(rs.my_rank.unwrap().rank_value, Some(700));
    }

    #[test]
    fn invalid_request_is_rejected() {
        assert!(RankSystem::query_for_role(42, &payload(0, 1, 0)).is_err());
        assert!(RankSystem::query_for_role(42, &payload(1, 0, 0)).is_err());
        assert!(RankSystem::query_for_role(42, &payload(1, 1, 9)).is_err());
    }
}
