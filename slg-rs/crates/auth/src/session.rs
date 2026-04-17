use deadpool_redis::{redis::cmd, Pool};
use uuid::Uuid;
use shared::GameError;

pub struct SessionManager {
    redis: Pool,
    ttl_seconds: usize,
}

impl SessionManager {
    pub fn new(redis: Pool, ttl_seconds: usize) -> Self {
        Self { redis, ttl_seconds }
    }

    /// 创建新会话，返回 UUID Token
    pub async fn create_session(&self, account_id: i64) -> Result<String, GameError> {
        let token = Uuid::new_v4().to_string();
        let key = format!("token:{}", token);
        
        let mut conn = self.redis.get().await?;
        
        // 使用 Redis SET 命令设置值并设置过期时间
        cmd("SET")
            .arg(&key)
            .arg(account_id)
            .arg("EX")
            .arg(self.ttl_seconds)
            .query_async::<_, ()>(&mut conn)
            .await?;
            
        Ok(token)
    }

    /// 验证会话，返回 account_id
    pub async fn validate_session(&self, token: &str) -> Result<Option<i64>, GameError> {
        let key = format!("token:{}", token);
        let mut conn = self.redis.get().await?;
        
        let account_id: Option<i64> = cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
            
        Ok(account_id)
    }
}
