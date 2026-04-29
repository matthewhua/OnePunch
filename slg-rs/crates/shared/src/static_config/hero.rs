//! 将领系统静态配置
//!
//! 对应数据库表：
//! - `s_hero`：将领基础属性
//! - `s_hero_lv`：将领等级经验
//! - `s_hero_star`：将领升星
//! - `s_skill`：技能
//! - `s_skill_lv`：技能等级

use std::collections::HashMap;
use sqlx::FromRow;

/// 将领基础属性（s_hero）
#[derive(Debug, Clone, FromRow)]
pub struct StaticHero {
    #[sqlx(rename = "heroId")]
    pub hero_id: i32,
    pub name: String,
    #[sqlx(rename = "unitType")]
    pub unit_type: String,
    #[sqlx(rename = "heroType")]
    pub hero_type: i32,
    #[sqlx(rename = "atkType")]
    pub atk_type: i32,
    pub state: Option<String>,
    pub quality: String,
    pub line: Option<i32>,
    #[sqlx(rename = "heroEquipId")]
    pub hero_equip_id: Option<i32>,
    #[sqlx(rename = "bestPosition")]
    pub best_position: Option<String>,
    pub attr: String,
    #[sqlx(rename = "attrLvSmallUp")]
    pub attr_lv_small_up: Option<String>,
    #[sqlx(rename = "attrLvLargeUp")]
    pub attr_lv_large_up: Option<String>,
    #[sqlx(rename = "tdAttr")]
    pub td_attr: Option<String>,
    #[sqlx(rename = "tdAttrUp")]
    pub td_attr_up: Option<String>,
    #[sqlx(rename = "fragmentId")]
    pub fragment_id: Option<i32>,
    #[sqlx(rename = "needFragment")]
    pub need_fragment: Option<i32>,
    #[sqlx(rename = "exchangeNeedRes")]
    pub exchange_need_res: Option<String>,
    #[sqlx(rename = "fragmentGet")]
    pub fragment_get: Option<i32>,
    #[sqlx(rename = "battleSkill")]
    pub battle_skill: Option<String>,
    #[sqlx(rename = "outsideSkill")]
    pub outside_skill: Option<String>,
    #[sqlx(rename = "starSkill")]
    pub star_skill: Option<String>,
    pub gender: Option<bool>,
}

/// 将领等级经验（s_hero_lv）
#[derive(Debug, Clone, FromRow)]
pub struct StaticHeroLv {
    pub id: i32,
    pub level: i32,
    #[sqlx(rename = "needBuilding")]
    pub need_building: String,
    #[sqlx(rename = "upNeedResource")]
    pub up_need_resource: Option<String>,
}

/// 将领升星（s_hero_star）
#[derive(Debug, Clone, FromRow)]
pub struct StaticHeroStar {
    pub id: i32,
    #[sqlx(rename = "heroId")]
    pub hero_id: i32,
    pub tier: i32,
    #[sqlx(rename = "tierDisplay")]
    pub tier_display: i32,
    #[sqlx(rename = "starDisplay")]
    pub star_display: i32,
    pub quality: String,
    #[sqlx(rename = "upNeedResource")]
    pub up_need_resource: Option<String>,
    #[sqlx(rename = "attrStarUp")]
    pub attr_star_up: Option<String>,
    #[sqlx(rename = "tdAttrUp")]
    pub td_attr_up: Option<String>,
    #[sqlx(rename = "skillUnlock")]
    pub skill_unlock: Option<String>,
    #[sqlx(rename = "skillLv")]
    pub skill_lv: Option<String>,
}

/// 技能（s_skill）
#[derive(Debug, Clone, FromRow)]
pub struct StaticSkill {
    pub id: i32,
    #[sqlx(rename = "skillId")]
    pub skill_id: Option<i32>,
    #[sqlx(rename = "skillType")]
    pub skill_type: Option<i32>,
    #[sqlx(rename = "sceneType")]
    pub scene_type: Option<i32>,
    pub name: Option<String>,
    pub dec: Option<String>,
    pub value: Option<String>,
    pub quality: Option<i32>,
    pub range: Option<String>,
    pub icon: Option<String>,
    #[sqlx(rename = "buffEffectId")]
    pub buff_effect_id: Option<String>,
}

// ─── 聚合配置 ─────────────────────────────────────────────────────────────────

/// 将领系统完整配置
#[derive(Debug, Clone, Default)]
pub struct HeroConfig {
    /// heroId → StaticHero
    pub heroes: HashMap<i32, StaticHero>,
    /// id → StaticHeroLv
    pub hero_levels: Vec<StaticHeroLv>,
    /// id → StaticHeroStar
    pub hero_stars: Vec<StaticHeroStar>,
    /// skillId → StaticSkill
    pub skills: HashMap<i32, StaticSkill>,

    // ── 二级索引 ──
    /// heroId → Vec<StaticHeroStar index>（按将领分组的升星数据）
    pub stars_by_hero_idx: HashMap<i32, Vec<usize>>,
    /// heroType → Vec<heroId>
    pub heroes_by_type_idx: HashMap<i32, Vec<i32>>,
}

impl HeroConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (hero_rows, level_rows, star_rows, skill_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticHero>("SELECT * FROM s_hero").fetch_all(pool),
            sqlx::query_as::<_, StaticHeroLv>("SELECT * FROM s_hero_lv").fetch_all(pool),
            sqlx::query_as::<_, StaticHeroStar>("SELECT * FROM s_hero_star").fetch_all(pool),
            sqlx::query_as::<_, StaticSkill>("SELECT * FROM s_skill").fetch_all(pool),
        )?;

        let heroes: HashMap<i32, StaticHero> = hero_rows
            .into_iter().map(|r| (r.hero_id, r)).collect();
        let skills: HashMap<i32, StaticSkill> = skill_rows
            .into_iter().map(|r| (r.id, r)).collect();

        let mut cfg = Self {
            heroes, hero_levels: level_rows, hero_stars: star_rows, skills,
            ..Default::default()
        };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (i, star) in self.hero_stars.iter().enumerate() {
            self.stars_by_hero_idx.entry(star.hero_id).or_default().push(i);
        }
        for (hid, hero) in &self.heroes {
            self.heroes_by_type_idx.entry(hero.hero_type).or_default().push(*hid);
        }
    }
}
