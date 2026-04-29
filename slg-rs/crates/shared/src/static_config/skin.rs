//! 皮肤系统静态配置
//!
//! 对应数据库表：
//! - `s_skin`：皮肤定义

use std::collections::HashMap;
use sqlx::FromRow;

/// 皮肤定义（s_skin）
#[derive(Debug, Clone, FromRow)]
pub struct StaticSkin {
    pub skin_id: i32,
    pub skin_type: Option<i32>,
    pub dsc: Option<String>,
    pub quality: Option<i32>,
    pub star: Option<i32>,
    #[sqlx(rename = "getWay")]
    pub get_way: Option<i32>,
    pub unlock_cond: Option<String>,
    pub show_type: Option<i32>,
    #[sqlx(rename = "defaultSkin")]
    pub default_skin: Option<i32>,
    pub buff: Option<String>,
    pub new_tag: Option<String>,
    pub icon: Option<String>,
    pub skin_res: Option<String>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
}

/// 皮肤系统完整配置
#[derive(Debug, Clone, Default)]
pub struct SkinConfig {
    /// skin_id → StaticSkin
    pub skins: HashMap<i32, StaticSkin>,
    // ── 二级索引 ──
    /// skin_type → Vec<skin_id>
    pub skins_by_type_idx: HashMap<i32, Vec<i32>>,
}

impl SkinConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let rows = sqlx::query_as::<_, StaticSkin>("SELECT * FROM s_skin")
            .fetch_all(pool).await?;

        let skins: HashMap<i32, StaticSkin> = rows
            .into_iter().map(|r| (r.skin_id, r)).collect();

        let mut cfg = Self { skins, ..Default::default() };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        for (sid, s) in &self.skins {
            if let Some(st) = s.skin_type {
                self.skins_by_type_idx.entry(st).or_default().push(*sid);
            }
        }
    }
}
