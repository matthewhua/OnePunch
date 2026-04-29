//! 战斗系统静态配置
//!
//! 对应数据库表：
//! - `s_battle_skill`：战斗技能
//! - `s_battle_buff`：战斗 buff
//! - `s_battle_effect`：战斗效果
//! - `s_battle_type`：战斗类型
//! - `s_buff`：通用 buff
//! - `s_buff_effect`：buff 效果

use std::collections::HashMap;
use sqlx::FromRow;

/// 战斗技能（s_battle_skill）
#[derive(Debug, Clone, FromRow)]
pub struct StaticBattleSkill {
    pub id: i32,
    #[sqlx(rename = "skillId")]
    pub skill_id: Option<i32>,
    #[sqlx(rename = "skillType")]
    pub skill_type: Option<i32>,
    pub name: Option<String>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub value: Option<String>,
    pub quality: Option<i32>,
    pub range: Option<String>,
    pub icon: Option<String>,
}

/// 战斗类型（s_battle_type）
#[derive(Debug, Clone, FromRow)]
pub struct StaticBattleType {
    pub id: i32,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    pub battle_type: Option<i32>,
    pub round: Option<i32>,
}

/// 通用 buff（s_buff）
#[derive(Debug, Clone, FromRow)]
pub struct StaticBuff {
    pub id: i32,
    #[sqlx(rename = "buffType")]
    pub buff_type: Option<i32>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub duration: Option<i32>,
    pub icon: Option<String>,
    #[sqlx(rename = "effectId")]
    pub effect_id: Option<String>,
}

/// buff 效果（s_buff_effect）
#[derive(Debug, Clone, FromRow)]
pub struct StaticBuffEffect {
    pub id: i32,
    #[sqlx(rename = "effectType")]
    pub effect_type: Option<i32>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub value: Option<String>,
    pub param: Option<String>,
}

/// 战斗系统完整配置
#[derive(Debug, Clone, Default)]
pub struct BattleConfig {
    /// id → StaticBattleSkill
    pub battle_skills: HashMap<i32, StaticBattleSkill>,
    /// id → StaticBattleType
    pub battle_types: HashMap<i32, StaticBattleType>,
    /// id → StaticBuff
    pub buffs: HashMap<i32, StaticBuff>,
    /// id → StaticBuffEffect
    pub buff_effects: HashMap<i32, StaticBuffEffect>,
}

impl BattleConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (skill_rows, type_rows, buff_rows, effect_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticBattleSkill>("SELECT * FROM s_battle_skill").fetch_all(pool),
            sqlx::query_as::<_, StaticBattleType>("SELECT * FROM s_battle_type").fetch_all(pool),
            sqlx::query_as::<_, StaticBuff>("SELECT * FROM s_buff").fetch_all(pool),
            sqlx::query_as::<_, StaticBuffEffect>("SELECT * FROM s_buff_effect").fetch_all(pool),
        )?;

        let battle_skills: HashMap<i32, StaticBattleSkill> = skill_rows
            .into_iter().map(|r| (r.id, r)).collect();
        let battle_types: HashMap<i32, StaticBattleType> = type_rows
            .into_iter().map(|r| (r.id, r)).collect();
        let buffs: HashMap<i32, StaticBuff> = buff_rows
            .into_iter().map(|r| (r.id, r)).collect();
        let buff_effects: HashMap<i32, StaticBuffEffect> = effect_rows
            .into_iter().map(|r| (r.id, r)).collect();

        Ok(Self { battle_skills, battle_types, buffs, buff_effects })
    }
}
