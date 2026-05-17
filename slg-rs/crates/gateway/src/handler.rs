//! 连接处理器
//!
//! 每个 TCP 连接对应一个 `ConnectionHandler` 实例（在独立 tokio task 中运行）。
//! 实现完整的会话状态机和消息路由。
//!
//! # 登录流程
//!
//! ```text
//! Client                    Gateway                   Auth Service
//!   │──DoLoginRq(plat_id)──▶│──gRPC Login(plat_id)──▶│
//!   │                        │◀──LoginResponse─────────│
//!   │◀──DoLoginRs(keyId,tok)─│
//!   │
//!   │──VerifyRq(keyId,token)─▶│──gRPC ValidateToken──▶│
//!   │                          │◀──ValidateTokenRs──────│
//!   │◀──VerifyRs(keyId,platId)─│
//!   │
//!   │──BeginGameRq(serverId)──▶│──gRPC BeginGame──▶ Home Service
//!   │◀──BeginGameRs(state)──────│
//!   │
//!   │──[业务消息]──────────────▶│──路由到 Home/World Service
//! ```

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_util::codec::Framed;
use tracing::{debug, error, info, warn};

use crate::codec::{GameCodec, GamePacket};
use crate::session::{DisconnectNotice, DisconnectTx, SessionState, SessionStore};
use proto::slg::auth_service_client::AuthServiceClient;
use proto::slg::home_service_client::HomeServiceClient;
use proto::slg::world_service_client::WorldServiceClient;
use shared::cmd::{CmdRoute, GameCmd, GameCmdExt};

/// 心跳超时（60 秒无任何包则断开）
const HEARTBEAT_TIMEOUT_SECS: u64 = 60;
/// 最大帧大小（64KB）
const MAX_FRAME_SIZE: usize = 65536;

/// 连接处理器
///
/// 每个连接一个实例，持有共享的 SessionStore 和服务地址。
pub struct ConnectionHandler {
    pub sessions: Arc<SessionStore>,
    pub auth_addr: String,
    pub home_addr: String,
    pub world_addr: String,
    pub disconnect_tx: DisconnectTx,
}

impl ConnectionHandler {
    pub fn new(
        sessions: Arc<SessionStore>,
        auth_addr: String,
        home_addr: String,
        world_addr: String,
        disconnect_tx: DisconnectTx,
    ) -> Self {
        Self {
            sessions,
            auth_addr,
            home_addr,
            world_addr,
            disconnect_tx,
        }
    }

    /// 处理一个 TCP 连接的完整生命周期
    pub async fn handle(&self, stream: TcpStream) -> Result<()> {
        let peer_addr = stream.peer_addr()?;
        let conn_id = self.sessions.alloc_conn_id();
        self.sessions.register(conn_id, peer_addr);
        let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel();
        self.sessions.register_shutdown(conn_id, shutdown_tx);

        info!(conn_id, %peer_addr, "Connection established");

        let result = self.run_connection(conn_id, stream, shutdown_rx).await;

        // 连接结束，清理会话并发送下线通知
        let account_key_id = self.sessions.get_account_key_id(conn_id);
        let role_id = self.sessions.get_role_id(conn_id);
        self.sessions.remove(conn_id);

        if let Some(akid) = account_key_id {
            let _ = self.disconnect_tx.send(DisconnectNotice {
                conn_id,
                account_key_id: akid,
                role_id,
            });
        }

        info!(conn_id, %peer_addr, online = self.sessions.online_count(), "Connection closed");

        result
    }

    async fn run_connection(
        &self,
        conn_id: u64,
        stream: TcpStream,
        mut shutdown_rx: mpsc::UnboundedReceiver<()>,
    ) -> Result<()> {
        let mut framed = Framed::new(stream, GameCodec);

        loop {
            let packet_result = tokio::select! {
                _ = shutdown_rx.recv() => {
                    debug!(conn_id, "Connection shutdown requested");
                    break;
                }
                packet_result = timeout(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS), framed.next()) => {
                    packet_result
                }
            };

            match packet_result {
                Ok(Some(Ok(packet))) => {
                    // 帧大小检查
                    if packet.payload.len() > MAX_FRAME_SIZE {
                        warn!(conn_id, cmd = ?packet.cmd, size = packet.payload.len(), "Frame too large");
                        break;
                    }

                    self.sessions.touch(conn_id);

                    if let Err(e) = self.dispatch(conn_id, packet, &mut framed).await {
                        warn!(conn_id, "Dispatch error: {}", e);
                        break;
                    }
                }
                Ok(Some(Err(e))) => {
                    error!(conn_id, "Protocol error: {}", e);
                    break;
                }
                Ok(None) => {
                    debug!(conn_id, "Connection closed by peer");
                    break;
                }
                Err(_) => {
                    warn!(conn_id, "Heartbeat timeout ({}s)", HEARTBEAT_TIMEOUT_SECS);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 根据会话状态分发消息
    async fn dispatch(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let state = self
            .sessions
            .get_state(conn_id)
            .unwrap_or(SessionState::Disconnecting);

        match state {
            SessionState::Connected => {
                // 未登录：只允许 DoLoginRq
                if packet.cmd == GameCmd::DoLoginRq {
                    self.handle_do_login(conn_id, packet, framed).await
                } else {
                    warn!(conn_id, cmd = ?packet.cmd, "Rejected: not authenticated");
                    let err = shared::msg::GameMessage::build_error(packet.cmd as i32, 401)?;
                    framed.send(GamePacket::new(packet.cmd, err)).await?;
                    Err(anyhow!("Unauthorized command before login"))
                }
            }
            SessionState::Authenticated => {
                // 已登录：只允许 VerifyRq
                if packet.cmd == GameCmd::VerifyRq {
                    self.handle_verify(conn_id, packet, framed).await
                } else {
                    warn!(conn_id, cmd = ?packet.cmd, "Rejected: verify required");
                    Err(anyhow!("VerifyRq required"))
                }
            }
            SessionState::Verified => {
                // 已验证：只允许 BeginGameRq
                if packet.cmd == GameCmd::BeginGameRq {
                    self.handle_begin_game(conn_id, packet, framed).await
                } else {
                    warn!(conn_id, cmd = ?packet.cmd, "Rejected: BeginGame required");
                    Err(anyhow!("BeginGameRq required"))
                }
            }
            SessionState::InGame => {
                // 已进入游戏：按 cmd 路由
                self.handle_in_game(conn_id, packet, framed).await
            }
            SessionState::Disconnecting => Err(anyhow!("Session is disconnecting")),
        }
    }

    // ── 登录流程 ──────────────────────────────────────────────────────────────

    /// 处理 DoLoginRq（ext=103）
    ///
    /// 从 DoLoginRq 提取 plat_id（param[0]）和 device_no，
    /// 调用 Auth Service 验证，返回 DoLoginRs（keyId + token）。
    async fn handle_do_login(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let msg = packet.to_message()?;
        let rq: proto::slg::DoLoginRq = msg.get_payload()?;

        // plat_id 在 param[0]，device_no 在 deviceNo 字段
        let plat_id = rq.param.first().cloned().unwrap_or_default();
        let device_no = rq.device_no.clone();

        debug!(conn_id, plat_id, "DoLogin attempt");

        // 调用 Auth Service
        let mut auth_client = AuthServiceClient::connect(self.auth_addr.clone())
            .await
            .map_err(|e| anyhow!("Auth service unavailable: {}", e))?;

        let login_resp = auth_client
            .login(proto::slg::LoginRequest {
                username: plat_id.clone(),
                password: device_no,
            })
            .await?
            .into_inner();

        if !login_resp.success {
            warn!(conn_id, plat_id, "DoLogin failed: {}", login_resp.error_msg);
            let err = shared::msg::GameMessage::build_error(GameCmd::DoLoginRs as i32, 101)?;
            framed
                .send(GamePacket::new(GameCmd::DoLoginRs, err))
                .await?;
            return Err(anyhow!("Login failed: {}", login_resp.error_msg));
        }

        let account_key_id = login_resp.account_id;
        let token = login_resp.token.clone();

        // 踢掉旧连接（如果有）
        if let Some(old_conn_id) =
            self.sessions
                .bind_account(conn_id, account_key_id, token.clone())
        {
            let marked = self.sessions.mark_disconnecting(old_conn_id);
            warn!(
                conn_id,
                old_conn_id, account_key_id, marked, "Kicking duplicate login"
            );
        }

        // 返回 DoLoginRs
        let rs = proto::slg::DoLoginRs {
            key_id: Some(account_key_id),
            token: Some(token),
            ..Default::default()
        };
        framed
            .send(GamePacket::build_rs(GameCmd::DoLoginRs, &rs)?)
            .await?;

        info!(conn_id, account_key_id, "DoLogin success");
        Ok(())
    }

    /// 处理 VerifyRq（ext=105）
    ///
    /// 验证 token 有效性，确认区服号，返回 VerifyRs。
    async fn handle_verify(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let msg = packet.to_message()?;
        let rq: proto::slg::VerifyRq = msg.get_payload()?;

        let token = rq.token.clone();
        let server_id = rq.server_id;

        debug!(conn_id, server_id, "Verify attempt");

        // 调用 Auth Service 验证 token
        let mut auth_client = AuthServiceClient::connect(self.auth_addr.clone())
            .await
            .map_err(|e| anyhow!("Auth service unavailable: {}", e))?;

        let validate_resp = auth_client
            .validate_token(proto::slg::ValidateTokenRequest {
                token: token.clone(),
            })
            .await?
            .into_inner();

        if !validate_resp.valid {
            warn!(conn_id, "Verify failed: invalid token");
            let err = shared::msg::GameMessage::build_error(GameCmd::VerifyRs as i32, 102)?;
            framed.send(GamePacket::new(GameCmd::VerifyRs, err)).await?;
            return Err(anyhow!("Invalid token"));
        }

        // 更新会话状态
        self.sessions.set_verified(conn_id, server_id);

        // 返回 VerifyRs
        let rs = proto::slg::VerifyRs {
            key_id: Some(validate_resp.account_id),
            server_id: Some(server_id),
            ..Default::default()
        };
        framed
            .send(GamePacket::build_rs(GameCmd::VerifyRs, &rs)?)
            .await?;

        info!(
            conn_id,
            account_key_id = validate_resp.account_id,
            server_id,
            "Verify success"
        );
        Ok(())
    }

    /// 处理 BeginGameRq（ext=1101）
    ///
    /// 查询角色状态，转发到 Home Service，返回 BeginGameRs。
    async fn handle_begin_game(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let msg = packet.to_message()?;
        let rq: proto::slg::BeginGameRq = msg.get_payload()?;

        let account_key_id = self
            .sessions
            .get_account_key_id(conn_id)
            .ok_or_else(|| anyhow!("No account_key_id in session"))?;
        let server_id = self.sessions.get_server_id(conn_id).unwrap_or(rq.server_id);

        debug!(conn_id, account_key_id, server_id, "BeginGame");

        // 转发到 Home Service
        let mut home_client = HomeServiceClient::connect(self.home_addr.clone())
            .await
            .map_err(|e| anyhow!("Home service unavailable: {}", e))?;

        let begin_rq = proto::slg::BeginGameRq {
            server_id,
            key_id: account_key_id,
            token: rq.token.clone(),
            device_no: rq.device_no.clone(),
            cur_version: rq.cur_version.clone(),
            ..Default::default()
        };

        let rs = home_client.begin_game(begin_rq).await?.into_inner();

        // 如果已有角色，更新 role_id
        if let Some(role_id) = rs.role_id {
            self.sessions.set_in_game(conn_id, role_id);
            info!(
                conn_id,
                account_key_id,
                role_id,
                state = rs.state,
                "BeginGame: role exists"
            );
        } else {
            // 未创建角色，仍进入 InGame 状态（等待 CreateRole）
            self.sessions.set_in_game(conn_id, 0);
            info!(conn_id, account_key_id, "BeginGame: no role yet");
        }

        framed
            .send(GamePacket::build_rs(GameCmd::BeginGameRs, &rs)?)
            .await?;
        Ok(())
    }

    // ── 游戏内消息路由 ────────────────────────────────────────────────────────

    /// 处理已登录玩家的业务消息
    async fn handle_in_game(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let cmd = packet.cmd;

        // 心跳特殊处理（直接回包，不转发后端）
        if cmd == GameCmd::HeartbeatRq {
            return self.handle_heartbeat(conn_id, framed).await;
        }
        if cmd == GameCmd::CreateRoleRq {
            return self.handle_create_role(conn_id, packet, framed).await;
        }

        // 按 cmd 路由
        match cmd.route() {
            CmdRoute::Auth => {
                // Auth 命令在 InGame 阶段不应出现，忽略
                warn!(conn_id, cmd = ?cmd, "Unexpected Auth cmd in InGame state");
                Ok(())
            }
            CmdRoute::Home => self.forward_to_home(conn_id, packet, framed).await,
            CmdRoute::World => self.forward_to_world(conn_id, packet, framed).await,
        }
    }

    /// 处理心跳（HeartbeatRq ext=1115 → HeartbeatRs ext=1116）
    async fn handle_heartbeat(
        &self,
        conn_id: u64,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let rs = proto::slg::HeartbeatRs {
            time: Some(chrono::Utc::now().timestamp() as i32),
        };
        framed
            .send(GamePacket::build_rs(GameCmd::HeartbeatRs, &rs)?)
            .await?;
        debug!(conn_id, "Heartbeat");
        Ok(())
    }

    /// 处理 CreateRoleRq。创建成功后立即重新 BeginGame，同步 session 中的 role_id。
    async fn handle_create_role(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let msg = packet.to_message()?;
        let rq: proto::slg::CreateRoleRq = msg.get_payload()?;
        let account_key_id = self
            .sessions
            .get_account_key_id(conn_id)
            .ok_or_else(|| anyhow!("No account_key_id in session"))?;
        let server_id = self
            .sessions
            .get_server_id(conn_id)
            .unwrap_or_else(|| rq.server_id.unwrap_or(1));

        let mut home_client = HomeServiceClient::connect(self.home_addr.clone())
            .await
            .map_err(|e| anyhow!("Home service unavailable: {}", e))?;

        let mut request = tonic::Request::new(proto::slg::CreateRoleRq {
            server_id: Some(server_id),
            nick: rq.nick.clone(),
            ..Default::default()
        });
        request.metadata_mut().insert(
            "x-account-id",
            account_key_id
                .to_string()
                .parse()
                .map_err(|e| anyhow!("invalid x-account-id metadata: {}", e))?,
        );

        let rs = home_client.create_role(request).await?.into_inner();
        if rs.state == Some(1) || rs.state == Some(2) {
            let begin_rs = home_client
                .begin_game(proto::slg::BeginGameRq {
                    server_id,
                    key_id: account_key_id,
                    ..Default::default()
                })
                .await?
                .into_inner();
            if let Some(role_id) = begin_rs.role_id {
                self.sessions.set_in_game(conn_id, role_id);
                info!(
                    conn_id,
                    account_key_id, role_id, "CreateRole: role is ready"
                );
            }
        }

        framed
            .send(GamePacket::build_rs(GameCmd::CreateRoleRs, &rs)?)
            .await?;
        Ok(())
    }

    /// 转发消息到 Home Service（gRPC Dispatch）
    ///
    /// 流程：
    /// 1. 从 session 取 role_id
    /// 2. 调用 HomeService::Dispatch(DispatchRq { role_id, cmd, payload })
    /// 3. 将响应 payload 封装为 GamePacket 发回客户端
    async fn forward_to_home(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let role_id = self
            .sessions
            .get_role_id(conn_id)
            .filter(|&id| id > 0)
            .ok_or_else(|| anyhow!("No role_id in session for conn {}", conn_id))?;

        let cmd = packet.cmd;
        let payload = packet.payload.clone();

        debug!(conn_id, role_id, cmd = ?cmd, "Forward to Home via gRPC Dispatch");

        // 建立 gRPC 连接（TODO: 连接池复用）
        let mut home_client = HomeServiceClient::connect(self.home_addr.clone())
            .await
            .map_err(|e| anyhow!("Home service unavailable: {}", e))?;

        let dispatch_rq = proto::slg::DispatchRq {
            role_id,
            cmd: cmd as i32,
            payload,
        };

        let rs = home_client
            .dispatch(dispatch_rq)
            .await
            .map_err(|s| anyhow!("Dispatch gRPC error: {}", s))?
            .into_inner();

        if rs.code != 0 {
            warn!(conn_id, role_id, cmd = ?cmd, code = rs.code, "Dispatch returned error code");
            // 将错误码封装为 GameMessage 错误包发回客户端
            let err_payload = shared::msg::GameMessage::build_error(cmd as i32, rs.code)?;
            // 响应 cmd = 请求 cmd + 1（约定：Rq 为奇数，Rs 为偶数）
            let rs_cmd = GameCmd::from(cmd as u32 + 1);
            framed.send(GamePacket::new(rs_cmd, err_payload)).await?;
            return Ok(());
        }

        // 将响应 payload 直接封装为 GamePacket 发回客户端
        // 响应 cmd = 请求 cmd + 1
        let rs_cmd = GameCmd::from(cmd as u32 + 1);
        framed.send(GamePacket::new(rs_cmd, rs.payload)).await?;

        Ok(())
    }

    /// 转发消息到 World Service（gRPC Dispatch）。
    async fn forward_to_world(
        &self,
        conn_id: u64,
        packet: GamePacket,
        framed: &mut Framed<TcpStream, GameCodec>,
    ) -> Result<()> {
        let role_id = self
            .sessions
            .get_role_id(conn_id)
            .filter(|&id| id > 0)
            .ok_or_else(|| anyhow!("No role_id in session for conn {}", conn_id))?;

        let cmd = packet.cmd;
        debug!(conn_id, role_id, cmd = ?cmd, "Forward to World via gRPC Dispatch");

        let mut world_client = WorldServiceClient::connect(self.world_addr.clone())
            .await
            .map_err(|e| anyhow!("World service unavailable: {}", e))?;

        let rs = world_client
            .dispatch(proto::slg::DispatchRq {
                role_id,
                cmd: cmd as i32,
                payload: packet.payload,
            })
            .await
            .map_err(|s| anyhow!("World Dispatch gRPC error: {}", s))?
            .into_inner();

        let rs_cmd = GameCmd::from(cmd as u32 + 1);
        if rs.code != 0 {
            warn!(conn_id, role_id, cmd = ?cmd, code = rs.code, "World dispatch returned error");
            let err_payload = shared::msg::GameMessage::build_error(cmd as i32, rs.code)?;
            framed.send(GamePacket::new(rs_cmd, err_payload)).await?;
            return Ok(());
        }

        framed.send(GamePacket::new(rs_cmd, rs.payload)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::auth_service_server::{AuthService, AuthServiceServer};
    use proto::slg::home_service_client::HomeServiceClient;
    use proto::slg::home_service_server::{HomeService, HomeServiceServer};
    use proto::slg::{
        BeginGameRq, BeginGameRs, CreateRoleRq, CreateRoleRs, DispatchRq, DispatchRs, DoLoginRq,
        GetRoleDataRs, LoginRequest, LoginResponse, PlayerOfflineRq, PlayerOfflineRs, RoleLoginRq,
        RoleLoginRs, ValidateTokenRequest, ValidateTokenResponse, VerifyRq, WorldOutboundRq,
        WorldOutboundRs,
    };
    use shared::msg::GameMessage;
    use std::sync::Mutex;
    use tokio::net::TcpListener;
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;
    use tonic::{Request, Response, Status};

    const ACCOUNT_ID: i64 = 77_001;
    const ROLE_ID: i64 = 88_001;

    #[derive(Default)]
    struct MockAuthService;

    #[tonic::async_trait]
    impl AuthService for MockAuthService {
        async fn login(
            &self,
            _request: Request<LoginRequest>,
        ) -> Result<Response<LoginResponse>, Status> {
            Ok(Response::new(LoginResponse {
                success: true,
                token: "token-77001".to_string(),
                account_id: ACCOUNT_ID,
                error_msg: String::new(),
            }))
        }

        async fn validate_token(
            &self,
            _request: Request<ValidateTokenRequest>,
        ) -> Result<Response<ValidateTokenResponse>, Status> {
            Ok(Response::new(ValidateTokenResponse {
                valid: true,
                account_id: ACCOUNT_ID,
            }))
        }
    }

    #[derive(Default)]
    struct MockHomeService {
        offline_roles: Arc<Mutex<Vec<i64>>>,
    }

    #[tonic::async_trait]
    impl HomeService for MockHomeService {
        async fn begin_game(
            &self,
            _request: Request<BeginGameRq>,
        ) -> Result<Response<BeginGameRs>, Status> {
            Ok(Response::new(BeginGameRs {
                state: Some(2),
                role_id: Some(ROLE_ID),
                camp: Some(1),
                ..Default::default()
            }))
        }

        async fn create_role(
            &self,
            _request: Request<CreateRoleRq>,
        ) -> Result<Response<CreateRoleRs>, Status> {
            Ok(Response::new(CreateRoleRs::default()))
        }

        async fn role_login(
            &self,
            _request: Request<RoleLoginRq>,
        ) -> Result<Response<RoleLoginRs>, Status> {
            Ok(Response::new(RoleLoginRs {
                state: Some(1),
                ..Default::default()
            }))
        }

        async fn dispatch(
            &self,
            _request: Request<DispatchRq>,
        ) -> Result<Response<DispatchRs>, Status> {
            Ok(Response::new(DispatchRs {
                code: 0,
                payload: GameMessage::build_response(1110, &GetRoleDataRs::default()).unwrap(),
            }))
        }

        async fn world_outbound(
            &self,
            _request: Request<WorldOutboundRq>,
        ) -> Result<Response<WorldOutboundRs>, Status> {
            Ok(Response::new(WorldOutboundRs {
                code: 0,
                msg: String::new(),
            }))
        }

        async fn player_offline(
            &self,
            request: Request<PlayerOfflineRq>,
        ) -> Result<Response<PlayerOfflineRs>, Status> {
            self.offline_roles
                .lock()
                .unwrap()
                .push(request.into_inner().role_id);
            Ok(Response::new(PlayerOfflineRs { code: 0 }))
        }
    }

    async fn spawn_auth_service() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            Server::builder()
                .add_service(AuthServiceServer::new(MockAuthService))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        });
        format!("http://{}", addr)
    }

    async fn spawn_home_service(offline_roles: Arc<Mutex<Vec<i64>>>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            Server::builder()
                .add_service(HomeServiceServer::new(MockHomeService { offline_roles }))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        });
        format!("http://{}", addr)
    }

    async fn send_and_recv<T: prost::Message>(
        framed: &mut Framed<TcpStream, GameCodec>,
        cmd: GameCmd,
        body: &T,
    ) -> GamePacket {
        let payload = GameMessage::build_response(cmd as i32, body).unwrap();
        framed.send(GamePacket::new(cmd, payload)).await.unwrap();
        framed.next().await.unwrap().unwrap()
    }

    async fn connect_through_begin_game(
        gateway_addr: std::net::SocketAddr,
    ) -> Framed<TcpStream, GameCodec> {
        let stream = TcpStream::connect(gateway_addr).await.unwrap();
        let mut framed = Framed::new(stream, GameCodec);

        let mut login = DoLoginRq::default();
        login.sid = "sid".to_string();
        login.base_version = "1".to_string();
        login.version = "1".to_string();
        login.device_no = "device".to_string();
        login.plat = "test".to_string();
        login.param = vec!["plat-account".to_string()];
        let login_packet = send_and_recv(&mut framed, GameCmd::DoLoginRq, &login).await;
        assert_eq!(login_packet.cmd, GameCmd::DoLoginRs);

        let mut verify = VerifyRq::default();
        verify.key_id = ACCOUNT_ID;
        verify.server_id = 1;
        verify.token = "token-77001".to_string();
        verify.cur_version = "1".to_string();
        verify.device_no = "device".to_string();
        verify.channel_id = 1;
        let verify_packet = send_and_recv(&mut framed, GameCmd::VerifyRq, &verify).await;
        assert_eq!(verify_packet.cmd, GameCmd::VerifyRs);

        let mut begin = BeginGameRq::default();
        begin.server_id = 1;
        begin.key_id = ACCOUNT_ID;
        begin.token = "token-77001".to_string();
        begin.device_no = "device".to_string();
        begin.cur_version = "1".to_string();
        let begin_packet = send_and_recv(&mut framed, GameCmd::BeginGameRq, &begin).await;
        assert_eq!(begin_packet.cmd, GameCmd::BeginGameRs);
        assert_eq!(
            begin_packet
                .to_message()
                .unwrap()
                .get_payload::<BeginGameRs>()
                .unwrap()
                .role_id,
            Some(ROLE_ID)
        );

        framed
    }

    #[tokio::test]
    async fn duplicate_login_disconnects_old_session_and_notifies_home_offline() {
        let auth_addr = spawn_auth_service().await;
        let offline_roles = Arc::new(Mutex::new(Vec::new()));
        let home_addr = spawn_home_service(offline_roles.clone()).await;

        let sessions = SessionStore::new();
        let (disconnect_tx, mut disconnect_rx) = crate::session::disconnect_channel();
        let home_addr_for_offline = home_addr.clone();
        tokio::spawn(async move {
            while let Some(notice) = disconnect_rx.recv().await {
                if let Some(role_id) = notice.role_id.filter(|id| *id > 0) {
                    let mut client = HomeServiceClient::connect(home_addr_for_offline.clone())
                        .await
                        .unwrap();
                    let _ = client.player_offline(PlayerOfflineRq { role_id }).await;
                }
            }
        });

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let gateway_addr = listener.local_addr().unwrap();
        let handler = Arc::new(ConnectionHandler::new(
            sessions,
            auth_addr,
            home_addr,
            "http://127.0.0.1:9".to_string(),
            disconnect_tx,
        ));
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let handler = handler.clone();
                tokio::spawn(async move {
                    let _ = handler.handle(stream).await;
                });
            }
        });

        let _old_client = connect_through_begin_game(gateway_addr).await;
        let _new_client = connect_through_begin_game(gateway_addr).await;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            if offline_roles.lock().unwrap().contains(&ROLE_ID) {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "duplicate login did not notify Home PlayerOffline for role {}",
                ROLE_ID
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
