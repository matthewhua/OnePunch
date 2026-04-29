use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, error};
use crate::codec::{GameCodec, GamePacket};
use shared::cmd::GameCmd;
use proto::slg::auth_service_client::AuthServiceClient;

pub struct ConnectionHandler {
    auth_addr: String,
}

impl ConnectionHandler {
    pub fn new(auth_addr: String) -> Self {
        Self { auth_addr }
    }

    pub async fn handle(&self, stream: TcpStream) -> anyhow::Result<()> {
        let mut framed = Framed::new(stream, GameCodec);
        let mut account_id: Option<i64> = None;

        info!("New connection from: {:?}", framed.get_ref().peer_addr()?);

        loop {
            // 设置 60 秒超时
            let packet_result = timeout(Duration::from_secs(60), framed.next()).await;

            match packet_result {
                Ok(Some(Ok(packet))) => {
                    if account_id.is_none() {
                        // 鉴权前，只允许登录请求
                        if packet.cmd == GameCmd::LoginRq {
                            match self.handle_login(packet, &mut framed).await {
                                Ok(id) => {
                                    account_id = Some(id);
                                    info!("Login success for account: {}", id);
                                }
                                Err(e) => {
                                    warn!("Login failed: {}", e);
                                    break;
                                }
                            }
                        } else {
                            warn!("Unauthorized command before login: {:?}", packet.cmd);
                            break;
                        }
                    } else {
                        // 已登录逻辑
                        self.handle_authenticated_packet(packet, account_id.unwrap(), &mut framed).await?;
                    }
                }
                Ok(Some(Err(e))) => {
                    error!("Protocol error: {}", e);
                    break;
                }
                Ok(None) => {
                    info!("Connection closed by peer");
                    break;
                }
                Err(_) => {
                    // 60秒超时
                    warn!("Connection idle timeout (60s)");
                    // 可以尝试发送断开通知
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_login(&self, packet: GamePacket, framed: &mut Framed<TcpStream, GameCodec>) -> anyhow::Result<i64> {
        // 1. 将原始包转换为 GameMessage (解析 Base + Extensions)
        let msg = packet.to_message()?;
        
        // 2. 从 Extension 中提取具体的请求 payload
        // 注意：由于是内部 AuthService 用的 LoginRequest，这里假设客户端协议与之对应
        // 如果客户端协议不同，则需要根据具体的 proto2 消息类型进行中转
        let req: proto::slg::LoginRequest = msg.get_payload()?;
        
        // 调用 Auth 服务
        let mut client = AuthServiceClient::connect(self.auth_addr.clone()).await?;
        let response = client.login(req).await?.into_inner();

        if response.success {
            // 返回成功给客户端
            // 3. 使用 build_rs 构建符合全兼容协议的响应包 (Base + Extension)
            let rs_packet = GamePacket::build_rs(GameCmd::LoginRs, &response)?;
            framed.send(rs_packet).await?;
            
            Ok(response.account_id)
        } else {
            // 4. 构建带错误码的响应包 (Base.code)
            let err_payload = shared::msg::GameMessage::build_error(GameCmd::LoginRs as i32, 101)?; // 101: 登录失败错误码
            framed.send(GamePacket::new(GameCmd::LoginRs, err_payload)).await?;
            
            return Err(anyhow::anyhow!("Auth failed: {}", response.error_msg));
        }
    }

    async fn handle_authenticated_packet(
        &self, 
        packet: GamePacket, 
        account_id: i64, 
        _framed: &mut Framed<TcpStream, GameCodec>
    ) -> anyhow::Result<()> {
        info!("Handling packet for authenticated user {}: {:?}", account_id, packet.cmd);
        // 这里后续会转发到 Home/World 服务
        Ok(())
    }
}
