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

use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, error, debug};
use anyhow::{Result, anyhow};

use crate::codec::{GameCodec, GamePacket};
use crate::session::{SessionStore, SessionState, DisconnectTx, DisconnectNotice};
use shared::cmd::{GameCmd, GameCmdExt, CmdRoute};
use proto::slg::auth_service_client::AuthServiceClient;
use proto::slg::home_service_client::HomeServiceClient;

/// 心跳超时（60 秒无任何包则断开）
const HEARTBEAT_TIMEOUT_SECS: u64 = 60;
/// 最大帧大小（64KB）
const MAX_FRAME_SIZE: usize = 65536;
// World commands cannot yet be translated into `WorldService::Call(RpcMsg)`.
// `RpcMsg` only has proto2 extension space for 10001..=40000, while World command ids
// start at 50001 and the Gateway only holds raw `GameMessage` bytes.
const WORLD_ROUTE_UNIMPLEMENTED_CODE: i32 = 501;

/// 连接处理器
///
/// 每个连接一个实例，持有共享的 SessionStore 和服务地址。
pub struct ConnectionHandler {
    pub sessions: Arc<SessionStore>,
    pub auth_addr: String,
    pub home_addr: String,
    pub disconnect_tx: DisconnectTx,
}

impl ConnectionHandler {
    pub fn new(
        sessions: Arc<SessionStore>,
        auth_addr: String,
        home_addr: String,
        disconnect_tx: DisconnectTx,
    ) -> Self {
        Self { sessions, auth_addr, home_addr, disconnect_tx }
    }

    /// 处理一个 TCP 连接的完整生命周期
    pub async fn handle(&self, stream: TcpStream) -> Result<()> {
        let peer_addr = stream.peer_addr()?;
        let conn_id = self.sessions.alloc_conn_id();
        self.sessions.register(conn_id, peer_addr);

        info!(conn_id, %peer_addr, "Connection established");

        let result = self.run_connection(conn_id, stream).await;

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

    async fn run_connection(&self, conn_id: u64, stream: TcpStream) -> Result<()> {
        let mut framed = Framed::new(stream, GameCodec);

        loop {
            let packet_result = timeout(
                Duration::from_secs(HEARTBEAT_TIMEOUT_SECS),
                framed.next(),
            ).await;

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
        let state = self.sessions.get_state(conn_id)
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
            SessionState::Disconnecting => {
                Err(anyhow!("Session is disconnecting"))
            }
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
        let mut auth_client = AuthServiceClient::connect(self.auth_addr.clone()).await
            .map_err(|e| anyhow!("Auth service unavailable: {}", e))?;

        let login_resp = auth_client.login(proto::slg::LoginRequest {
            username: plat_id.clone(),
            password: device_no,
        }).await?.into_inner();

        if !login_resp.success {
            warn!(conn_id, plat_id, "DoLogin failed: {}", login_resp.error_msg);
            let err = shared::msg::GameMessage::build_error(GameCmd::DoLoginRs as i32, 101)?;
            framed.send(GamePacket::new(GameCmd::DoLoginRs, err)).await?;
            return Err(anyhow!("Login failed: {}", login_resp.error_msg));
        }

        let account_key_id = login_resp.account_id;
        let token = login_resp.token.clone();

        // 踢掉旧连接（如果有）
        if let Some(old_conn_id) = self.sessions.bind_account(conn_id, account_key_id, token.clone()) {
            warn!(conn_id, old_conn_id, account_key_id, "Kicking duplicate login");
            // 旧连接会在下次 dispatch 时因 session 被覆盖而断开
        }

        // 返回 DoLoginRs
        let rs = proto::slg::DoLoginRs {
            key_id: Some(account_key_id),
            token: Some(token),
            ..Default::default()
        };
        framed.send(GamePacket::build_rs(GameCmd::DoLoginRs, &rs)?).await?;

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
        let mut auth_client = AuthServiceClient::connect(self.auth_addr.clone()).await
            .map_err(|e| anyhow!("Auth service unavailable: {}", e))?;

        let validate_resp = auth_client.validate_token(proto::slg::ValidateTokenRequest {
            token: token.clone(),
        }).await?.into_inner();

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
        framed.send(GamePacket::build_rs(GameCmd::VerifyRs, &rs)?).await?;

        info!(conn_id, account_key_id = validate_resp.account_id, server_id, "Verify success");
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

        let account_key_id = self.sessions.get_account_key_id(conn_id)
            .ok_or_else(|| anyhow!("No account_key_id in session"))?;
        let server_id = self.sessions.get_server_id(conn_id)
            .unwrap_or(rq.server_id);

        debug!(conn_id, account_key_id, server_id, "BeginGame");

        // 转发到 Home Service
        let mut home_client = HomeServiceClient::connect(self.home_addr.clone()).await
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
            info!(conn_id, account_key_id, role_id, state = rs.state, "BeginGame: role exists");
        } else {
            // 未创建角色，仍进入 InGame 状态（等待 CreateRole）
            self.sessions.set_in_game(conn_id, 0);
            info!(conn_id, account_key_id, "BeginGame: no role yet");
        }

        framed.send(GamePacket::build_rs(GameCmd::BeginGameRs, &rs)?).await?;
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

        // 按 cmd 路由
        match cmd.route() {
            CmdRoute::Auth => {
                // Auth 命令在 InGame 阶段不应出现，忽略
                warn!(conn_id, cmd = ?cmd, "Unexpected Auth cmd in InGame state");
                Ok(())
            }
            CmdRoute::Home => {
                self.forward_to_home(conn_id, packet, framed).await
            }
            CmdRoute::World => {
                let response = build_world_unimplemented_response(cmd)?;
                warn!(
                    conn_id,
                    cmd = ?cmd,
                    "World cmd reached Gateway, but raw packet cannot be forwarded to WorldService::Call yet"
                );
                framed.send(response).await?;
                Ok(())
            }
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
        framed.send(GamePacket::build_rs(GameCmd::HeartbeatRs, &rs)?).await?;
        debug!(conn_id, "Heartbeat");
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
        let role_id = self.sessions.get_role_id(conn_id)
            .filter(|&id| id > 0)
            .ok_or_else(|| anyhow!("No role_id in session for conn {}", conn_id))?;

        let cmd = packet.cmd;
        let payload = packet.payload.clone();

        debug!(conn_id, role_id, cmd = ?cmd, "Forward to Home via gRPC Dispatch");

        // 建立 gRPC 连接（TODO: 连接池复用）
        let mut home_client = HomeServiceClient::connect(self.home_addr.clone()).await
            .map_err(|e| anyhow!("Home service unavailable: {}", e))?;

        let dispatch_rq = proto::slg::DispatchRq {
            role_id,
            cmd: cmd as i32,
            payload,
        };

        let rs = home_client.dispatch(dispatch_rq).await
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
}

fn build_world_unimplemented_response(cmd: GameCmd) -> Result<GamePacket> {
    let rs_cmd = GameCmd::from(cmd as u32 + 1);
    let err_payload = shared::msg::GameMessage::build_error(
        rs_cmd as i32,
        WORLD_ROUTE_UNIMPLEMENTED_CODE,
    )?;
    Ok(GamePacket::new(rs_cmd, err_payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_boundary_returns_wrapped_error_response() {
        let packet = build_world_unimplemented_response(GameCmd::DispatchTroopRq)
            .expect("world boundary error packet");

        assert_eq!(packet.cmd, GameCmd::DispatchTroopRs);

        let message = packet.to_message().expect("decode wrapped error");
        assert_eq!(message.base.cmd, GameCmd::DispatchTroopRs as i32);
        assert_eq!(message.base.code, Some(WORLD_ROUTE_UNIMPLEMENTED_CODE));
    }
}
