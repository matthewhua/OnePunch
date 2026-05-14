use tonic::{Request, Response, Status};
use proto::slg::world_service_server::WorldService;
use proto::slg::{JoinMapRequest, JoinMapResponse, RpcMsg};

pub struct WorldServiceImpl {
    // 后面可以关联 MapManager 等
}

impl WorldServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl WorldService for WorldServiceImpl {
    async fn call(&self, _request: Request<RpcMsg>) -> Result<Response<RpcMsg>, Status> {
        // 这个通用入口会在 Gateway 能把 World 命令转成 RpcMsg 之后再启用。
        // 目前 Gateway 只有 raw GameMessage bytes，而 World 命令号在 50001+，
        // 不在 RpcMsg 的 extension 范围内，因此这里保留明确的 unimplemented 边界。
        Err(Status::unimplemented("World generic call is not wired yet"))
    }

    async fn join_map(
        &self,
        request: Request<JoinMapRequest>,
    ) -> Result<Response<JoinMapResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Player {} joining map {}", req.role_id, req.map_id);
        
        // TODO: 设置玩家初始坐标并加入 AOI
        
        Ok(Response::new(JoinMapResponse {
            code: 0,
            msg: "Success".to_string(),
        }))
    }
}
