use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use shared::cmd::GameCmd;
use std::io;
use shared::msg::GameMessage;

/// 通讯包结构：
/// [Length: u32] (包含 Cmd + Payload 的长度)
/// [Cmd ID: u32] 
/// [Protobuf Payload: Bytes] (Base 消息 + Extensions)
#[derive(Debug)]
pub struct GamePacket {
    pub cmd: GameCmd,
    pub payload: Vec<u8>, // 原始 Base 消息字节
}

impl GamePacket {
    /// 新建一个包
    pub fn new(cmd: GameCmd, payload: Vec<u8>) -> Self {
        Self { cmd, payload }
    }

    /// 从具体业务消息构建响应包
    pub fn build_rs<T: prost::Message>(cmd: GameCmd, msg: &T) -> anyhow::Result<Self> {
        let payload = GameMessage::build_response(cmd.into(), msg)?;
        Ok(Self { cmd, payload })
    }

    /// 转换为 GameMessage 结构进行深度解析
    pub fn to_message(&self) -> anyhow::Result<GameMessage> {
        GameMessage::decode(self.payload.clone())
    }
}

pub struct GameCodec;

impl Decoder for GameCodec {
    type Item = GamePacket;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        // 1. 读取总长度 (4字节)，先 peek 不消费
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&src[..4]);
        let total_len = u32::from_be_bytes(len_bytes) as usize;

        // total_len 应该是 cmd_id(4) + payload_len
        // 验证最小长度
        if total_len < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Packet length too short: {}", total_len)));
        }

        if src.len() < 4 + total_len {
            // 数据包不完整
            return Ok(None);
        }

        // 确认数据完整后，消费长度头
        src.advance(4);

        // 2. 读取 Cmd ID (4字节)
        let cmd_id = src.get_u32_be();
        let cmd = GameCmd::from(cmd_id);

        // 3. 读取剩余 Payload (Base 消息)
        let payload_len = total_len - 4;
        let payload = src.split_to(payload_len).to_vec();

        Ok(Some(GamePacket { cmd, payload }))
    }
}

impl Encoder<GamePacket> for GameCodec {
    type Error = io::Error;

    fn encode(&mut self, item: GamePacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // total_len = cmd_id(4) + payload_len
        let total_len = (4 + item.payload.len()) as u32;
        
        dst.reserve(4 + total_len as usize);
        dst.put_u32_be(total_len);
        dst.put_u32_be(item.cmd.into());
        dst.put_slice(&item.payload);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::cmd::GameCmd;
    use shared::msg::GameMessage;
    use proto::slg::BeginGameRq;

    #[test]
    fn test_codec_roundtrip() {
        let mut codec = GameCodec;
        let mut buf = BytesMut::new();
        
        let original_payload = vec![1, 2, 3, 4, 5];
        let packet = GamePacket::new(GameCmd::LoginRq, original_payload.clone());
        
        codec.encode(packet, &mut buf).unwrap();
        
        // [Len: 4+5=9] [Cmd: 1001] [Payload: 1,2,3,4,5]
        assert_eq!(buf.len(), 4 + 4 + 5);
        
        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded.cmd, GameCmd::LoginRq);
        assert_eq!(decoded.payload, original_payload);
    }

    #[test]
    fn test_full_protocol_stack_roundtrip() {
        // 1. 模拟客户端行为：构建业务消息并序列化为 Base Extension 格式
        let mut rq = BeginGameRq::default();
        rq.server_id = 99;
        rq.token = "session_key".to_string();
        rq.device_no = "iphone_15".to_string();
        
        let cmd = GameCmd::BeginGameRq;
        let base_payload = GameMessage::build_response(cmd.into(), &rq).expect("Failed to build base response");
        
        // 2. 网关层封包：添加帧长度头和命令号头
        let mut codec = GameCodec;
        let mut buf = BytesMut::new();
        let packet = GamePacket::new(cmd, base_payload);
        codec.encode(packet, &mut buf).expect("Failed to encode packet");
        
        // 3. 模拟网关接收：剥离封包头，验证命令号一致性
        let decoded_packet = codec.decode(&mut buf).expect("Failed to decode").expect("Packet is None");
        assert_eq!(decoded_packet.cmd, GameCmd::BeginGameRq);
        
        // 4. 应用层解析：使用 GameMessage 解析 Base 消息并提取其内嵌的 Extension
        let msg = decoded_packet.to_message().expect("Failed to parse GameMessage");
        assert_eq!(msg.base.cmd, cmd as i32);
        
        let decoded_rq: BeginGameRq = msg.get_payload().expect("Failed to get business payload");
        
        // 5. 验证业务数据完整性
        assert_eq!(decoded_rq.server_id, rq.server_id);
        assert_eq!(decoded_rq.token, rq.token);
        assert_eq!(decoded_rq.device_no, rq.device_no);
    }
}


