//! 商店系统静态配置
//!
//! 对应数据库表：
//! - `s_shop`：商店定义
//! - `s_shop_prop`：商店商品

use std::collections::HashMap;
use sqlx::FromRow;

/// 商店定义（s_shop）
#[derive(Debug, Clone, FromRow)]
pub struct StaticShop {
    pub id: i32,
    pub name: Option<String>,
    #[sqlx(rename = "shopType")]
    pub shop_type: Option<i32>,
    #[sqlx(rename = "showType")]
    pub show_type: Option<i32>,
    #[sqlx(rename = "showRes")]
    pub show_res: Option<String>,
    #[sqlx(rename = "limitedTime")]
    pub limited_time: Option<String>,
    #[sqlx(rename = "manualRefresh")]
    pub manual_refresh: Option<i32>,
    #[sqlx(rename = "freeRefresh")]
    pub free_refresh: Option<i32>,
    #[sqlx(rename = "refreshNeed")]
    pub refresh_need: Option<String>,
    #[sqlx(rename = "slotNum")]
    pub slot_num: Option<i32>,
    #[sqlx(rename = "refreshType")]
    pub refresh_type: Option<i32>,
    pub sort: Option<i32>,
    #[sqlx(rename = "fieldConfiguration")]
    pub field_configuration: Option<String>,
    #[sqlx(rename = "group")]
    pub group_str: Option<String>,
    #[sqlx(rename = "functionOpen")]
    pub function_open: Option<i32>,
    pub form_id: Option<String>,
}

/// 商店商品（s_shop_prop）
#[derive(Debug, Clone, FromRow)]
pub struct StaticShopProp {
    pub id: i32,
    #[sqlx(rename = "shopId")]
    pub shop_id: Option<i32>,
    pub dsc: Option<String>,
    #[sqlx(rename = "showType")]
    pub show_type: Option<i32>,
    pub prop: Option<String>,
    #[sqlx(rename = "group")]
    pub group_val: Option<i32>,
    pub weight: Option<String>,
    pub price: Option<String>,
    pub count: Option<i32>,
    #[sqlx(rename = "singleLimit")]
    pub single_limit: Option<i32>,
    pub discount: Option<i32>,
    #[sqlx(rename = "unlockTime")]
    pub unlock_time: Option<String>,
    #[sqlx(rename = "featureLv")]
    pub feature_lv: Option<String>,
    pub sort: Option<i32>,
}

/// 商店系统完整配置
#[derive(Debug, Clone, Default)]
pub struct ShopConfig {
    /// id → StaticShop
    pub shops: HashMap<i32, StaticShop>,
    /// id → StaticShopProp
    pub shop_props: Vec<StaticShopProp>,
    // ── 二级索引 ──
    /// shopId → Vec<StaticShopProp index>
    pub props_by_shop_idx: HashMap<i32, Vec<usize>>,
}

impl ShopConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (shop_rows, prop_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticShop>("SELECT * FROM s_shop").fetch_all(pool),
            sqlx::query_as::<_, StaticShopProp>("SELECT * FROM s_shop_prop").fetch_all(pool),
        )?;

        let shops: HashMap<i32, StaticShop> = shop_rows
            .into_iter().map(|r| (r.id, r)).collect();

        let mut cfg = Self { shops, shop_props: prop_rows, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (i, p) in self.shop_props.iter().enumerate() {
            if let Some(sid) = p.shop_id {
                self.props_by_shop_idx.entry(sid).or_default().push(i);
            }
        }
    }
}
