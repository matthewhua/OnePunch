use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use deadpool_redis::{Config, Pool, Runtime};

/// 初始化 MySQL 连接池
pub async fn init_mysql(url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(50)
        .connect(url)
        .await
}

/// 初始化 Redis 连接池
pub fn init_redis(url: &str) -> Result<Pool, deadpool_redis::CreatePoolError> {
    let cfg = Config::from_url(url);
    cfg.create_pool(Some(Runtime::Tokio1))
}
