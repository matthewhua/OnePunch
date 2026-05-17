//! 世界地图静态配置
//!
//! 对应数据库表：
//! - `s_map`：地图配置
//! - `s_npc`：NPC 配置
//! - `s_mine`：资源矿配置
//! - `s_wall`：城墙配置
//! - `s_area`：区域配置

use sqlx::FromRow;
use std::collections::HashMap;

/// 地图配置（s_map）
#[derive(Debug, Clone, FromRow)]
pub struct StaticMap {
    pub id: i32,
    #[sqlx(rename = "type")]
    pub map_type: Option<String>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub name: Option<String>,
    #[sqlx(rename = "designSize")]
    pub design_size: Option<String>,
    #[sqlx(rename = "areaNum")]
    pub area_num: Option<i32>,
    #[sqlx(rename = "displayType")]
    pub display_type: Option<i32>,
    #[sqlx(rename = "tileSize")]
    pub tile_size: Option<String>,
    #[sqlx(rename = "displaySize")]
    pub display_size: Option<String>,
    #[sqlx(rename = "campNum")]
    pub camp_num: Option<i32>,
    #[sqlx(rename = "canMoveIn")]
    pub can_move_in: Option<i32>,
    #[sqlx(rename = "backgroundAssetId")]
    pub background_asset_id: i32,
    #[sqlx(rename = "dataFile")]
    pub data_file: Option<String>,
}

/// NPC 配置（s_npc）
#[derive(Debug, Clone, FromRow)]
pub struct StaticNpc {
    pub id: i32,
    #[sqlx(rename = "npcId")]
    pub npc_id: Option<i32>,
    pub queue: Option<String>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub quality: Option<i32>,
    pub asset: Option<String>,
    pub line: Option<i32>,
    #[sqlx(rename = "armType")]
    pub arm_type: Option<i32>,
    pub resource: Option<String>,
    #[sqlx(rename = "battleAnimation")]
    pub battle_animation: Option<String>,
}

/// 资源矿配置（s_mine）
#[derive(Debug, Clone, FromRow)]
pub struct StaticMine {
    #[sqlx(rename = "mineId")]
    pub mine_id: i32,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub asset: Option<String>,
    #[sqlx(rename = "mineType")]
    pub mine_type: i32,
    pub lv: i32,
    pub weight: i32,
    pub reward: Option<String>,
    pub speed: i32,
    pub banner: Option<String>,
    pub sound: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticMineReward {
    pub award_type: i32,
    pub resource_id: i32,
    pub amount: i64,
}

impl StaticMine {
    pub fn parsed_reward(&self) -> anyhow::Result<StaticMineReward> {
        let raw = self
            .reward
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("s_mine.mineId={} reward is empty", self.mine_id))?;

        let parts: Vec<i64> = serde_json::from_str(raw).map_err(|err| {
            anyhow::anyhow!(
                "s_mine.mineId={} reward parse failed: {}",
                self.mine_id,
                err
            )
        })?;
        if parts.len() != 3 {
            anyhow::bail!(
                "s_mine.mineId={} reward must be [type,id,count], got {} values",
                self.mine_id,
                parts.len()
            );
        }
        if parts[2] <= 0 {
            anyhow::bail!(
                "s_mine.mineId={} reward count must be positive, got {}",
                self.mine_id,
                parts[2]
            );
        }

        Ok(StaticMineReward {
            award_type: i32::try_from(parts[0]).map_err(|_| {
                anyhow::anyhow!(
                    "s_mine.mineId={} reward type is out of i32 range",
                    self.mine_id
                )
            })?,
            resource_id: i32::try_from(parts[1]).map_err(|_| {
                anyhow::anyhow!(
                    "s_mine.mineId={} reward id is out of i32 range",
                    self.mine_id
                )
            })?,
            amount: parts[2],
        })
    }

    pub fn collect_duration_ms_for_amount(&self, amount: i64) -> Option<i64> {
        if self.speed <= 0 || amount <= 0 {
            return None;
        }

        let numerator = i128::from(amount).checked_mul(3_600_000)?;
        let speed = i128::from(self.speed);
        let duration_ms = (numerator + speed - 1) / speed;
        i64::try_from(duration_ms).ok().filter(|value| *value > 0)
    }
}

/// 城墙配置（s_wall）
#[derive(Debug, Clone, FromRow)]
pub struct StaticWall {
    pub id: i32,
    pub level: Option<i32>,
    pub durability: Option<String>,
}

/// 世界地图系统完整配置
#[derive(Debug, Clone, Default)]
pub struct WorldConfig {
    /// id → StaticMap
    pub maps: HashMap<i32, StaticMap>,
    /// id → StaticNpc
    pub npcs: HashMap<i32, StaticNpc>,
    /// mineId → StaticMine
    pub mines: HashMap<i32, StaticMine>,
    /// id → StaticWall
    pub walls: HashMap<i32, StaticWall>,
    // ── 二级索引 ──
    /// mineType → Vec<mineId>
    pub mines_by_type_idx: HashMap<i32, Vec<i32>>,
}

impl WorldConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (map_rows, npc_rows, mine_rows, wall_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticMap>("SELECT * FROM s_map").fetch_all(pool),
            sqlx::query_as::<_, StaticNpc>("SELECT * FROM s_npc").fetch_all(pool),
            sqlx::query_as::<_, StaticMine>("SELECT * FROM s_mine").fetch_all(pool),
            sqlx::query_as::<_, StaticWall>("SELECT * FROM s_wall").fetch_all(pool),
        )?;

        let maps: HashMap<i32, StaticMap> = map_rows.into_iter().map(|r| (r.id, r)).collect();
        let npcs: HashMap<i32, StaticNpc> = npc_rows.into_iter().map(|r| (r.id, r)).collect();
        let mines: HashMap<i32, StaticMine> =
            mine_rows.into_iter().map(|r| (r.mine_id, r)).collect();
        let walls: HashMap<i32, StaticWall> = wall_rows.into_iter().map(|r| (r.id, r)).collect();

        let mut cfg = Self {
            maps,
            npcs,
            mines,
            walls,
            ..Default::default()
        };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (mid, m) in &self.mines {
            self.mines_by_type_idx
                .entry(m.mine_type)
                .or_default()
                .push(*mid);
        }
    }

    pub fn mine(&self, mine_id: i32) -> Option<&StaticMine> {
        self.mines.get(&mine_id)
    }
}
