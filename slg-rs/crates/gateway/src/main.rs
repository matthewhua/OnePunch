use tokio::net::TcpListener;
use tracing::{info, warn, error};
use std::sync::Arc;
use shared::config::AppConfig;

mod codec;
mod handler;
mod session;

use session::{SessionStore, disconnect_channel, DisconnectNotice};
use handler::ConnectionHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    // 2. 加载配置
    let config = AppConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load config ({}), using defaults", e);
        AppConfig::default()
    });

    info!("Starting Gateway on {}", config.server_addr);
    info!("Auth Service: {}", config.auth_service_addr);
    info!("Home Service: {}", config.home_service_addr);

    // 3. 初始化会话存储
    let sessions = SessionStore::new();

    // 4. 断开通知 channel（连接断开时通知 Home Service 玩家下线）
    let (disconnect_tx, mut disconnect_rx) = disconnect_channel();

    // 5. 后台任务：处理玩家下线通知
    let home_addr_for_offline = config.home_service_addr.clone();
    tokio::spawn(async move {
        while let Some(notice) = disconnect_rx.recv().await {
            handle_disconnect(notice, &home_addr_for_offline).await;
        }
    });

    // 6. 启动 TCP 监听
    let listener = TcpListener::bind(&config.server_addr).await?;
    info!("Gateway listening on {}", config.server_addr);

    // 7. 循环接受连接
    loop {
        let (stream, peer_addr) = listener.accept().await?;

        let handler = Arc::new(ConnectionHandler::new(
            sessions.clone(),
            config.auth_service_addr.clone(),
            config.home_service_addr.clone(),
            disconnect_tx.clone(),
        ));

        tokio::spawn(async move {
            if let Err(e) = handler.handle(stream).await {
                error!(%peer_addr, "Connection error: {}", e);
            }
        });
    }
}

/// 处理玩家断开连接（通知 Home Service 存盘下线）
async fn handle_disconnect(notice: DisconnectNotice, home_addr: &str) {
    if let Some(role_id) = notice.role_id {
        if role_id == 0 {
            return; // 未进入游戏，无需通知
        }
        info!(
            conn_id = notice.conn_id,
            account_key_id = notice.account_key_id,
            role_id,
            "Player offline, notifying Home Service"
        );
        // TODO: 调用 HomeService::PlayerOffline gRPC 接口
        // let mut client = HomeServiceClient::connect(home_addr).await?;
        // client.player_offline(PlayerOfflineRq { role_id }).await?;
        let _ = home_addr;
    }
}
