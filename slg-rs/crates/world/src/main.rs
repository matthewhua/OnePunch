use anyhow::{anyhow, Result};
use proto::slg::{
    home_service_client::HomeServiceClient, world_service_server::WorldServiceServer,
    WorldOutboundRq, WorldOutboundRs,
};
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
async fn main() -> Result<()> {
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
    let home_service_addr = config.home_service_addr.clone();
    let (home_outbound_tx, home_outbound_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        run_home_outbound_consumer(home_service_addr, home_outbound_rx).await;
    });
    let role_resolver_marching_mgr = marching_mgr.clone();
    let outbound_dispatcher = outbound::WorldOutboundDispatcher::new(
        Arc::new(outbound::HomeOutboundChannelSink::new(
            home_outbound_tx,
            move |event| role_resolver_marching_mgr.troop_owner(event.troop_key()),
        )),
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

async fn run_home_outbound_consumer(
    home_service_addr: String,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<WorldOutboundRq>,
) {
    while let Some(request) = rx.recv().await {
        match send_home_outbound_with_retry(&home_service_addr, request.clone()).await {
            Ok(response) if response.code == 0 => {
                info!(
                    role_id = request.role_id,
                    event_type = request.event_type,
                    troop_key = request.troop_key,
                    "World outbound delivered to Home"
                );
            }
            Ok(response) => {
                error!(
                    role_id = request.role_id,
                    event_type = request.event_type,
                    troop_key = request.troop_key,
                    code = response.code,
                    msg = %response.msg,
                    "Home rejected World outbound event"
                );
            }
            Err(err) => {
                error!(
                    role_id = request.role_id,
                    event_type = request.event_type,
                    troop_key = request.troop_key,
                    error = %err,
                    "Failed to deliver World outbound event to Home"
                );
            }
        }
    }
}

async fn send_home_outbound_with_retry(
    home_service_addr: &str,
    request: WorldOutboundRq,
) -> Result<WorldOutboundRs> {
    let mut last_error = None;

    for attempt in 1..=3 {
        match HomeServiceClient::connect(home_service_addr.to_string()).await {
            Ok(mut client) => match client.world_outbound(request.clone()).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.code >= 500 && attempt < 3 {
                        last_error = Some(anyhow!(
                            "Home WorldOutbound returned code={} msg={}",
                            response.code,
                            response.msg
                        ));
                    } else {
                        return Ok(response);
                    }
                }
                Err(err) => {
                    last_error = Some(anyhow!("Home WorldOutbound RPC failed: {}", err));
                }
            },
            Err(err) => {
                last_error = Some(anyhow!("connect Home service failed: {}", err));
            }
        }

        tokio::time::sleep(Duration::from_millis(100 * attempt)).await;
    }

    Err(last_error.unwrap_or_else(|| anyhow!("Home WorldOutbound delivery failed")))
}
