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
        let payload = GameMessage::build_response(cmd as i32, msg)?;
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

        // 2. 读取 Cmd ID (4字节，大端)
        let cmd_id = src.get_u32();
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
        dst.put_u32(total_len);
        dst.put_u32(item.cmd.into());
        dst.put_slice(&item.payload);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::cmd::GameCmd;
    use shared::msg::GameMessage;
    use proto::slg::{BeginGameRq, BeginGameRs, RoleLoginRs};

    #[test]
    fn test_codec_roundtrip_raw_payload() {
        let mut codec = GameCodec;
        let mut buf = BytesMut::new();

        let original_payload = vec![1, 2, 3, 4, 5];
        let packet = GamePacket::new(GameCmd::BeginGameRq, original_payload.clone());

        codec.encode(packet, &mut buf).unwrap();

        // 帧格式：[Len=9(4+5): 4字节] [Cmd=1101: 4字节] [Payload: 5字节]
        assert_eq!(buf.len(), 4 + 4 + 5);

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded.cmd, GameCmd::BeginGameRq);
        assert_eq!(decoded.payload, original_payload);
    }

    #[test]
    fn test_codec_empty_payload() {
        let mut codec = GameCodec;
        let mut buf = BytesMut::new();

        let packet = GamePacket::new(GameCmd::HeartbeatRq, vec![]);
        codec.encode(packet, &mut buf).unwrap();

        // 帧格式：[Len=4: 4字节] [Cmd=1115: 4字节]
        assert_eq!(buf.len(), 8);

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded.cmd, GameCmd::HeartbeatRq);
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn test_codec_partial_data() {
        let mut codec = GameCodec;
        let mut buf = BytesMut::new();

        // 只写入 3 字节（不足以读取长度头）
        buf.put_slice(&[0, 0, 0]);
        assert!(codec.decode(&mut buf).unwrap().is_none());

        // 写入长度头但 payload 不完整
        buf.clear();
        buf.put_u32(100); // 声明 100 字节，但实际只有 4 字节
        buf.put_u32(1101);
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn test_full_protocol_stack_begin_game() {
        // 1. 构建业务消息
        let mut rq = BeginGameRq::default();
        rq.server_id = 99;
        rq.token = "session_key".to_string();
        rq.device_no = "iphone_15".to_string();
        rq.key_id = 888888;

        let cmd = GameCmd::BeginGameRq;

        // 2. 编码为 Base + extension 格式
        let base_payload = GameMessage::build_response(cmd as i32, &rq)
            .expect("build_response failed");

        // 3. 封包（添加帧头）
        let mut codec = GameCodec;
        let mut buf = BytesMut::new();
        let packet = GamePacket::new(cmd, base_payload);
        codec.encode(packet, &mut buf).expect("encode failed");

        // 4. 解包
        let decoded_packet = codec.decode(&mut buf)
            .expect("decode error")
            .expect("packet is None");
        assert_eq!(decoded_packet.cmd, GameCmd::BeginGameRq);

        // 5. 解析 Base + extension
        let msg = decoded_packet.to_message().expect("to_message failed");
        assert_eq!(msg.base.cmd, cmd as i32);

        let decoded_rq: BeginGameRq = msg.get_payload().expect("get_payload failed");
        assert_eq!(decoded_rq.server_id, rq.server_id);
        assert_eq!(decoded_rq.token, rq.token);
        assert_eq!(decoded_rq.device_no, rq.device_no);
        assert_eq!(decoded_rq.key_id, rq.key_id);
    }

    #[test]
    fn test_full_protocol_stack_role_login_rs() {
        // 测试服务端响应消息的编解码
        let mut rs = RoleLoginRs::default();
        rs.state = Some(1);

        let cmd = GameCmd::RoleLoginRs;
        let base_payload = GameMessage::build_response(cmd as i32, &rs)
            .expect("build_response failed");

        let mut codec = GameCodec;
        let mut buf = BytesMut::new();
        codec.encode(GamePacket::new(cmd, base_payload), &mut buf).unwrap();

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded.cmd, GameCmd::RoleLoginRs);

        let msg = decoded.to_message().unwrap();
        let decoded_rs: RoleLoginRs = msg.get_payload().unwrap();
        assert_eq!(decoded_rs.state, Some(1));
    }

    #[test]
    fn test_error_response_codec() {
        let cmd = GameCmd::BeginGameRs;
        let err_payload = GameMessage::build_error(cmd as i32, 403)
            .expect("build_error failed");

        let mut codec = GameCodec;
        let mut buf = BytesMut::new();
        codec.encode(GamePacket::new(cmd, err_payload), &mut buf).unwrap();

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        let msg = decoded.to_message().unwrap();
        assert_eq!(msg.base.cmd, cmd as i32);
        assert_eq!(msg.base.code, Some(403));
    }

    #[test]
    fn test_multiple_packets_in_buffer() {
        // 测试粘包处理：buffer 中有多个完整包
        let mut codec = GameCodec;
        let mut buf = BytesMut::new();

        let p1 = GamePacket::new(GameCmd::HeartbeatRq, vec![]);
        let p2 = GamePacket::new(GameCmd::HeartbeatRs, vec![1, 2]);

        codec.encode(p1, &mut buf).unwrap();
        codec.encode(p2, &mut buf).unwrap();

        let d1 = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(d1.cmd, GameCmd::HeartbeatRq);

        let d2 = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(d2.cmd, GameCmd::HeartbeatRs);
        assert_eq!(d2.payload, vec![1, 2]);

        // buffer 已空
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }
}


