use tonic::transport::Server;
use proto::slg::home_service_server::HomeServiceServer;
use shared::config::AppConfig;
use shared::db::init_mysql;
use std::sync::Arc;
use tracing::info;

mod service;
pub mod systems;
pub mod actors;
pub mod managers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    // 2. 加载配置
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig {
        database_url: "mysql://root:password@localhost:3306/slg".to_string(),
        redis_url: "redis://127.0.0.1:6379".to_string(),
        server_addr: "0.0.0.0:50052".to_string(), // Home 监听端口
    });

    info!("Starting Home Service on {}", config.server_addr);

    // 3. 初始化数据库
    let db = init_mysql(&config.database_url).await?;
    
    // 4. 初始化管理器与服务
    let manager = Arc::new(managers::player_manager::PlayerManager::new());
    let home_service = service::HomeServiceImpl::new(db, manager);

    // 5. 启动 gRPC 服务
    let addr = config.server_addr.parse()?;
    
    Server::builder()
        .add_service(HomeServiceServer::new(home_service))
        .serve(addr)
        .await?;

    Ok(())
}
