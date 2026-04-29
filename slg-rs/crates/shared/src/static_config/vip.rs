//! VIP 系统静态配置
//!
//! 对应数据库表：
//! - `s_vip`：VIP 等级特权

use std::collections::HashMap;
use sqlx::FromRow;

/// VIP 等级特权（s_vip）
#[derive(Debug, Clone, FromRow)]
pub struct StaticVip {
    #[sqlx(rename = "vipId")]
    pub vip_id: i32,
    pub level: Option<i32>,
    #[sqlx(rename = "needRes")]
    pub need_res: Option<String>,
    #[sqlx(rename = "freeGiftPack")]
    pub free_gift_pack: Option<String>,
    #[sqlx(rename = "payGiftPack")]
    pub pay_gift_pack: Option<String>,
    #[sqlx(rename = "freeBoxRes")]
    pub free_box_res: Option<String>,
    #[sqlx(rename = "freeFigureRes")]
    pub free_figure_res: Option<String>,
    #[sqlx(rename = "payHeroRes")]
    pub pay_hero_res: Option<String>,
    #[sqlx(rename = "buffEffectId")]
    pub buff_effect_id: Option<String>,
    pub fight: Option<i32>,
}

/// VIP 系统完整配置
#[derive(Debug, Clone, Default)]
pub struct VipConfig {
    /// vipId → StaticVip
    pub vips: HashMap<i32, StaticVip>,
    /// level → vipId（按等级查找）
    pub vip_by_level_idx: HashMap<i32, i32>,
}

impl VipConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let rows = sqlx::query_as::<_, StaticVip>("SELECT * FROM s_vip")
            .fetch_all(pool).await?;

        let vips: HashMap<i32, StaticVip> = rows
            .into_iter().map(|r| (r.vip_id, r)).collect();

        let mut cfg = Self { vips, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (vid, v) in &self.vips {
            if let Some(lv) = v.level {
                self.vip_by_level_idx.insert(lv, *vid);
            }
        }
    }
}
