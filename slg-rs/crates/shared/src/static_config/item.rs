//! 道具系统静态配置
//!
//! 对应数据库表：
//! - `s_prop_conf`：道具基础信息

use std::collections::HashMap;
use sqlx::FromRow;

/// 道具基础信息（s_prop_conf）
#[derive(Debug, Clone, FromRow)]
pub struct StaticPropConf {
    #[sqlx(rename = "propId")]
    pub prop_id: i32,
    #[sqlx(rename = "desc")]
    pub description: String,
    pub desc2: String,
    pub asset: Option<String>,
    #[sqlx(rename = "assetBase")]
    pub asset_base: Option<String>,
    pub badge: Option<String>,
    #[sqlx(rename = "getWay")]
    pub get_way: Option<i32>,
    #[sqlx(rename = "propType")]
    pub prop_type: i32,
    pub quality: i32,
    pub order: i32,
    #[sqlx(rename = "rewardList")]
    pub reward_list: Option<String>,
    #[sqlx(rename = "backType")]
    pub back_type: Option<i32>,
    #[sqlx(rename = "canSell")]
    pub can_sell: Option<i32>,
    pub duration: Option<i32>,
    pub attrs: Option<String>,
    pub jump: Option<String>,
    #[sqlx(rename = "numDisplay")]
    pub num_display: Option<i32>,
    #[sqlx(rename = "disPosition")]
    pub dis_position: Option<i32>,
    #[sqlx(rename = "canUse")]
    pub can_use: Option<i32>,
    #[sqlx(rename = "cliButton")]
    pub cli_button: Option<i32>,
    #[sqlx(rename = "batchUse")]
    pub batch_use: Option<i32>,
    #[sqlx(rename = "shopPropId")]
    pub shop_prop_id: Option<i32>,
    #[sqlx(rename = "functionOpen")]
    pub function_open: Option<String>,
    pub access: Option<String>,
    pub effect: Option<String>,
    #[sqlx(rename = "showType")]
    pub show_type: Option<i32>,
    #[sqlx(rename = "buffEffectId")]
    pub buff_effect_id: Option<String>,
    #[sqlx(rename = "effectTipsType")]
    pub effect_tips_type: i32,
}

/// 道具系统完整配置
#[derive(Debug, Clone, Default)]
pub struct ItemConfig {
    /// propId → StaticPropConf
    pub props: HashMap<i32, StaticPropConf>,
    // ── 二级索引 ──
    /// propType → Vec<propId>
    pub props_by_type_idx: HashMap<i32, Vec<i32>>,
}

impl ItemConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let rows = sqlx::query_as::<_, StaticPropConf>("SELECT * FROM s_prop_conf")
            .fetch_all(pool).await?;

        let props: HashMap<i32, StaticPropConf> = rows
            .into_iter().map(|r| (r.prop_id, r)).collect();

        let mut cfg = Self { props, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (pid, p) in &self.props {
            self.props_by_type_idx.entry(p.prop_type).or_default().push(*pid);
        }
    }
}
