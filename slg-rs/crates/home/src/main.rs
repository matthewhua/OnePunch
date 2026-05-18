use tonic::transport::Server;
use proto::slg::home_service_server::HomeServiceServer;
use shared::config::AppConfig;
use shared::db::init_mysql;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::info;
use shared::event::GlobalEvent;
use crate::actors::global_event_bus::GlobalEventBus;
use crate::actors::activity_actor::{ActivityActor, ActivityMessage};
use crate::systems::rank::RankSystem;
use shared::registry::EtcdRegistry;

mod service;
pub mod systems;
pub mod actors;
pub mod managers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    // 2. 加载配置
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());

    info!("Starting Home Service on {}", config.server_addr);
    let server_id = parse_server_id(&config.server_id);

    // 3. 初始化数据库
    let db = init_mysql(&config.database_url).await?;

    // 4. 服务注册（etcd feature 未启用时跳过）
    let _registry = EtcdRegistry::new(config.etcd_endpoints.clone()).await;
    // 注意：register 方法仅在 etcd feature 启用时可用
    // registry.register("home", &config.server_id, &config.server_addr, 30).await?;
    // 初始化静态配置及 Watcher
    let (config_watcher, config_rx) = shared::static_config::ConfigWatcher::new(db.clone()).await?;
    let _config_watcher = Arc::new(config_watcher);
    // 4. 初始化事件总线与全局 ActivityActor
    let (global_to_activity_tx, global_to_activity_rx) = mpsc::channel::<GlobalEvent>(1024);
    let (_act_msg_tx, act_msg_rx) = mpsc::channel::<ActivityMessage>(1024);
    
    let event_bus = GlobalEventBus::new(global_to_activity_tx);

    let activity_actor = ActivityActor::new(act_msg_rx, global_to_activity_rx, config_rx.clone());
    tokio::spawn(async move {
        activity_actor.run().await;
    });

    // 5. 初始化管理器、Rank 全局持久化与服务
    let dao = Arc::new(shared::persistence::PlayerDao::new(db.clone()));
    let _rank_persistence_task = RankSystem::spawn_global_persistence_task(
        dao.clone(),
        server_id,
        Duration::from_secs(30),
    );
    let manager = Arc::new(managers::player_manager::PlayerManager::new(event_bus, config_rx, dao));
    let home_service = service::HomeServiceImpl::new(db, manager);

    // 5. 启动 gRPC 服务
    let addr = config.server_addr.parse()?;
    
    Server::builder()
        .add_service(HomeServiceServer::new(home_service))
        .serve(addr)
        .await?;

    Ok(())
}

fn parse_server_id(raw: &str) -> i32 {
    raw.rsplit(['-', '_', ':'])
        .next()
        .and_then(|part| part.parse::<i32>().ok())
        .or_else(|| raw.parse::<i32>().ok())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::parse_server_id;

    #[test]
    fn parses_numeric_suffix_server_id() {
        assert_eq!(parse_server_id("default-1"), 1);
        assert_eq!(parse_server_id("home_12"), 12);
        assert_eq!(parse_server_id("7"), 7);
        assert_eq!(parse_server_id("dev"), 1);
    }
}
