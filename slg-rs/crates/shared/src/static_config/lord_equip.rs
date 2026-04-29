//! 领主装备静态配置
//!
//! 对应数据库表：
//! - `s_lord_equip`：领主装备
//! - `s_lord_equip_set`：领主装备套装

use std::collections::HashMap;
use sqlx::FromRow;

/// 领主装备（s_lord_equip）
#[derive(Debug, Clone, FromRow)]
pub struct StaticLordEquip {
    pub id: i32,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    pub equip_type: Option<i32>,
    pub lv: Option<i32>,
    pub quality: Option<i32>,
    pub class: Option<i32>,
    pub star: Option<i32>,
    #[sqlx(rename = "upCost")]
    pub up_cost: Option<String>,
    #[sqlx(rename = "heroType")]
    pub hero_type: Option<i32>,
    pub attr: Option<String>,
    pub icon: Option<String>,
    pub unlock: Option<String>,
    #[sqlx(rename = "combatPower")]
    pub combat_power: Option<i32>,
}

/// 领主装备套装（s_lord_equip_set）
#[derive(Debug, Clone, FromRow)]
pub struct StaticLordEquipSet {
    pub id: i32,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    #[sqlx(rename = "setType")]
    pub set_type: i32,
    pub lv: Option<i32>,
    pub attr: Option<String>,
    pub fight: Option<i32>,
}

/// 领主装备系统完整配置
#[derive(Debug, Clone, Default)]
pub struct LordEquipConfig {
    /// id → StaticLordEquip
    pub lord_equips: HashMap<i32, StaticLordEquip>,
    /// id → StaticLordEquipSet
    pub lord_equip_sets: HashMap<i32, StaticLordEquipSet>,
    // ── 二级索引 ──
    /// type → Vec<id>
    pub equips_by_type_idx: HashMap<i32, Vec<i32>>,
}

impl LordEquipConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (equip_rows, set_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticLordEquip>("SELECT * FROM s_lord_equip").fetch_all(pool),
            sqlx::query_as::<_, StaticLordEquipSet>("SELECT * FROM s_lord_equip_set").fetch_all(pool),
        )?;

        let lord_equips: HashMap<i32, StaticLordEquip> = equip_rows
            .into_iter().map(|r| (r.id, r)).collect();
        let lord_equip_sets: HashMap<i32, StaticLordEquipSet> = set_rows
            .into_iter().map(|r| (r.id, r)).collect();

        let mut cfg = Self { lord_equips, lord_equip_sets, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (id, e) in &self.lord_equips {
            if let Some(t) = e.equip_type {
                self.equips_by_type_idx.entry(t).or_default().push(*id);
            }
        }
    }
}
