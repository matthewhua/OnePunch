use tonic::{Request, Response, Status};
use proto::slg::home_service_server::HomeService;
use proto::slg::{BeginGameRq, BeginGameRs, CreateRoleRq, CreateRoleRs, RoleLoginRq, RoleLoginRs};
use sqlx::MySqlPool;
use std::sync::Arc;
use crate::managers::player_manager::PlayerManager;
use rand::{distributions::Alphanumeric, Rng};
use tracing::info;

pub struct HomeServiceImpl {
    db: MySqlPool,
    _manager: Arc<PlayerManager>,
}

impl HomeServiceImpl {
    pub fn new(db: MySqlPool, manager: Arc<PlayerManager>) -> Self {
        Self { db, _manager: manager }
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

        // 生成 role_id（Java 版格式：server_id * 10^9 + 自增序列）
        // 简化：使用 p_lord 自增 + server_id 前缀
        // 实际应从 p_server_config 读取 max_key 并原子递增
        let on_time = chrono::Utc::now().timestamp() as i32;

        // 1. 插入 p_lord（role_id 由 Java 侧生成，这里用 server_id * 1e9 + 随机）
        // TODO: 实现与 Java 版一致的 role_id 生成策略
        let role_id: i64 = (server_id as i64) * 1_000_000_000
            + rand::thread_rng().gen_range(1..=999_999_999);

        // 2. 插入 p_lord
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

        // 3. 插入 p_data（全列 NULL）
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
        _request: Request<RoleLoginRq>,
    ) -> Result<Response<RoleLoginRs>, Status> {
        // TODO: 从 gRPC Metadata 获取 account_id 和 role_id，启动 PlayerActor
        Ok(Response::new(RoleLoginRs {
            state: Some(1),
            ..Default::default()
        }))
    }
}
