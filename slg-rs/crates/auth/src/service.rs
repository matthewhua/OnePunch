use tonic::{Request, Response, Status};
use proto::slg::auth_service_server::AuthService;
use proto::slg::{LoginRequest, LoginResponse, ValidateTokenRequest, ValidateTokenResponse};
use sqlx::MySqlPool;
use std::sync::Arc;
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
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        
        // 1. 验证账号密码 (简单实现)
        // 注意：生产环境应使用 password hashing (如 argon2)
        let row: Option<(i64,)> = sqlx::query_as("SELECT id FROM p_account WHERE username = ? AND password = ?")
            .bind(&req.username)
            .bind(&req.password)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        match row {
            Some((account_id,)) => {
                // 2. 创建会话
                let token = self.session_mgr.create_session(account_id).await
                    .map_err(|e| Status::internal(format!("Session error: {}", e)))?;
                
                Ok(Response::new(LoginResponse {
                    success: true,
                    token,
                    account_id,
                    error_msg: String::new(),
                }))
            }
            None => {
                Ok(Response::new(LoginResponse {
                    success: false,
                    token: String::new(),
                    account_id: 0,
                    error_msg: "Invalid username or password".to_string(),
                }))
            }
        }
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();
        
        let account_id = self.session_mgr.validate_session(&req.token).await
            .map_err(|e| Status::internal(format!("Session error: {}", e)))?;

        match account_id {
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
