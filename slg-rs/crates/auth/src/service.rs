use tonic::{Request, Response, Status};
use proto::slg::auth_service_server::AuthService;
use proto::slg::{LoginRequest, LoginResponse, ValidateTokenRequest, ValidateTokenResponse};
use sqlx::MySqlPool;
use std::sync::Arc;
use tracing::{info, warn};
use crate::session::SessionManager;

pub struct AuthServiceImpl {
    db: MySqlPool,
    session_mgr: Arc<SessionManager>,
}

impl AuthServiceImpl {
    pub fn new(db: MySqlPool, session_mgr: Arc<SessionManager>) -> Self {
        Self { db, session_mgr }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    /// 登录验证
    ///
    /// 对应 Java 版 DoLoginHandler / VerifyHandler 流程：
    /// 1. 根据 plat_id（渠道账号ID）查询 p_account
    /// 2. 检查封号状态
    /// 3. 生成 session token 存入 Redis
    /// 4. 返回 account_key_id 和 token
    ///
    /// 注意：当前 LoginRequest 使用 username 字段传 plat_id，
    /// password 字段传 device_no（与 Java 版 DoLoginRq 对应）。
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let plat_id = &req.username;   // 渠道账号ID（plat_id）
        let device_no = &req.password; // 设备号（device_no）

        // 1. 查询 p_account（按 plat_id 查找）
        let row: Option<(i64, i64, i32, i32)> = sqlx::query_as(
            "SELECT account_key_id, role_id, forbid, created \
             FROM p_account WHERE plat_id = ? LIMIT 1"
        )
        .bind(plat_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Status::internal(format!("DB error: {}", e)))?;

        match row {
            Some((account_key_id, role_id, forbid, created)) => {
                // 2. 封号检查
                if forbid != 0 {
                    warn!(account_key_id, "Login rejected: account is banned");
                    return Ok(Response::new(LoginResponse {
                        success: false,
                        token: String::new(),
                        account_id: 0,
                        error_msg: "Account is banned".to_string(),
                    }));
                }

                // 3. 更新设备号和登录时间
                if !device_no.is_empty() {
                    let _ = sqlx::query(
                        "UPDATE p_account SET device_no = ?, login_date = NOW() \
                         WHERE account_key_id = ?"
                    )
                    .bind(device_no)
                    .bind(account_key_id)
                    .execute(&self.db)
                    .await;
                }

                // 4. 生成 session token
                let token = self.session_mgr.create_session(account_key_id).await
                    .map_err(|e| Status::internal(format!("Session error: {}", e)))?;

                info!(account_key_id, role_id, created, "Login success");

                Ok(Response::new(LoginResponse {
                    success: true,
                    token,
                    account_id: account_key_id,
                    error_msg: String::new(),
                }))
            }
            None => {
                // 账号不存在 — 返回失败，由 Gateway 层决定是否引导注册
                // 注意：Java 版在账号服（Center）处理注册，游戏服不直接创建账号
                warn!(plat_id, "Login failed: account not found");
                Ok(Response::new(LoginResponse {
                    success: false,
                    token: String::new(),
                    account_id: 0,
                    error_msg: "Account not found".to_string(),
                }))
            }
        }
    }

    /// 验证 Token
    ///
    /// Gateway 在每次转发请求前调用，验证 session 有效性。
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        let account_key_id = self.session_mgr.validate_session(&req.token).await
            .map_err(|e| Status::internal(format!("Session error: {}", e)))?;

        match account_key_id {
            Some(id) => Ok(Response::new(ValidateTokenResponse {
                valid: true,
                account_id: id,
            })),
            None => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                account_id: 0,
            })),
        }
    }
}
