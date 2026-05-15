use std::collections::HashMap;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use proto::slg::BaseEntity;

pub const MAP_WIDTH: i32 = 1300;
pub const MAP_HEIGHT: i32 = 1300;
pub const GRID_SIZE: i32 = 50; // 50x50 一个格子
pub const SECTOR_COUNT_X: i32 = 4;
pub const SECTOR_COUNT_Y: i32 = 4;
pub const SECTOR_WIDTH: i32 = MAP_WIDTH / SECTOR_COUNT_X; // 325
pub const SECTOR_HEIGHT: i32 = MAP_HEIGHT / SECTOR_COUNT_Y; // 325

/// 坐标转换工具
pub fn pos_to_xy(pos: i32) -> (i32, i32) {
    (pos % MAP_WIDTH, pos / MAP_WIDTH)
}

pub fn xy_to_pos(x: i32, y: i32) -> i32 {
    y * MAP_WIDTH + x
}

pub fn is_valid_xy(x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < MAP_WIDTH && y < MAP_HEIGHT
}

pub fn is_valid_pos(pos: i32) -> bool {
    if pos < 0 {
        return false;
    }
    let (x, y) = pos_to_xy(pos);
    is_valid_xy(x, y)
}

/// 计算坐标所在的 Grid 索引
pub fn pos_to_grid(pos: i32) -> i32 {
    let (x, y) = pos_to_xy(pos);
    let gx = x / GRID_SIZE;
    let gy = y / GRID_SIZE;
    // 假设横向有 1300/50 = 26 个 Grid
    gy * (MAP_WIDTH / GRID_SIZE) + gx
}

/// 计算坐标所在的 Sector 索引
pub fn pos_to_sector_id(pos: i32) -> i32 {
    let (x, y) = pos_to_xy(pos);
    let sx = (x / SECTOR_WIDTH).min(SECTOR_COUNT_X - 1);
    let sy = (y / SECTOR_HEIGHT).min(SECTOR_COUNT_Y - 1);
    sy * SECTOR_COUNT_X + sx
}

/// 地图网格存储
pub struct MapGrid {
    // GridID -> [Pos -> Entity]
    // 使用 DashMap 保证分区域并发安全
    pub sectors: DashMap<i32, HashMap<i32, BaseEntity>>,
}

impl MapGrid {
    pub fn new() -> Self {
        Self {
            sectors: DashMap::new(),
        }
    }

    /// 在指定位置添加实体
    pub fn add_entity(&self, entity: BaseEntity) {
        let gid = pos_to_grid(entity.pos);
        self.sectors.entry(gid).or_default().insert(entity.pos, entity);
    }

    pub fn upsert_entity(&self, entity: BaseEntity) -> Result<Option<BaseEntity>> {
        if !is_valid_pos(entity.pos) {
            return Err(anyhow!("invalid world position: {}", entity.pos));
        }

        let gid = pos_to_grid(entity.pos);
        Ok(self.sectors.entry(gid).or_default().insert(entity.pos, entity))
    }

    /// 移除指定位置的实体
    pub fn remove_entity(&self, pos: i32) -> Option<BaseEntity> {
        let gid = pos_to_grid(pos);
        if let Some(mut sector) = self.sectors.get_mut(&gid) {
            sector.remove(&pos)
        } else {
            None
        }
    }

    pub fn get_entity(&self, pos: i32) -> Option<BaseEntity> {
        if !is_valid_pos(pos) {
            return None;
        }

        let gid = pos_to_grid(pos);
        self.sectors
            .get(&gid)
            .and_then(|sector| sector.get(&pos).cloned())
    }

    pub fn move_entity(&self, from: i32, to: i32) -> Result<BaseEntity> {
        if !is_valid_pos(from) {
            return Err(anyhow!("invalid origin world position: {}", from));
        }
        if !is_valid_pos(to) {
            return Err(anyhow!("invalid target world position: {}", to));
        }

        let mut entity = self
            .remove_entity(from)
            .ok_or_else(|| anyhow!("entity not found at position {}", from))?;
        entity.pos = to;
        self.upsert_entity(entity.clone())?;
        Ok(entity)
    }

    pub fn search_by_type(&self, entity_type: i32) -> Vec<BaseEntity> {
        let mut entities: Vec<BaseEntity> = self
            .sectors
            .iter()
            .flat_map(|sector| {
                sector
                    .value()
                    .values()
                    .filter(move |entity| entity.entity_type == Some(entity_type))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();
        entities.sort_by_key(|entity| entity.pos);
        entities
    }

    /// 获取玩家视野内的所有 Grid ID (周围 9 宫格)
    pub fn get_view_grid_ids(pos: i32) -> Vec<i32> {
        let (x, y) = pos_to_xy(pos);
        let gx = x / GRID_SIZE;
        let gy = y / GRID_SIZE;
        let cols = MAP_WIDTH / GRID_SIZE;
        
        let mut gids = Vec::with_capacity(9);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let ngx = gx + dx;
                let ngy = gy + dy;
                if ngx >= 0 && ngx < cols && ngy >= 0 && ngy < (MAP_HEIGHT / GRID_SIZE) {
                    gids.push(ngy * cols + ngx);
                }
            }
        }
        gids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(pos: i32, entity_type: i32, key_id: i32) -> BaseEntity {
        BaseEntity {
            pos,
            entity_type: Some(entity_type),
            key_id: Some(key_id),
            ..Default::default()
        }
    }

    #[test]
    fn rejects_positions_outside_map_bounds() {
        let grid = MapGrid::new();

        assert!(grid.upsert_entity(entity(-1, 4, 1)).is_err());
        assert!(grid.upsert_entity(entity(MAP_WIDTH * MAP_HEIGHT, 4, 1)).is_err());
        assert!(!is_valid_xy(MAP_WIDTH, 0));
        assert!(!is_valid_xy(0, MAP_HEIGHT));
    }

    #[test]
    fn indexes_entities_by_position_and_entity_type() {
        let grid = MapGrid::new();
        let city = entity(xy_to_pos(10, 20), 4, 1001);
        let mine = entity(xy_to_pos(12, 20), 3, 2001);

        grid.upsert_entity(city.clone()).unwrap();
        grid.upsert_entity(mine.clone()).unwrap();

        assert_eq!(grid.get_entity(city.pos).as_ref(), Some(&city));
        assert_eq!(grid.get_entity(mine.pos).as_ref(), Some(&mine));

        let cities = grid.search_by_type(4);
        assert_eq!(cities, vec![city]);
    }

    #[test]
    fn moving_entity_updates_old_and_new_grid_indexes() {
        let grid = MapGrid::new();
        let from = xy_to_pos(49, 49);
        let to = xy_to_pos(50, 50);
        grid.upsert_entity(entity(from, 4, 1001)).unwrap();

        let moved = grid.move_entity(from, to).unwrap();

        assert_eq!(moved.pos, to);
        assert!(grid.get_entity(from).is_none());
        assert_eq!(grid.get_entity(to).map(|e| e.pos), Some(to));
    }
}
