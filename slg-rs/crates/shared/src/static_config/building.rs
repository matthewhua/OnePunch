//! 建筑系统静态配置
//!
//! 对应数据库表：
//! - `s_building`：建筑属性
//! - `s_sim_building_conf`：模拟经营建筑配置

use std::collections::HashMap;
use sqlx::FromRow;

/// 建筑属性（s_building）
#[derive(Debug, Clone, FromRow)]
pub struct StaticBuilding {
    #[sqlx(rename = "buildingSign")]
    pub building_sign: i32,
    #[sqlx(rename = "desc")]
    pub description: String,
    #[sqlx(rename = "mapId")]
    pub map_id: i32,
    #[sqlx(rename = "buildingSize")]
    pub building_size: String,
    #[sqlx(rename = "buildingType")]
    pub building_type: i32,
    #[sqlx(rename = "featureType")]
    pub feature_type: i32,
    #[sqlx(rename = "actSize")]
    pub act_size: String,
    #[sqlx(rename = "initLv")]
    pub init_lv: String,
    #[sqlx(rename = "canUp")]
    pub can_up: i32,
    #[sqlx(rename = "maxLv")]
    pub max_lv: String,
    #[sqlx(rename = "canDestroy")]
    pub can_destroy: i32,
    #[sqlx(rename = "canMove")]
    pub can_move: i32,
    #[sqlx(rename = "buttonDisplay")]
    pub button_display: Option<String>,
    #[sqlx(rename = "workPosition")]
    pub work_position: Option<String>,
    #[sqlx(rename = "initialPosition")]
    pub initial_position: Option<String>,
    #[sqlx(rename = "upPosition")]
    pub up_position: Option<String>,
    #[sqlx(rename = "modelPosition")]
    pub model_position: Option<String>,
}

/// 建筑系统完整配置
#[derive(Debug, Clone, Default)]
pub struct BuildingConfig {
    /// buildingSign → StaticBuilding
    pub buildings: HashMap<i32, StaticBuilding>,
    // ── 二级索引 ──
    /// buildingType → Vec<buildingSign>
    pub buildings_by_type_idx: HashMap<i32, Vec<i32>>,
}

impl BuildingConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let rows = sqlx::query_as::<_, StaticBuilding>("SELECT * FROM s_building")
            .fetch_all(pool).await?;

        let buildings: HashMap<i32, StaticBuilding> = rows
            .into_iter().map(|r| (r.building_sign, r)).collect();

        let mut cfg = Self { buildings, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (sign, b) in &self.buildings {
            self.buildings_by_type_idx.entry(b.building_type).or_default().push(*sign);
        }
    }
}
