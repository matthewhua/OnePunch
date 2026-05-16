use proto::slg::world_service_server::WorldServiceServer;
use shared::config::AppConfig;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;
use tracing::{error, info};

pub mod arrival;
pub mod assembly;
mod circuit_breaker;
pub mod collect;
pub mod garrison;
mod health;
mod map;
mod march;
mod message;
mod metrics;
pub mod outbound;
pub mod router;
mod rpc_client;
mod runtime;
mod sector_actor;
mod service;
mod shutdown;
mod supervisor;
mod timer_wheel;
mod wal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();
    let config = AppConfig::load().unwrap_or_default();

    // 2. 初始化核心组件
    let grid = Arc::new(map::grid::MapGrid::new());
    let marching_mgr = Arc::new(march::MarchingManager::new());

    // 3. 启动行军 Ticker (每 100ms 检查一次)
    let march_clone = marching_mgr.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let arrived = march_clone.tick();
            for troop_key in arrived {
                info!("Troop {} has arrived at its destination", troop_key);
                // TODO: 触发到达逻辑 (战斗、采集等)
            }
        }
    });

    // 4. 启动 gRPC 服务
    let bind_addr = config
        .world_service_addr
        .strip_prefix("http://")
        .or_else(|| config.world_service_addr.strip_prefix("https://"))
        .unwrap_or(&config.world_service_addr);
    let addr = bind_addr.parse()?;
    let (home_outbound_tx, mut home_outbound_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = home_outbound_rx.recv().await {
            info!(?event, "World outbound Home event consumed");
        }
    });
    let outbound_dispatcher = outbound::WorldOutboundDispatcher::new(
        Arc::new(outbound::ChannelOutboundSink::new(home_outbound_tx)),
        Arc::new(outbound::InMemoryOutboundSink::new()),
    );
    let world_service = service::WorldServiceImpl::new_with_outbound(
        grid.clone(),
        marching_mgr.clone(),
        Arc::new(outbound_dispatcher),
    );

    info!("World Service starting on {}", addr);

    Server::builder()
        .add_service(WorldServiceServer::new(world_service))
        .serve(addr)
        .await?;

    Ok(())
}
