use std::collections::HashMap;
use dashmap::DashMap;
use proto::slg::BaseEntity;

pub const MAP_WIDTH: i32 = 1300;
pub const MAP_HEIGHT: i32 = 1300;
pub const GRID_SIZE: i32 = 50; // 50x50 一个格子

/// 坐标转换工具
pub fn pos_to_xy(pos: i32) -> (i32, i32) {
    (pos % MAP_WIDTH, pos / MAP_WIDTH)
}

pub fn xy_to_pos(x: i32, y: i32) -> i32 {
    y * MAP_WIDTH + x
}

/// 计算坐标所在的 Grid 索引
pub fn pos_to_grid(pos: i32) -> i32 {
    let (x, y) = pos_to_xy(pos);
    let gx = x / GRID_SIZE;
    let gy = y / GRID_SIZE;
    // 假设横向有 1300/50 = 26 个 Grid
    gy * (MAP_WIDTH / GRID_SIZE) + gx
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

    /// 移除指定位置的实体
    pub fn remove_entity(&self, pos: i32) -> Option<BaseEntity> {
        let gid = pos_to_grid(pos);
        if let Some(mut sector) = self.sectors.get_mut(&gid) {
            sector.remove(&pos)
        } else {
            None
        }
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
