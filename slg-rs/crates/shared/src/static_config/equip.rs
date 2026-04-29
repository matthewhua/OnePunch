//! 装备系统静态配置
//!
//! 对应数据库表：
//! - `s_equip`：装备基础属性
//! - `s_equip_lv`：装备强化等级

use std::collections::HashMap;
use sqlx::FromRow;

/// 装备基础属性（s_equip）
#[derive(Debug, Clone, FromRow)]
pub struct StaticEquip {
    #[sqlx(rename = "equipId")]
    pub equip_id: i32,
    pub name: String,
    #[sqlx(rename = "unitType")]
    pub unit_type: Option<i32>,
    #[sqlx(rename = "slotType")]
    pub slot_type: Option<i32>,
    pub quality: Option<i32>,
    #[sqlx(rename = "canProvideExp")]
    pub can_provide_exp: Option<String>,
    pub attr: Option<String>,
    pub dec: Option<String>,
    pub icon: Option<String>,
}

/// 装备强化等级（s_equip_lv）
#[derive(Debug, Clone, FromRow)]
pub struct StaticEquipLv {
    pub id: i32,
    #[sqlx(rename = "equipId")]
    pub equip_id: i32,
    pub level: String,
    #[sqlx(rename = "upNeedResource")]
    pub up_need_resource: Option<String>,
    pub attr: Option<String>,
}

/// 装备系统完整配置
#[derive(Debug, Clone, Default)]
pub struct EquipConfig {
    /// equipId → StaticEquip
    pub equips: HashMap<i32, StaticEquip>,
    /// id → StaticEquipLv
    pub equip_levels: Vec<StaticEquipLv>,
    // ── 二级索引 ──
    /// equipId → Vec<StaticEquipLv index>
    pub levels_by_equip_idx: HashMap<i32, Vec<usize>>,
}

impl EquipConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (equip_rows, level_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticEquip>("SELECT * FROM s_equip").fetch_all(pool),
            sqlx::query_as::<_, StaticEquipLv>("SELECT * FROM s_equip_lv").fetch_all(pool),
        )?;

        let equips: HashMap<i32, StaticEquip> = equip_rows
            .into_iter().map(|r| (r.equip_id, r)).collect();

        let mut cfg = Self { equips, equip_levels: level_rows, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (i, lv) in self.equip_levels.iter().enumerate() {
            self.levels_by_equip_idx.entry(lv.equip_id).or_default().push(i);
        }
    }
}
