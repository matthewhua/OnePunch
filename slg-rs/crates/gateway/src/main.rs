use tokio::net::TcpListener;
use tracing::{info, error};
use std::sync::Arc;
use shared::config::AppConfig;
use shared::registry::EtcdRegistry;

mod codec;
mod handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    // 2. 加载配置
    let config = AppConfig::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config ({}), using defaults", e);
        AppConfig::default()
    });

    info!("Starting Gateway on {}", config.server_addr);

    // 3. 服务发现（尝试 etcd，失败则降级到静态配置）
    let auth_addr = {
        let registry = EtcdRegistry::new(config.etcd_endpoints.clone()).await;
        match registry {
            Ok(r) => r.discover_one("auth").await.unwrap_or_else(|e| {
                info!("Dynamic auth discovery failed: {}, falling back to config", e);
                config.auth_service_addr.clone()
            }),
            Err(e) => {
                info!("Etcd unavailable: {}, using static auth addr", e);
                config.auth_service_addr.clone()
            }
        }
    };
    info!("Target Auth Service: {}", auth_addr);

    // 4. 启动 TCP 监听
    let listener = TcpListener::bind(&config.server_addr).await?;
    info!("Gateway listening on {}", config.server_addr);

    // 5. 初始化 Handler 工厂
    let handler = Arc::new(handler::ConnectionHandler::new(auth_addr));

    // 6. 循环接受连接
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        info!("New connection from {}", peer_addr);
        let handler_clone = handler.clone();

        tokio::spawn(async move {
            if let Err(e) = handler_clone.handle(stream).await {
                error!("Handler error from {}: {}", peer_addr, e);
            }
        });
    }
}
