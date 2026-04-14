use common::slg::world_service_client::WorldServiceClient;
use common::slg::JoinMapRequest;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化日志系统
    tracing_subscriber::fmt::init();

    // 2. 定义 World 服的地址
    // 注意：在真实的 SLG 架构中，Gateway 通常会通过 Service Discovery 获取逻辑服地址
    let world_addr = "http://127.0.0.1:50051";
    info!("Gateway 启动中...");
    info!("正在建立与 World 服务的连接: {}", world_addr);

    // 3. 初始化 gRPC 客户端
    let mut client = WorldServiceClient::connect(world_addr).await?;
    info!("gRPC 连接已建立。");

    // 4. 发起一次模拟请求：让玩家 10086 尝试加入 1 号地图
    info!("正在发送请求: JoinMap(role_id=10086, map_id=1)");
    
    let request = tonic::Request::new(JoinMapRequest {
        role_id: 10086,
        map_id: 1,
    });

    // 这里是异步调用，Gateway 会非阻塞地等待 World 服返回
    let response = client.join_map(request).await?;

    // 5. 打印响应结果
    info!("收到后端逻辑服响应: {:?}", response.into_inner());

    info!("Gateway 演示任务完成，程序退出。");
    Ok(())
}
