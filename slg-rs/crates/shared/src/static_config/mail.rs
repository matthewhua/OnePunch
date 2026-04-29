//! 邮件系统静态配置
//!
//! 对应数据库表：
//! - `s_mail`：邮件模板

use std::collections::HashMap;
use sqlx::FromRow;

/// 邮件模板（s_mail）
#[derive(Debug, Clone, FromRow)]
pub struct StaticMail {
    pub id: i32,
    #[sqlx(rename = "type")]
    pub mail_type: Option<i32>,
    #[sqlx(rename = "tabType")]
    pub tab_type: Option<i32>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub icon: Option<String>,
    #[sqlx(rename = "chatId")]
    pub chat_id: Option<i32>,
    pub duration: Option<i32>,
    pub banner: Option<String>,
}

/// 邮件系统完整配置
#[derive(Debug, Clone, Default)]
pub struct MailConfig {
    /// id → StaticMail
    pub mails: HashMap<i32, StaticMail>,
}

impl MailConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let rows = sqlx::query_as::<_, StaticMail>("SELECT * FROM s_mail")
            .fetch_all(pool).await?;

        let mails: HashMap<i32, StaticMail> = rows
            .into_iter().map(|r| (r.id, r)).collect();

        Ok(Self { mails })
    }
}
