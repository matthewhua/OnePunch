//! 充值系统静态配置
//!
//! 对应数据库表：
//! - `s_pay`：充值商品

use std::collections::HashMap;
use sqlx::FromRow;

/// 充值商品（s_pay）
#[derive(Debug, Clone, FromRow)]
pub struct StaticPay {
    #[sqlx(rename = "payId")]
    pub pay_id: i32,
    pub dsc: Option<String>,
    pub topup: i32,
    #[sqlx(rename = "banFlag")]
    pub ban_flag: i32,
    pub asset: i32,
    #[sqlx(rename = "productId")]
    pub product_id: Option<String>,
    pub price: i32,
    pub usd: String,
    pub icon: String,
    pub eventpts: i32,
    #[sqlx(rename = "goldenPrice")]
    pub golden_price: Option<i32>,
    #[sqlx(rename = "goldenNum")]
    pub golden_num: Option<String>,
    pub pay_score: Option<String>,
}

/// 充值系统完整配置
#[derive(Debug, Clone, Default)]
pub struct PayConfig {
    /// payId → StaticPay
    pub pays: HashMap<i32, StaticPay>,
}

impl PayConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let rows = sqlx::query_as::<_, StaticPay>("SELECT * FROM s_pay")
            .fetch_all(pool).await?;

        let pays: HashMap<i32, StaticPay> = rows
            .into_iter().map(|r| (r.pay_id, r)).collect();

        Ok(Self { pays })
    }
}
