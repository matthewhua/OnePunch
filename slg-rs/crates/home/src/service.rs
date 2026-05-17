use crate::actors::player_actor::PlayerMessage;
use crate::managers::player_manager::PlayerManager;
use proto::slg::home_service_server::HomeService;
use proto::slg::{
    BeginGameRq, BeginGameRs, CreateRoleRq, CreateRoleRs, DispatchRq, DispatchRs, PlayerOfflineRq,
    PlayerOfflineRs, RoleLoginRq, RoleLoginRs, WorldOutboundRq, WorldOutboundRs,
};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::MySqlPool;
use std::sync::Arc;
use tokio::sync::oneshot;
use tonic::{Request, Response, Status};
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

    async fn get_account_key_id_by_role(&self, role_id: i64) -> Result<i64, Status> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT account_key_id FROM p_account WHERE role_id = ? LIMIT 1")
                .bind(role_id)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

        row.map(|v| v.0)
            .ok_or_else(|| Status::not_found(format!("account not found by role_id={}", role_id)))
    }

    async fn allocate_role_id(&self, server_id: i32) -> Result<i64, Status> {
        let mut tx =
            self.db.begin().await.map_err(|e| {
                Status::internal(format!("begin role id transaction failed: {}", e))
            })?;

        let current: Option<(i32, String)> = sqlx::query_as(
            "SELECT param_id, param_value FROM p_server_config \
             WHERE param_name = 'maxRoleKey' ORDER BY param_id LIMIT 1 FOR UPDATE",
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| Status::internal(format!("lock maxRoleKey failed: {}", e)))?;

        let next_key = match current {
            Some((param_id, value)) => {
                let next = value.parse::<i64>().unwrap_or(0) + 1;
                sqlx::query("UPDATE p_server_config SET param_value = ? WHERE param_id = ?")
                    .bind(next.to_string())
                    .bind(param_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Status::internal(format!("update maxRoleKey failed: {}", e)))?;
                next
            }
            None => {
                sqlx::query(
                    "INSERT INTO p_server_config (title, param_name, param_value, descs) \
                     VALUES ('最大角色ID', 'maxRoleKey', '1', '当前区服已分配的最大角色序号')",
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| Status::internal(format!("init maxRoleKey failed: {}", e)))?;
                1
            }
        };

        tx.commit()
            .await
            .map_err(|e| Status::internal(format!("commit role id transaction failed: {}", e)))?;

        Ok((server_id as i64) * 1_000_000_000 + next_key)
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
            "SELECT role_id, 0 FROM p_account WHERE account_key_id = ? AND server_id = ?",
        )
        .bind(account_key_id)
        .bind(server_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let mut res = BeginGameRs::default();
        res.time = Some(chrono::Utc::now().timestamp() as i32);

        match account.filter(|(role_id, _)| *role_id > 0) {
            Some((role_id, _)) => {
                // 已有角色，查询 p_lord 获取 camp
                let lord: Option<(i32,)> =
                    sqlx::query_as("SELECT camp_id FROM p_lord WHERE role_id = ?")
                        .bind(role_id)
                        .fetch_optional(&self.db)
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?;

                res.state = Some(2); // 已创建角色
                res.role_id = Some(role_id);
                res.camp = lord.map(|(c,)| c).or(Some(0));

                // 已有角色时，直接拉起在线 Actor，保证后续 Dispatch 可达。
                let tx = self.manager.spawn_actor(account_key_id, role_id);
                let (reply_tx, reply_rx) = oneshot::channel();
                tx.send(PlayerMessage::RoleLogin(reply_tx)).map_err(|_| {
                    Status::internal("PlayerActor channel closed during begin_game")
                })?;
                reply_rx
                    .await
                    .map_err(|_| Status::internal("PlayerActor did not reply during begin_game"))?
                    .map_err(|e| Status::internal(e.to_string()))?;
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
        let account_key_id = request
            .metadata()
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| Status::unauthenticated("Missing x-account-id metadata"))?;
        let req = request.into_inner();
        let nick = req.nick.as_deref().unwrap_or("").to_string();
        let server_id = req.server_id.unwrap_or(1);
        let camp = 1i32; // 默认阵营，客户端不传 camp（由服务端分配）

        info!(account_key_id, nick, server_id, "Creating role");

        let existing: Option<(i64,)> = sqlx::query_as(
            "SELECT role_id FROM p_account WHERE account_key_id = ? AND server_id = ? LIMIT 1",
        )
        .bind(account_key_id)
        .bind(server_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Status::internal(format!("Failed to query existing role: {}", e)))?;

        if let Some((role_id,)) = existing {
            if role_id > 0 {
                let tx = self.manager.spawn_actor(account_key_id, role_id);
                let (reply_tx, reply_rx) = oneshot::channel();
                tx.send(PlayerMessage::RoleLogin(reply_tx)).map_err(|_| {
                    Status::internal("PlayerActor channel closed during create_role")
                })?;
                reply_rx
                    .await
                    .map_err(|_| Status::internal("PlayerActor did not reply during create_role"))?
                    .map_err(|e| Status::internal(e.to_string()))?;
                return Ok(Response::new(CreateRoleRs {
                    state: Some(2),
                    ..Default::default()
                }));
            }
        }
        let has_account_row = existing.is_some();

        let on_time = chrono::Utc::now().timestamp() as i32;
        let role_id = self.allocate_role_id(server_id).await?;
        let plat_id = account_key_id.to_string();

        // 插入 p_lord
        let mut tx = self.db.begin().await.map_err(|e| {
            Status::internal(format!("begin create_role transaction failed: {}", e))
        })?;

        sqlx::query(
            "INSERT INTO p_lord (role_id, nick, diamond, gold, meat, stamina, \
             vip_level, vip_exp, camp_id, on_time, ol_time, off_time, \
             total_login, current_streak, battle_fight, fame, pay_amount) \
             VALUES (?, ?, 0, 0, 0, 200, 0, 0, ?, ?, 0, 0, 1, 1, 0, 0, 0)",
        )
        .bind(role_id)
        .bind(&nick)
        .bind(camp)
        .bind(on_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| Status::internal(format!("Failed to create lord: {}", e)))?;

        // 插入 p_data（全列 NULL）
        sqlx::query("INSERT IGNORE INTO p_data (role_id) VALUES (?)")
            .bind(role_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Status::internal(format!("Failed to init p_data: {}", e)))?;

        if has_account_row {
            sqlx::query(
                "UPDATE p_account SET role_id = ?, created = 1, plat_id = ?, \
                 login_date = NOW(), log_off = 0 \
                 WHERE account_key_id = ? AND server_id = ?",
            )
            .bind(role_id)
            .bind(&plat_id)
            .bind(account_key_id)
            .bind(server_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Status::internal(format!("Failed to update account binding: {}", e)))?;
        } else {
            sqlx::query(
                "INSERT INTO p_account \
                 (account_key_id, server_id, plat_no, plat_id, role_id, created, \
                  login_days, create_date, login_date, log_off) \
                 VALUES (?, ?, 1, ?, ?, 1, 1, NOW(), NOW(), 0)",
            )
            .bind(account_key_id)
            .bind(server_id)
            .bind(&plat_id)
            .bind(role_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Status::internal(format!("Failed to create account binding: {}", e)))?;
        }

        tx.commit().await.map_err(|e| {
            Status::internal(format!("commit create_role transaction failed: {}", e))
        })?;

        let tx = self.manager.spawn_actor(account_key_id, role_id);
        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(PlayerMessage::RoleLogin(reply_tx))
            .map_err(|_| Status::internal("PlayerActor channel closed after create_role"))?;
        reply_rx
            .await
            .map_err(|_| Status::internal("PlayerActor did not reply after create_role"))?
            .map_err(|e| Status::internal(e.to_string()))?;

        info!(account_key_id, role_id, nick, "Role created");

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

        let rs = reply_rx
            .await
            .map_err(|_| Status::internal("PlayerActor did not reply"))?
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(rs))
    }

    /// Dispatch：Gateway 转发业务命令到 PlayerActor 处理
    ///
    /// 流程：PlayerManager 查找 Actor sender → 发送 GameCommand → 等待 reply → 返回
    async fn dispatch(&self, request: Request<DispatchRq>) -> Result<Response<DispatchRs>, Status> {
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
        })
        .map_err(|_| Status::internal("PlayerActor channel closed"))?;

        let result = reply_rx
            .await
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

    /// WorldOutbound：World 服务投递到达/出站事件到在线 PlayerActor。
    async fn world_outbound(
        &self,
        request: Request<WorldOutboundRq>,
    ) -> Result<Response<WorldOutboundRs>, Status> {
        let req = request.into_inner();
        let role_id = req.role_id;

        if role_id <= 0 {
            warn!(
                role_id,
                event_type = req.event_type,
                troop_key = req.troop_key,
                "WorldOutbound: invalid role_id"
            );
            return Ok(Response::new(WorldOutboundRs {
                code: 400,
                msg: format!("invalid role_id={}", role_id),
            }));
        }

        let tx = match self.manager.get_by_role(role_id) {
            Some(tx) => tx,
            None => {
                warn!(
                    role_id,
                    event_type = req.event_type,
                    troop_key = req.troop_key,
                    world_entity_id = req.world_entity_id,
                    "WorldOutbound: player not online"
                );
                return Ok(Response::new(WorldOutboundRs {
                    code: 404,
                    msg: format!("Player {} not online", role_id),
                }));
            }
        };

        let event_type = req.event_type;
        let troop_key = req.troop_key;
        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(PlayerMessage::WorldOutbound {
            event: req,
            reply: reply_tx,
        })
        .map_err(|_| Status::internal("PlayerActor channel closed during world_outbound"))?;

        let result = reply_rx
            .await
            .map_err(|_| Status::internal("PlayerActor did not reply during world_outbound"))?;

        match result {
            Ok(rs) => Ok(Response::new(rs)),
            Err(e) => {
                warn!(
                    role_id,
                    event_type, troop_key, "WorldOutbound actor error: {}", e
                );
                Ok(Response::new(WorldOutboundRs {
                    code: 500,
                    msg: e.to_string(),
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
        let account_id = self.get_account_key_id_by_role(role_id).await.unwrap_or(0);

        if let Some(tx) = self.manager.get_by_role(role_id) {
            info!(role_id, "PlayerOffline: sending Shutdown to actor");
            let _ = tx.send(PlayerMessage::Shutdown);
            self.manager.remove_player(account_id, role_id);
        } else {
            warn!(
                role_id,
                "PlayerOffline: player not found (already offline?)"
            );
        }

        Ok(Response::new(PlayerOfflineRs { code: 0 }))
    }
}
