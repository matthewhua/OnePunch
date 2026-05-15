use anyhow::{Context, Result};
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool;
use tracing::debug;
use uuid::Uuid;

/// Session 管理器
///
/// 使用 Redis 存储 token → account_key_id 映射。
/// key 格式：`session:{token}`，值为 account_key_id（i64）。
pub struct SessionManager {
    redis: Pool,
    /// Token 有效期（秒）
    ttl_seconds: u64,
}

#[allow(dead_code)]
impl SessionManager {
    pub fn new(redis: Pool, ttl_seconds: u64) -> Self {
        Self { redis, ttl_seconds }
    }

    /// 创建新会话，返回 UUID Token
    pub async fn create_session(&self, account_key_id: i64) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let key = format!("session:{}", token);

        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        // SET key value EX ttl
        conn.set_ex::<_, _, ()>(&key, account_key_id, self.ttl_seconds)
            .await
            .context("Failed to set session in Redis")?;

        debug!(account_key_id, token = %token, ttl = self.ttl_seconds, "Session created");
        Ok(token)
    }

    /// 验证 Token，返回 account_key_id（不存在或过期返回 None）
    pub async fn validate_session(&self, token: &str) -> Result<Option<i64>> {
        let key = format!("session:{}", token);

        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        let account_key_id: Option<i64> = conn
            .get(&key)
            .await
            .context("Failed to get session from Redis")?;

        Ok(account_key_id)
    }

    /// 删除会话（登出时调用）
    pub async fn delete_session(&self, token: &str) -> Result<()> {
        let key = format!("session:{}", token);

        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        conn.del::<_, ()>(&key)
            .await
            .context("Failed to delete session from Redis")?;

        Ok(())
    }

    /// 刷新 Token 过期时间（心跳时调用）
    pub async fn refresh_session(&self, token: &str) -> Result<bool> {
        let key = format!("session:{}", token);

        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        let refreshed: bool = conn
            .expire(&key, self.ttl_seconds as i64)
            .await
            .context("Failed to refresh session TTL")?;

        Ok(refreshed)
    }
}
