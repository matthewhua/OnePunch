use tonic::{Request, Response, Status};
use proto::slg::home_service_server::HomeService;
use proto::slg::{BeginGameRq, BeginGameRs, CreateRoleRq, CreateRoleRs, RoleLoginRq, RoleLoginRs};
use sqlx::MySqlPool;
use std::sync::Arc;
use crate::managers::player_manager::PlayerManager;
use crate::actors::player_actor::PlayerMessage;
use rand::{distributions::Alphanumeric, Rng};
use tracing::info;

pub struct HomeServiceImpl {
    db: MySqlPool,
    manager: Arc<PlayerManager>,
}

impl HomeServiceImpl {
    pub fn new(db: MySqlPool, manager: Arc<PlayerManager>) -> Self {
        Self { db, manager }
    }

    /// 生成随机名: Guest{serverId}{Random}
    fn generate_random_name(&self, server_id: i32) -> String {
        let suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        format!("Guest{}{}", server_id, suffix)
    }
}

#[tonic::async_trait]
impl HomeService for HomeServiceImpl {
    async fn begin_game(
        &self,
        request: Request<BeginGameRq>,
    ) -> Result<Response<BeginGameRs>, Status> {
        let req = request.into_inner();
        let account_id = req.key_id; // 假设 keyId 就是 account_id
        
        // 1. 检查角色是否存在
        let role: Option<(i64, i32, String)> = sqlx::query_as("SELECT role_id, camp, nickname FROM p_role WHERE account_id = ? AND serverId = ?")
            .bind(account_id)
            .bind(req.server_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut res = BeginGameRs::default();
        res.time = Some(chrono::Utc::now().timestamp() as i32);

        match role {
            Some((role_id, camp, _)) => {
                res.state = Some(2); // 已创建
                res.role_id = Some(role_id);
                res.camp = Some(camp);
            }
            None => {
                res.state = Some(1); // 未创建
                // 生成一些随机名字供预览
                res.name = vec![
                    self.generate_random_name(req.server_id),
                    self.generate_random_name(req.server_id),
                    self.generate_random_name(req.server_id),
                ];
                res.camp = Some(1); // 默认推荐阵营
            }
        }

        Ok(Response::new(res))
    }

    async fn create_role(
        &self,
        request: Request<CreateRoleRq>,
    ) -> Result<Response<CreateRoleRs>, Status> {
        let req = request.into_inner();
        // 此处需要获取 account_id。通常从 context 获取，此处演示直接先假设从某处拿。
        // 为了演示，我们假设 account_id 在请求里或通过外部注入。
        // TODO: 完善从 gRPC Metadata 获取 account_id
        
        info!("Creating role for server {}: {}", req.server_id.unwrap_or(0), req.nick.as_deref().unwrap_or(""));

        // 简化的创角逻辑
        // TODO: 实现真正的 DB 插入并返回 role_id
        
        Ok(Response::new(CreateRoleRs {
            state: Some(1), // 成功
            ..Default::default()
        }))
    }

    async fn role_login(
        &self,
        request: Request<RoleLoginRq>,
    ) -> Result<Response<RoleLoginRs>, Status> {
        // TODO: 从 Manager 启动或获取 Actor
        // TODO: 这里的逻辑需要 account_id 和 role_id
        Ok(Response::new(RoleLoginRs {
            state: Some(1),
            ..Default::default()
        }))
    }
}
