mod player;
mod session;

use tonic::{transport::Server, Request, Response, Status};
use common::slg::world_service_server::{WorldService, WorldServiceServer};
use common::slg::{RpcMsg, JoinMapRequest, JoinMapResponse};
use tracing::{info, error};

// 定义 World 逻辑处理器
// 在真实项目中，这里通常会持有数据库连接池、内存地图数据等状态
pub struct MyWorld;

#[tonic::async_trait]
impl WorldService for MyWorld {
    /// 统一入口 Call：模拟原有的 Java/Dubbo Command 模式
    /// 这种模式适合兼容老协议，通过 cmd 编号分发逻辑
    async fn call(&self, request: Request<RpcMsg>) -> Result<Response<RpcMsg>, Status> {
        let msg = request.into_inner();
        info!("收到统一 RPC 请求: cmd={}, lord_id={}", msg.cmd, msg.lord_id);

        // TODO: 在这里编写你的逻辑分发器 (Dispatcher)
        // match msg.cmd { ... }

        // 暂时原样返回
        Ok(Response::new(msg))
    }

    /// 具体业务接口 JoinMap：展示 Rust gRPC 的强类型原生接口
    /// 推荐新开发的逻辑采用这种模式，可读性和安全性更高
    async fn join_map(&self, request: Request<JoinMapRequest>) -> Result<Response<JoinMapResponse>, Status> {
        let req = request.into_inner();
        info!("处理地图进入请求: 玩家={}, 地图={}", req.role_id, req.map_id);

        Ok(Response::new(JoinMapResponse {
            code: 0,
            msg: "Success".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化日志追踪
    tracing_subscriber::fmt::init();

    // 2. 设定监听物理地址
    let addr = "0.0.0.0:50051".parse()?;
    let world_service = MyWorld;

    info!("SLG World Logic Server 正在启动...");
    info!("gRPC 服务监听地址: {}", addr);

    // 3. 启动 Tonic Server
    Server::builder()
        .add_service(WorldServiceServer::new(world_service))
        .serve(addr)
        .await?;

    Ok(())
}
