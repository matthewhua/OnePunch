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

    // 2. 加载配置 (假设有默认配置或从环境变量读取)
    // 注意：在没有配置文件时，AppConfig::load 会返回错误，这里为了演示使用硬编码 fallback 或让它报错提示
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig {
        database_url: "mysql://root:password@localhost:3306/slg".to_string(),
        redis_url: "redis://127.0.0.1:6379".to_string(),
        server_addr: "0.0.0.0:50051".to_string(),
    });

    info!("Starting Auth Service on {}", config.server_addr);

    // 3. 初始化数据库连接
    let db = init_mysql(&config.database_url).await?;
    let redis = init_redis(&config.redis_url)?;
    
    // 4. 初始化业务组件
    let session_mgr = Arc::new(session::SessionManager::new(redis, 30)); // 30s 有效期
    let auth_service = service::AuthServiceImpl::new(db, session_mgr);

    // 5. 启动 gRPC 服务
    let addr = config.server_addr.parse()?;
    
    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
