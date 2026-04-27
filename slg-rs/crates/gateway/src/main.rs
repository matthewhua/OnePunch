use tokio::net::TcpListener;
use tracing::{info, error};
use std::sync::Arc;
use shared::config::AppConfig;

mod codec;
mod handler;
use shared::registry::EtcdRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    info!("Starting Gateway on {}", config.server_addr);
    
    // 3. 服务发现 (从 Etcd 获取 Auth 地址)
    let registry = EtcdRegistry::new(config.etcd_endpoints.clone()).await?;
    let auth_addr = registry.discover_one("auth").await.unwrap_or_else(|e| {
        info!("Dynamic auth discovery failed: {}, falling back to config", e);
        config.auth_service_addr.clone()
    });
    info!("Target Auth Service: {}", auth_addr);

    // 3. 启动 TCP 监听
    let listener = TcpListener::bind(&config.server_addr).await?;
    
    // 4. 初始化 Handler 工厂
    let handler = Arc::new(handler::ConnectionHandler::new(config.auth_service_addr.clone()));

    // 5. 循环接受连接
    loop {
        let (stream, _) = listener.accept().await?;
        let handler_clone = handler.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handler_clone.handle(stream).await {
                error!("Handler error: {}", e);
            }
        });
    }
}
