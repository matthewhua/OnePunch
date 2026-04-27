use prost::{Message, encoding::{decode_key, WireType}};
use proto::Base;
use anyhow::{Result, anyhow};
use bytes::{Buf, BytesMut};

/// 游戏消息包装，处理 Proto2 Extensions 兼容性
/// Java 版使用 Protobuf 2.5 extensions，通过 Base 消息的 extension 字段挂载业务消息
#[derive(Debug)]
pub struct GameMessage {
    /// 基础 Base 消息（包含 cmd, code, param 等）
    pub base: Base,
    /// 完整的原始字节数据，用于二次解析 Extension
    pub raw_data: Vec<u8>,
}

impl GameMessage {
    /// 从原始字节流解码 Base 消息
    pub fn decode(data: Vec<u8>) -> Result<Self> {
        let base = Base::decode(data.as_slice())?;
        Ok(Self {
            base,
            raw_data: data,
        })
    }

    /// 提取指定 field_number 的 Extension 负载并反序列化为具体业务消息
    /// Extension 的 field_number 通常等于 Base.cmd
    pub fn get_payload<T: Message + Default>(&self) -> Result<T> {
        let mut buf = &self.raw_data[..];
        let target_tag = self.base.cmd as u32;

        while buf.has_remaining() {
            let (tag, wire_type) = decode_key(&mut buf)?;
            if tag == target_tag {
                if wire_type != WireType::LengthDelimited {
                    return Err(anyhow!("Invalid wire type for extension payload: expected LengthDelimited(2), got {:?}", wire_type));
                }
                let len = prost::encoding::decode_varint(&mut buf)? as usize;
                if buf.remaining() < len {
                    return Err(anyhow!("Unexpected end of buffer parsing extension payload for cmd {}", target_tag));
                }
                let payload_data = &buf[..len];
                return T::decode(payload_data).map_err(|e| anyhow!("Failed to decode extension payload: {}", e));
            } else {
                // 跳过非目标字段
                prost::encoding::skip_field(wire_type, tag, &mut buf, prost::encoding::DecodeContext::default())?;
            }
        }
        Err(anyhow!("Extension payload field {} not found in Base message", target_tag))
    }

    /// 构建响应消息：Base 基础字段 + 手动编码的 Extension 字段
    pub fn build_response<T: Message>(cmd: i32, payload: &T) -> Result<Vec<u8>> {
        let mut base = Base::default();
        base.cmd = cmd;
        base.code = Some(0); // 成功

        let mut buf = BytesMut::new();
        // 1. 序列化 Base 基础字段
        base.encode(&mut buf)?;

        // 2. 手动追加 Extension 字段 (Tag: cmd, WireType: LengthDelimited)
        let payload_len = payload.encoded_len();
        prost::encoding::encode_key(cmd as u32, WireType::LengthDelimited, &mut buf);
        prost::encoding::encode_varint(payload_len as u64, &mut buf);
        payload.encode(&mut buf)?;

        Ok(buf.to_vec())
    }
    
    /// 构建带错误码的响应消息 (通常不含 Extension)
    pub fn build_error(cmd: i32, code: i32) -> Result<Vec<u8>> {
        let mut base = Base::default();
        base.cmd = cmd;
        base.code = Some(code);
        
        let mut buf = Vec::new();
        base.encode(&mut buf)?;
        Ok(buf)
    }

    /// 手动向 Buffer 中写入 Extension 字段
    pub fn encode_extension<T: Message, B: bytes::BufMut>(tag: u32, msg: &T, buf: &mut B) {
        let len = msg.encoded_len();
        prost::encoding::encode_key(tag, WireType::LengthDelimited, buf);
        prost::encoding::encode_varint(len as u64, buf);
        msg.encode_raw(buf);
    }
}

/// 核心模块 ID 定义 (对应 FunctionTypeDefine)
pub mod module_id {
    pub const LORD: i32 = 0;
    pub const SIM: i32 = 1;
    pub const HERO: i32 = 2;
    pub const BAG: i32 = 3;
    pub const TECHNOLOGY: i32 = 4;
    pub const MISSION: i32 = 5;
    pub const WORLD: i32 = 8;
    pub const ACTIVITY: i32 = 14;
}

/// 核心模块 Extension Tag 定义 (对应 FunctionClientBase 的扩展字段编号)
pub mod module_tag {
    pub const LORD: u32 = 11;
    pub const HERO: u32 = 12;
    pub const MISSION: u32 = 16;
    pub const ACTIVITY: u32 = 25;
}

/// 将系统模块数据转换为 FunctionClientBase (用于 GetRoleDataRs 或 SyncFunctionDataRs)
pub trait ToFunctionClientBaseBytes {
    /// 转换为兼容协议的 FunctionClientBase 字节流（包含 Extension）
    fn to_function_base_bytes(&self) -> Vec<u8>;
}

impl ToFunctionClientBaseBytes for proto::slg::ActivityFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        let mut base = proto::slg::FunctionClientBase::default();
        base.r#type = Some(module_id::ACTIVITY);
        let mut buf = BytesMut::new();
        base.encode(&mut buf).unwrap();
        GameMessage::encode_extension(module_tag::ACTIVITY, self, &mut buf);
        buf.to_vec()
    }
}

impl ToFunctionClientBaseBytes for proto::slg::LordDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        let mut base = proto::slg::FunctionClientBase::default();
        base.r#type = Some(module_id::LORD);
        let mut buf = BytesMut::new();
        base.encode(&mut buf).unwrap();
        GameMessage::encode_extension(module_tag::LORD, self, &mut buf);
        buf.to_vec()
    }
}

impl ToFunctionClientBaseBytes for proto::slg::MissionDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        let mut base = proto::slg::FunctionClientBase::default();
        base.r#type = Some(module_id::MISSION);
        let mut buf = BytesMut::new();
        base.encode(&mut buf).unwrap();
        GameMessage::encode_extension(module_tag::MISSION, self, &mut buf);
        buf.to_vec()
    }
}

impl ToFunctionClientBaseBytes for proto::slg::HeroDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        let mut base = proto::slg::FunctionClientBase::default();
        base.r#type = Some(module_id::HERO);
        let mut buf = BytesMut::new();
        base.encode(&mut buf).unwrap();
        GameMessage::encode_extension(module_tag::HERO, self, &mut buf);
        buf.to_vec()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::BeginGameRq;

    #[test]
    fn test_game_message_extension_roundtrip() {
        // 1. 构建 BeginGameRq
        let mut rq = BeginGameRq::default();
        rq.server_id = 1;
        rq.token = "test_token".to_string();
        rq.device_no = "device_123".to_string();
        rq.cur_version = "1.0.0".to_string();
        rq.key_id = 12345;

        // 2. 构建包含 Extension 的 Base 消息字节流
        let cmd = 1101; // BeginGameRq extension tag
        let encoded_bytes = GameMessage::build_response(cmd, &rq).expect("Failed to build response");

        // 3. 解码为 GameMessage
        let msg = GameMessage::decode(encoded_bytes).expect("Failed to decode GameMessage");
        assert_eq!(msg.base.cmd, cmd);

        // 4. 提取 Extension 负载
        let decoded_rq: BeginGameRq = msg.get_payload().expect("Failed to get extension payload");
        assert_eq!(decoded_rq.server_id, rq.server_id);
        assert_eq!(decoded_rq.token, rq.token);
        assert_eq!(decoded_rq.device_no, rq.device_no);
        assert_eq!(decoded_rq.key_id, rq.key_id);
    }

    #[test]
    fn test_function_client_base_roundtrip() {
        use proto::slg::ActivityFunction;
        use proto::slg::FunctionClientBase;

        // 1. 构建一个 ActivityFunction
        let mut act_func = ActivityFunction::default();
        act_func.activity.push(proto::slg::ActivityDataPb {
            activity_id: 101,
            open_times: Some(1),
            ..Default::default()
        });

        // 2. 使用 Trait 包装为 FunctionClientBase 字节流
        let f_base_bytes = act_func.to_function_base_bytes();
        
        // 3. 模拟解析：从字节流提取 ActivityFunction
        let mut buf = &f_base_bytes[..];
        // 首先解析出 FunctionClientBase 的 type 字段
        let f_base = proto::slg::FunctionClientBase::decode_length_delimited(&mut buf).expect("Failed to decode base");
        assert_eq!(f_base.r#type, Some(module_id::ACTIVITY));
        
        // 剩余部分应为 Extension
        let (tag, wire_type) = decode_key(&mut buf).expect("Failed to decode key");
        let (tag, wire_type) = decode_key(&mut buf).expect("Failed to decode key");
        assert_eq!(tag, module_tag::ACTIVITY);
        assert_eq!(wire_type, WireType::LengthDelimited);
        
        let len = prost::encoding::decode_varint(&mut buf).expect("Failed to decode varint");
        let payload = &buf[..len as usize];
        let decoded_act: ActivityFunction = ActivityFunction::decode(payload).expect("Failed to decode final payload");
        
        assert_eq!(decoded_act.activity[0].activity_id, 101);
    }
}

