pub mod activity;
pub use activity::ActivityConfig;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Debug, Clone, Default)]
pub struct StaticConfig {
    pub activity: ActivityConfig,
    // 未来扩展 hero, building 等等
}

impl StaticConfig {
    pub async fn load_from_db(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let activity = ActivityConfig::load(pool).await?;
        Ok(Self { activity })
    }
}

/// 包装了配置与 watch Channel 发送端的全局单例
pub struct ConfigWatcher {
    pub db: sqlx::MySqlPool,
    pub tx: watch::Sender<Arc<StaticConfig>>,
}

impl ConfigWatcher {
    pub async fn new(db: sqlx::MySqlPool) -> anyhow::Result<(Self, watch::Receiver<Arc<StaticConfig>>)> {
        let initial_config = StaticConfig::load_from_db(&db).await?;
        let (tx, rx) = watch::channel(Arc::new(initial_config));
        Ok((Self { db, tx }, rx))
    }

    pub async fn reload(&self) -> anyhow::Result<()> {
        let new_config = StaticConfig::load_from_db(&self.db).await?;
        let _ = self.tx.send(Arc::new(new_config));
        Ok(())
    }
}
