use tonic::transport::Server;
use proto::slg::auth_service_server::AuthServiceServer;
use shared::config::AppConfig;
use shared::db::{init_mysql, init_redis};
use std::sync::Arc;
use tracing::info;

mod service;
mod session;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    // 2. 加载配置
    let config = AppConfig::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config ({}), using defaults", e);
        AppConfig::default()
    });

    info!("Starting Auth Service on {}", config.server_addr);

    // 3. 初始化数据库连接
    let db = init_mysql(&config.database_url).await?;
    let redis = init_redis(&config.redis_url)?;

    // 4. 初始化业务组件
    // Session TTL = 86400 秒（24 小时）
    let session_mgr = Arc::new(session::SessionManager::new(redis, 86400));
    let auth_service = service::AuthServiceImpl::new(db, session_mgr);

    // 5. 启动 gRPC 服务
    let addr = config.server_addr.parse()?;
    info!("Auth gRPC listening on {}", addr);

    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
