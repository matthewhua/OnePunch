use tokio::net::TcpListener;
use tracing::{info, error};
use std::sync::Arc;
use shared::config::AppConfig;

mod codec;
mod handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    // 2. 加载配置 (硬编码 fallback)
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig {
        database_url: String::new(), // Gateway 不需要 DB
        redis_url: String::new(),    // Gateway 暂时直接调 Auth，不直接连 Redis
        server_addr: "0.0.0.0:8080".to_string(), // 客户端连接端口
    });
    
    // Auth 服务地址 (Phase 2 实现的地址)
    let auth_addr = "http://127.0.0.1:50051".to_string();

    info!("Starting Gateway on {}", config.server_addr);
    info!("Target Auth Service: {}", auth_addr);

    // 3. 启动 TCP 监听
    let listener = TcpListener::bind(&config.server_addr).await?;
    
    // 4. 初始化 Handler 工厂
    let handler = Arc::new(handler::ConnectionHandler::new(auth_addr));

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
