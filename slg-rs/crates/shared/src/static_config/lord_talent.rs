//! 领主天赋静态配置
//!
//! 对应数据库表：
//! - `s_lord_talent`：领主天赋
//! - `s_lord_talent_stage`：领主天赋阶段

use std::collections::HashMap;
use sqlx::FromRow;

/// 领主天赋（s_lord_talent）
#[derive(Debug, Clone, FromRow)]
pub struct StaticLordTalent {
    pub id: i32,
    #[sqlx(rename = "talentId")]
    pub talent_id: i32,
    #[sqlx(rename = "talentType")]
    pub talent_type: i32,
    #[sqlx(rename = "buffType")]
    pub buff_type: Option<i32>,
    #[sqlx(rename = "talentName")]
    pub talent_name: String,
    #[sqlx(rename = "talentGroup")]
    pub talent_group: i32,
    pub quality: Option<i32>,
    pub level: i32,
    #[sqlx(rename = "maxLv")]
    pub max_lv: String,
    pub cnt: i32,
    #[sqlx(rename = "upNeedResource")]
    pub up_need_resource: Option<String>,
    #[sqlx(rename = "needTalent")]
    pub need_talent: Option<String>,
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

/// 领主天赋阶段（s_lord_talent_stage）
#[derive(Debug, Clone, FromRow)]
pub struct StaticLordTalentStage {
    #[sqlx(rename = "stageId")]
    pub stage_id: i32,
    pub dec: Option<String>,
    #[sqlx(rename = "stageType")]
    pub stage_type: Option<i32>,
    pub quality: Option<i32>,
    #[sqlx(rename = "needTalentNum")]
    pub need_talent_num: Option<i32>,
    #[sqlx(rename = "stageAward")]
    pub stage_award: Option<String>,
    #[sqlx(rename = "stageTitle")]
    pub stage_title: Option<i32>,
    #[sqlx(rename = "buffEffectId")]
    pub buff_effect_id: Option<String>,
    pub icon: Option<String>,
    pub fight: Option<i32>,
}

/// 领主天赋系统完整配置
#[derive(Debug, Clone, Default)]
pub struct LordTalentConfig {
    /// id → StaticLordTalent
    pub talents: HashMap<i32, StaticLordTalent>,
    /// id → StaticLordTalentStage
    pub talent_stages: Vec<StaticLordTalentStage>,
    // ── 二级索引 ──
    /// talentType → Vec<id>
    pub talents_by_type_idx: HashMap<i32, Vec<i32>>,
    /// talentGroup → Vec<id>
    pub talents_by_group_idx: HashMap<i32, Vec<i32>>,
}

impl LordTalentConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (talent_rows, stage_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticLordTalent>("SELECT * FROM s_lord_talent").fetch_all(pool),
            sqlx::query_as::<_, StaticLordTalentStage>("SELECT * FROM s_lord_talent_stage").fetch_all(pool),
        )?;

        let talents: HashMap<i32, StaticLordTalent> = talent_rows
            .into_iter().map(|r| (r.id, r)).collect();

        let mut cfg = Self { talents, talent_stages: stage_rows, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (id, t) in &self.talents {
            self.talents_by_type_idx.entry(t.talent_type).or_default().push(*id);
            self.talents_by_group_idx.entry(t.talent_group).or_default().push(*id);
        }
    }
}
