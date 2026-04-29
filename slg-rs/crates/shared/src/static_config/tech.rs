//! 科技系统静态配置
//!
//! 对应数据库表：
//! - `s_tech_lv`：科技等级消耗

use std::collections::HashMap;
use sqlx::FromRow;

/// 科技等级（s_tech_lv）
#[derive(Debug, Clone, FromRow)]
pub struct StaticTechLv {
    pub id: i32,
    #[sqlx(rename = "techId")]
    pub tech_id: i32,
    #[sqlx(rename = "techType")]
    pub tech_type: i32,
    #[sqlx(rename = "techName")]
    pub tech_name: String,
    #[sqlx(rename = "techStage")]
    pub tech_stage: String,
    pub level: i32,
    #[sqlx(rename = "maxLv")]
    pub max_lv: String,
    pub cnt: i32,
    #[sqlx(rename = "upTime")]
    pub up_time: i32,
    #[sqlx(rename = "upNeedResource")]
    pub up_need_resource: Option<String>,
    #[sqlx(rename = "reputationRequire")]
    pub reputation_require: Option<String>,
    #[sqlx(rename = "needTech")]
    pub need_tech: Option<String>,
    #[sqlx(rename = "needBuilding")]
    pub need_building: Option<String>,
    #[sqlx(rename = "nextId")]
    pub next_id: Option<String>,
    #[sqlx(rename = "buffEffectId")]
    pub buff_effect_id: Option<String>,
    pub fight: Option<String>,
    pub icon: Option<String>,
    pub cond: Option<String>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
}

/// 科技系统完整配置
#[derive(Debug, Clone, Default)]
pub struct TechConfig {
    /// id → StaticTechLv
    pub tech_levels: HashMap<i32, StaticTechLv>,
    // ── 二级索引 ──
    /// techId → Vec<id>（同一科技的所有等级）
    pub levels_by_tech_idx: HashMap<i32, Vec<i32>>,
    /// techType → Vec<techId>
    pub techs_by_type_idx: HashMap<i32, Vec<i32>>,
}

impl TechConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let rows = sqlx::query_as::<_, StaticTechLv>("SELECT * FROM s_tech_lv")
            .fetch_all(pool).await?;

        let tech_levels: HashMap<i32, StaticTechLv> = rows
            .into_iter().map(|r| (r.id, r)).collect();

        let mut cfg = Self { tech_levels, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        let mut seen_techs = std::collections::HashSet::new();
        for (id, t) in &self.tech_levels {
            self.levels_by_tech_idx.entry(t.tech_id).or_default().push(*id);
            if seen_techs.insert(t.tech_id) {
                self.techs_by_type_idx.entry(t.tech_type).or_default().push(t.tech_id);
            }
        }
    }
}
