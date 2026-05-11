use tonic::{Request, Response, Status};
use proto::slg::home_service_server::HomeService;
use proto::slg::{
    BeginGameRq, BeginGameRs, CreateRoleRq, CreateRoleRs, RoleLoginRq, RoleLoginRs,
    DispatchRq, DispatchRs, PlayerOfflineRq, PlayerOfflineRs,
};
use sqlx::MySqlPool;
use std::sync::Arc;
use crate::managers::player_manager::PlayerManager;
use crate::actors::player_actor::PlayerMessage;
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::oneshot;
use tracing::{info, warn};

pub struct HomeServiceImpl {
    db: MySqlPool,
    manager: Arc<PlayerManager>,
}

impl HomeServiceImpl {
    pub fn new(db: MySqlPool, manager: Arc<PlayerManager>) -> Self {
        Self { db, manager }
    }

    fn generate_random_name(&self, server_id: i32) -> String {
        let suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        format!("Lord{}{}", server_id, suffix)
    }
}

#[tonic::async_trait]
impl HomeService for HomeServiceImpl {
    /// BeginGame：检查角色是否存在，返回状态
    ///
    /// 对应 Java 版 BeginGameHandler。
    /// key_id 是账号唯一ID（account_key_id），server_id 是区服号。
    async fn begin_game(
        &self,
        request: Request<BeginGameRq>,
    ) -> Result<Response<BeginGameRs>, Status> {
        let req = request.into_inner();
        let account_key_id = req.key_id;
        let server_id = req.server_id;

        // 查询 p_account 是否已有该账号在本服的记录
        let account: Option<(i64, i32)> = sqlx::query_as(
            "SELECT role_id, 0 FROM p_account WHERE account_key_id = ? AND server_id = ?"
        )
        .bind(account_key_id)
        .bind(server_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let mut res = BeginGameRs::default();
        res.time = Some(chrono::Utc::now().timestamp() as i32);

        match account {
            Some((role_id, _)) => {
                // 已有角色，查询 p_lord 获取 camp
                let lord: Option<(i32,)> = sqlx::query_as(
                    "SELECT camp_id FROM p_lord WHERE role_id = ?"
                )
                .bind(role_id)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

                res.state = Some(2); // 已创建角色
                res.role_id = Some(role_id);
                res.camp = lord.map(|(c,)| c).or(Some(0));
            }
            None => {
                res.state = Some(1); // 未创建角色
                res.name = vec![
                    self.generate_random_name(server_id),
                    self.generate_random_name(server_id),
                    self.generate_random_name(server_id),
                ];
                res.camp = Some(1);
            }
        }

        Ok(Response::new(res))
    }

    /// CreateRole：创建新角色
    ///
    /// 对应 Java 版 CreateRoleHandler。
    async fn create_role(
        &self,
        request: Request<CreateRoleRq>,
    ) -> Result<Response<CreateRoleRs>, Status> {
        let req = request.into_inner();
        let nick = req.nick.as_deref().unwrap_or("").to_string();
        let server_id = req.server_id.unwrap_or(1);
        let camp = 1i32; // 默认阵营，客户端不传 camp（由服务端分配）

        info!(nick, server_id, "Creating role");

        let on_time = chrono::Utc::now().timestamp() as i32;

        // TODO: 实现与 Java 版一致的 role_id 生成策略（从 p_server_config 读取 max_key）
        let role_id: i64 = (server_id as i64) * 1_000_000_000
            + rand::thread_rng().gen_range(1..=999_999_999);

        // 插入 p_lord
        sqlx::query(
            "INSERT INTO p_lord (role_id, nick, diamond, gold, meat, stamina, \
             vip_level, vip_exp, camp_id, on_time, ol_time, off_time, \
             total_login, current_streak, battle_fight, fame, pay_amount) \
             VALUES (?, ?, 0, 0, 0, 200, 0, 0, ?, ?, 0, 0, 1, 1, 0, 0, 0)"
        )
        .bind(role_id)
        .bind(&nick)
        .bind(camp)
        .bind(on_time)
        .execute(&self.db)
        .await
        .map_err(|e| Status::internal(format!("Failed to create lord: {}", e)))?;

        // 插入 p_data（全列 NULL）
        sqlx::query("INSERT IGNORE INTO p_data (role_id) VALUES (?)")
            .bind(role_id)
            .execute(&self.db)
            .await
            .map_err(|e| Status::internal(format!("Failed to init p_data: {}", e)))?;

        info!(role_id, nick, "Role created");

        Ok(Response::new(CreateRoleRs {
            state: Some(1),
            ..Default::default()
        }))
    }

    /// RoleLogin：玩家进入游戏，启动 PlayerActor
    async fn role_login(
        &self,
        request: Request<RoleLoginRq>,
    ) -> Result<Response<RoleLoginRs>, Status> {
        // 从 gRPC Metadata 读取 account_id 和 role_id
        let metadata = request.metadata();
        let account_id = metadata
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| Status::unauthenticated("Missing x-account-id metadata"))?;
        let role_id = metadata
            .get("x-role-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| Status::unauthenticated("Missing x-role-id metadata"))?;

        info!(account_id, role_id, "RoleLogin: spawning PlayerActor");

        // 启动（或复用）PlayerActor
        let tx = self.manager.spawn_actor(account_id, role_id);

        // 发送 RoleLogin 消息，等待 Actor 完成登录初始化
        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(PlayerMessage::RoleLogin(reply_tx))
            .map_err(|_| Status::internal("PlayerActor channel closed"))?;

        let rs = reply_rx.await
            .map_err(|_| Status::internal("PlayerActor did not reply"))?
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(rs))
    }

    /// Dispatch：Gateway 转发业务命令到 PlayerActor 处理
    ///
    /// 流程：PlayerManager 查找 Actor sender → 发送 GameCommand → 等待 reply → 返回
    async fn dispatch(
        &self,
        request: Request<DispatchRq>,
    ) -> Result<Response<DispatchRs>, Status> {
        let req = request.into_inner();
        let role_id = req.role_id;
        let cmd = req.cmd as u32;
        let payload = req.payload;

        // 查找 PlayerActor
        let tx = self.manager.get_by_role(role_id).ok_or_else(|| {
            warn!(role_id, cmd, "Dispatch: player not online");
            Status::not_found(format!("Player {} not online", role_id))
        })?;

        // 发送命令，等待响应
        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(PlayerMessage::GameCommand {
            cmd,
            payload,
            reply: reply_tx,
        }).map_err(|_| Status::internal("PlayerActor channel closed"))?;

        let result = reply_rx.await
            .map_err(|_| Status::internal("PlayerActor did not reply"))?;

        match result {
            Ok(resp_payload) => Ok(Response::new(DispatchRs {
                code: 0,
                payload: resp_payload,
            })),
            Err(e) => {
                warn!(role_id, cmd, "Dispatch error: {}", e);
                Ok(Response::new(DispatchRs {
                    code: 1,
                    payload: vec![],
                }))
            }
        }
    }

    /// PlayerOffline：Gateway 断线通知，触发 PlayerActor 存盘下线
    async fn player_offline(
        &self,
        request: Request<PlayerOfflineRq>,
    ) -> Result<Response<PlayerOfflineRs>, Status> {
        let role_id = request.into_inner().role_id;

        if let Some(tx) = self.manager.get_by_role(role_id) {
            info!(role_id, "PlayerOffline: sending Shutdown to actor");
            let _ = tx.send(PlayerMessage::Shutdown);
            // 从管理器移除（account_id 未知，仅按 role_id 清理）
            self.manager.remove_player(0, role_id);
        } else {
            warn!(role_id, "PlayerOffline: player not found (already offline?)");
        }

        Ok(Response::new(PlayerOfflineRs { code: 0 }))
    }
}
