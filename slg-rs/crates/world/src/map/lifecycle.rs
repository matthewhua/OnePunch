use super::grid::{is_valid_pos, xy_to_pos, MapGrid};
use anyhow::{anyhow, Result};
use proto::slg::BaseEntity;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntitySpawnArea {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySpawnRule {
    pub entity_type: i32,
    pub conf_id: i32,
    pub count: usize,
    pub positions: Vec<i32>,
    pub area: Option<EntitySpawnArea>,
    pub ttl_ms: Option<u64>,
}

#[derive(Debug, Default)]
pub struct EntityLifecycleManager {
    expires_at_ms: HashMap<i32, u64>,
}

impl EntitySpawnRule {
    pub fn at_positions(
        entity_type: i32,
        conf_id: i32,
        count: usize,
        positions: Vec<i32>,
        ttl_ms: Option<u64>,
    ) -> Self {
        Self {
            entity_type,
            conf_id,
            count,
            positions,
            area: None,
            ttl_ms,
        }
    }

    pub fn in_area(
        entity_type: i32,
        conf_id: i32,
        count: usize,
        area: EntitySpawnArea,
        ttl_ms: Option<u64>,
    ) -> Self {
        Self {
            entity_type,
            conf_id,
            count,
            positions: Vec::new(),
            area: Some(area),
            ttl_ms,
        }
    }

    fn candidate_positions(&self) -> Result<Vec<i32>> {
        let mut positions = self.positions.clone();
        if let Some(area) = self.area {
            if area.min_x > area.max_x || area.min_y > area.max_y {
                return Err(anyhow!("invalid spawn area: {:?}", area));
            }
            for y in area.min_y..=area.max_y {
                for x in area.min_x..=area.max_x {
                    positions.push(xy_to_pos(x, y));
                }
            }
        }

        positions.sort_unstable();
        positions.dedup();

        if let Some(pos) = positions.iter().find(|pos| !is_valid_pos(**pos)) {
            return Err(anyhow!("invalid spawn position: {}", pos));
        }

        Ok(positions)
    }
}

impl EntityLifecycleManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn_missing(
        &mut self,
        grid: &MapGrid,
        rule: &EntitySpawnRule,
        now_ms: u64,
    ) -> Result<Vec<BaseEntity>> {
        let existing = grid.search_entities(Some(rule.entity_type), Some(rule.conf_id));
        if existing.len() >= rule.count {
            return Ok(Vec::new());
        }

        let occupied_positions: HashSet<i32> = grid
            .all_entities()
            .into_iter()
            .map(|entity| entity.pos)
            .collect();
        let candidates = rule.candidate_positions()?;
        let missing = rule.count - existing.len();
        let mut spawned = Vec::with_capacity(missing);

        for pos in candidates {
            if occupied_positions.contains(&pos) {
                continue;
            }

            let entity = BaseEntity {
                pos,
                entity_type: Some(rule.entity_type),
                key_id: Some(deterministic_key_id(rule.entity_type, rule.conf_id, pos)),
                conf_id: Some(rule.conf_id),
                ..Default::default()
            };
            grid.upsert_entity(entity.clone())?;
            if let Some(ttl_ms) = rule.ttl_ms {
                self.expires_at_ms
                    .insert(pos, now_ms.saturating_add(ttl_ms));
            }
            spawned.push(entity);

            if spawned.len() == missing {
                break;
            }
        }

        Ok(spawned)
    }

    pub fn expire_at(&mut self, grid: &MapGrid, now_ms: u64) -> Vec<BaseEntity> {
        let mut expired_positions: Vec<i32> = self
            .expires_at_ms
            .iter()
            .filter_map(|(pos, expires_at)| (*expires_at <= now_ms).then_some(*pos))
            .collect();
        expired_positions.sort_unstable();

        let mut removed = Vec::with_capacity(expired_positions.len());
        for pos in expired_positions {
            self.expires_at_ms.remove(&pos);
            if let Some(entity) = grid.remove_entity(pos) {
                removed.push(entity);
            }
        }

        removed.sort_by_key(|entity| entity.pos);
        removed
    }

    pub fn refresh_at(
        &mut self,
        grid: &MapGrid,
        rules: &[EntitySpawnRule],
        now_ms: u64,
    ) -> Result<EntityRefreshReport> {
        let expired = self.expire_at(grid, now_ms);
        let mut spawned = Vec::new();
        for rule in rules {
            spawned.extend(self.spawn_missing(grid, rule, now_ms)?);
        }
        spawned.sort_by_key(|entity| entity.pos);

        Ok(EntityRefreshReport { expired, spawned })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EntityRefreshReport {
    pub expired: Vec<BaseEntity>,
    pub spawned: Vec<BaseEntity>,
}

fn deterministic_key_id(entity_type: i32, conf_id: i32, pos: i32) -> i32 {
    let mut hash = 0x811c_9dc5_u32;
    for value in [entity_type, conf_id, pos] {
        for byte in value.to_le_bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(0x0100_0193);
        }
    }
    let key = (hash & 0x7fff_ffff) as i32;
    if key == 0 {
        1
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::xy_to_pos;

    fn rule(ttl_ms: Option<u64>) -> EntitySpawnRule {
        EntitySpawnRule::at_positions(
            7,
            701,
            3,
            vec![xy_to_pos(4, 0), xy_to_pos(1, 0), xy_to_pos(2, 0)],
            ttl_ms,
        )
    }

    #[test]
    fn spawn_missing_fills_rule_count() {
        let grid = MapGrid::new();
        let mut lifecycle = EntityLifecycleManager::new();

        let spawned = lifecycle
            .spawn_missing(&grid, &rule(Some(1_000)), 100)
            .unwrap();

        assert_eq!(spawned.len(), 3);
        assert_eq!(
            grid.search_entities(Some(7), Some(701))
                .into_iter()
                .map(|entity| entity.pos)
                .collect::<Vec<_>>(),
            vec![xy_to_pos(1, 0), xy_to_pos(2, 0), xy_to_pos(4, 0)]
        );

        let second_spawn = lifecycle
            .spawn_missing(&grid, &rule(Some(1_000)), 200)
            .unwrap();
        assert!(second_spawn.is_empty());
    }

    #[test]
    fn expire_at_removes_due_entities_in_position_order() {
        let grid = MapGrid::new();
        let mut lifecycle = EntityLifecycleManager::new();
        lifecycle
            .spawn_missing(&grid, &rule(Some(500)), 1_000)
            .unwrap();

        assert!(lifecycle.expire_at(&grid, 1_499).is_empty());

        let expired = lifecycle.expire_at(&grid, 1_500);

        assert_eq!(
            expired.iter().map(|entity| entity.pos).collect::<Vec<_>>(),
            vec![xy_to_pos(1, 0), xy_to_pos(2, 0), xy_to_pos(4, 0)]
        );
        assert!(grid.search_entities(Some(7), Some(701)).is_empty());
    }

    #[test]
    fn refresh_at_expires_then_spawns_missing_by_type() {
        let grid = MapGrid::new();
        let mut lifecycle = EntityLifecycleManager::new();
        lifecycle.spawn_missing(&grid, &rule(Some(100)), 0).unwrap();

        let report = lifecycle
            .refresh_at(&grid, &[rule(Some(100))], 100)
            .unwrap();

        assert_eq!(report.expired.len(), 3);
        assert_eq!(report.spawned.len(), 3);
        assert_eq!(grid.search_entities(Some(7), Some(701)).len(), 3);
    }

    #[test]
    fn spawn_order_and_key_ids_are_deterministic() {
        let grid_a = MapGrid::new();
        let grid_b = MapGrid::new();
        let mut lifecycle_a = EntityLifecycleManager::new();
        let mut lifecycle_b = EntityLifecycleManager::new();
        let rule = EntitySpawnRule::in_area(
            8,
            801,
            4,
            EntitySpawnArea {
                min_x: 1,
                min_y: 1,
                max_x: 2,
                max_y: 2,
            },
            None,
        );

        let first = lifecycle_a.spawn_missing(&grid_a, &rule, 0).unwrap();
        let second = lifecycle_b.spawn_missing(&grid_b, &rule, 999).unwrap();

        let first_keys = first
            .iter()
            .map(|entity| (entity.pos, entity.key_id))
            .collect::<Vec<_>>();
        let second_keys = second
            .iter()
            .map(|entity| (entity.pos, entity.key_id))
            .collect::<Vec<_>>();

        assert_eq!(first_keys, second_keys);
        assert_eq!(
            first.iter().map(|entity| entity.pos).collect::<Vec<_>>(),
            vec![
                xy_to_pos(1, 1),
                xy_to_pos(2, 1),
                xy_to_pos(1, 2),
                xy_to_pos(2, 2),
            ]
        );
    }
}
