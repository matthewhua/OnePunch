//! 游戏协议消息封装
//!
//! Java 版使用 Protobuf 2.5 的 `extensions` 机制：所有业务消息通过
//! `extend Base { optional XxxRq ext = <cmd_id>; }` 挂载到 Base 消息上。
//!
//! prost 对 proto2 extensions 的支持有限，本模块手动实现 wire-format 解析，
//! 确保与 Java 客户端完全兼容。
//!
//! # 帧格式
//! ```text
//! ┌──────────┬──────────┬──────────────────────────────────────────┐
//! │ 4 bytes  │ 4 bytes  │ N bytes                                  │
//! │ 总长度    │ 命令号    │ protobuf Base 消息体                      │
//! │ (大端)    │ (大端)    │ (Base 基础字段 + extension 业务消息)       │
//! └──────────┴──────────┴──────────────────────────────────────────┘
//! 总长度 = 4(cmd) + N(body)，不含自身 4 字节
//! ```
//!
//! # Base 消息 extension 格式
//! ```text
//! Base { cmd=1101, code=0 }  +  extension field_1101 { BeginGameRq }
//! ```
//! extension 的 field_number 就等于 cmd 号（proto 文件中 `ext = 1101`）。

use prost::{Message, encoding::{decode_key, WireType}};
use proto::Base;
use anyhow::{Result, anyhow, Context};
use bytes::{Buf, BufMut, BytesMut};

/// 游戏消息包装，处理 Proto2 Extensions 兼容性
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
        let base = Base::decode(data.as_slice())
            .context("Failed to decode Base message")?;
        Ok(Self { base, raw_data: data })
    }

    /// 提取 Extension 负载并反序列化为具体业务消息
    ///
    /// proto2 extension 的 field_number 等于 cmd 号（如 BeginGameRq ext = 1101）。
    /// 本方法在原始字节流中按 wire-format 扫描，找到对应 tag 后提取 payload。
    pub fn get_payload<T: Message + Default>(&self) -> Result<T> {
        let target_tag = self.base.cmd as u32;
        extract_extension_payload(&self.raw_data, target_tag)
    }

    /// 从原始字节中提取指定 field_number 的 extension（用于嵌套 extension）
    pub fn get_extension_from_bytes<T: Message + Default>(data: &[u8], field_number: u32) -> Result<T> {
        extract_extension_payload(data, field_number)
    }

    /// 构建响应消息：Base 基础字段 + 手动编码的 Extension 字段
    ///
    /// 生成的字节流格式：
    /// `Base { cmd, code=0 }` + `extension { field_number=cmd: payload }`
    pub fn build_response<T: Message>(cmd: i32, payload: &T) -> Result<Vec<u8>> {
        let mut base = Base::default();
        base.cmd = cmd;
        base.code = Some(0);

        let mut buf = BytesMut::new();
        // 1. 序列化 Base 基础字段
        base.encode(&mut buf)
            .context("Failed to encode Base message")?;

        // 2. 手动追加 Extension 字段（tag = cmd，wire type = LengthDelimited）
        encode_extension_field(cmd as u32, payload, &mut buf);

        Ok(buf.to_vec())
    }

    /// 构建带错误码的响应消息（通常不含 Extension payload）
    pub fn build_error(cmd: i32, code: i32) -> Result<Vec<u8>> {
        let mut base = Base::default();
        base.cmd = cmd;
        base.code = Some(code);

        let mut buf = Vec::new();
        base.encode(&mut buf)
            .context("Failed to encode error Base message")?;
        Ok(buf)
    }

    /// 构建带错误码和错误参数的响应消息
    pub fn build_error_with_params(cmd: i32, code: i32, params: Vec<String>) -> Result<Vec<u8>> {
        let mut base = Base::default();
        base.cmd = cmd;
        base.code = Some(code);
        base.err_param = params;

        let mut buf = Vec::new();
        base.encode(&mut buf)
            .context("Failed to encode error Base message")?;
        Ok(buf)
    }

    /// 向 Buffer 中写入一个 Extension 字段（供外部使用）
    pub fn encode_extension<T: Message, B: BufMut>(tag: u32, msg: &T, buf: &mut B) {
        encode_extension_field(tag, msg, buf);
    }
}

/// 在 protobuf wire-format 字节流中扫描并提取指定 field_number 的 LengthDelimited 字段
fn extract_extension_payload<T: Message + Default>(data: &[u8], target_tag: u32) -> Result<T> {
    let mut buf = data;

    while buf.has_remaining() {
        let (tag, wire_type) = decode_key(&mut buf)
            .context("Failed to decode wire key")?;

        if tag == target_tag {
            if wire_type != WireType::LengthDelimited {
                return Err(anyhow!(
                    "Extension field {} has unexpected wire type {:?}, expected LengthDelimited",
                    target_tag, wire_type
                ));
            }
            let len = prost::encoding::decode_varint(&mut buf)
                .context("Failed to decode extension length")? as usize;
            if buf.remaining() < len {
                return Err(anyhow!(
                    "Buffer underflow: need {} bytes for extension {}, have {}",
                    len, target_tag, buf.remaining()
                ));
            }
            let payload_data = &buf[..len];
            return T::decode(payload_data)
                .with_context(|| format!("Failed to decode extension payload for field {}", target_tag));
        } else {
            // 跳过非目标字段
            prost::encoding::skip_field(
                wire_type,
                tag,
                &mut buf,
                prost::encoding::DecodeContext::default(),
            )
            .with_context(|| format!("Failed to skip field {}", tag))?;
        }
    }

    Err(anyhow!(
        "Extension field {} not found in message (searched {} bytes)",
        target_tag,
        data.len()
    ))
}

/// 将一个 protobuf 消息编码为 extension 字段追加到 buffer
fn encode_extension_field<T: Message, B: BufMut>(tag: u32, msg: &T, buf: &mut B) {
    let payload_len = msg.encoded_len();
    prost::encoding::encode_key(tag, WireType::LengthDelimited, buf);
    prost::encoding::encode_varint(payload_len as u64, buf);
    msg.encode_raw(buf);
}

// ─── FunctionClientBase 模块 tag 常量 ────────────────────────────────────────
//
// 来源：各 proto 文件中 `extend FunctionClientBase { optional XxxFunction ext = N; }`
// 对应 FunctionTypeDefine 枚举（Common.proto）

/// FunctionClientBase extension tag 定义
/// 每个模块对应一个唯一的 tag，用于在 GetRoleDataRs / SyncFunctionDataRs 中区分模块
pub mod func_tag {
    /// 领主基础数据（LordDataFunction）
    pub const LORD: u32 = 11;
    /// 将领数据（HeroDataFunction）
    pub const HERO: u32 = 12;
    /// 模拟经营数据（SimDataFunction）
    pub const SIM: u32 = 13;
    /// 背包数据（BackpackDataFunction）
    pub const BAG: u32 = 14;
    /// 科技数据（TechnologyDataFunction）
    pub const TECHNOLOGY: u32 = 15;
    /// 任务数据（MissionDataFunction）
    pub const MISSION: u32 = 16;
    /// 副本战斗数据（CombatDataFunction）
    pub const COMBAT: u32 = 17;
    /// 装备数据（EquipDataFunction）
    pub const EQUIP: u32 = 18;
    /// 世界数据（WorldDataFunction）
    pub const WORLD: u32 = 19;
    /// 邮件数据（MailFunction）
    pub const MAIL: u32 = 20;
    /// 充值数据（PayDataFunction）
    pub const PAY: u32 = 21;
    /// 外观数据（GuiseDataFunction）
    pub const GUISE: u32 = 22;
    /// 情报商会数据（IntelBrokerFunction）
    pub const INTEL_BROKER: u32 = 23;
    /// VIP 数据（VipDataFunction）
    pub const VIP: u32 = 24;
    /// 运营活动数据（ActivityFunction）
    pub const ACTIVITY: u32 = 25;
    /// 阵营数据（CampDataFunction）
    pub const CAMP: u32 = 26;
    /// 城墙数据（WallDataFunction）
    pub const WALL: u32 = 27;
    /// 玩法数据（GameplayDataFunction）
    pub const GAMEPLAY: u32 = 28;
    /// 聊天数据（ChatFunction）
    pub const CHAT: u32 = 29;
    /// 商店数据（ShopDataFunction）
    pub const SHOP: u32 = 30;
    /// 领主天赋数据（LordTalentDataFunction）
    pub const LORD_TALENT: u32 = 31;
    /// 竞技场数据（ArenaDataFunction）
    pub const ARENA: u32 = 32;
    /// 皮肤数据（SkinDataFunction）
    pub const SKIN: u32 = 33;
    /// 领主装备数据（LordEquipDataFunction）
    pub const LORD_EQUIP: u32 = 34;
    /// 社交数据（SocialFunction）
    pub const SOCIAL: u32 = 35;
    /// 里程碑数据（MilestoneDataFunction）
    pub const MILESTONE: u32 = 36;
}

/// FunctionTypeDefine 枚举值（与 Common.proto 中定义一致）
pub mod func_type {
    pub const LORD: i32 = 0;
    pub const SIM: i32 = 1;
    pub const HERO: i32 = 2;
    pub const BAG: i32 = 3;
    pub const TECHNOLOGY: i32 = 4;
    pub const MISSION: i32 = 5;
    pub const COMBAT: i32 = 6;
    pub const EQUIP: i32 = 7;
    pub const WORLD: i32 = 8;
    pub const PAY: i32 = 9;
    pub const MAIL: i32 = 10;
    pub const GUISE: i32 = 11;
    pub const INTEL_BROKER: i32 = 12;
    pub const VIP: i32 = 13;
    pub const ACTIVITY: i32 = 14;
    pub const CAMP: i32 = 15;
    pub const WALL: i32 = 16;
    pub const GAMEPLAY: i32 = 17;
    pub const CHAT: i32 = 18;
    pub const SHOP: i32 = 19;
    pub const LORD_TALENT: i32 = 20;
    pub const ARENA: i32 = 21;
    pub const SKIN: i32 = 22;
    pub const LORD_EQUIP: i32 = 23;
    pub const SOCIAL: i32 = 24;
}

/// 将系统模块数据转换为 FunctionClientBase 字节流
///
/// 格式：`FunctionClientBase { type = N }` + `extension { tag=N: XxxFunction }`
/// 客户端通过 `type` 字段识别模块，再通过对应 tag 解析具体数据。
pub trait ToFunctionClientBaseBytes {
    /// 转换为兼容协议的 FunctionClientBase 字节流（包含 Extension）
    fn to_function_base_bytes(&self) -> Vec<u8>;
}

/// 构建 FunctionClientBase 字节流的通用辅助函数
fn build_function_base_bytes<T: Message>(func_type: i32, ext_tag: u32, data: &T) -> Vec<u8> {
    let mut base = proto::slg::FunctionClientBase::default();
    base.r#type = Some(func_type);
    let mut buf = BytesMut::new();
    base.encode(&mut buf).expect("FunctionClientBase encode failed");
    encode_extension_field(ext_tag, data, &mut buf);
    buf.to_vec()
}

/// 公开版本，供 home crate 的各 System 实现 ToFunctionClientBaseBytes 时使用
pub fn build_function_base_bytes_pub<T: Message>(func_type: i32, ext_tag: u32, data: &T) -> Vec<u8> {
    build_function_base_bytes(func_type, ext_tag, data)
}

impl ToFunctionClientBaseBytes for proto::slg::ActivityFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::ACTIVITY, func_tag::ACTIVITY, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::LordDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::LORD, func_tag::LORD, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::MissionDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::MISSION, func_tag::MISSION, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::HeroDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::HERO, func_tag::HERO, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::TechnologyDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::TECHNOLOGY, func_tag::TECHNOLOGY, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::BackpackDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::BAG, func_tag::BAG, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::EquipDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::EQUIP, func_tag::EQUIP, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::VipDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::VIP, func_tag::VIP, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::MailFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::MAIL, func_tag::MAIL, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::ShopDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::SHOP, func_tag::SHOP, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::SimDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::SIM, func_tag::SIM, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::WorldDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::WORLD, func_tag::WORLD, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::CampDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::CAMP, func_tag::CAMP, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::SkinDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::SKIN, func_tag::SKIN, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::LordEquipDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::LORD_EQUIP, func_tag::LORD_EQUIP, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::LordTalentDataFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::LORD_TALENT, func_tag::LORD_TALENT, self)
    }
}

impl ToFunctionClientBaseBytes for proto::slg::SocialFunction {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        build_function_base_bytes(func_type::SOCIAL, func_tag::SOCIAL, self)
    }
}

// ─── 单元测试 ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::BeginGameRq;

    #[test]
    fn test_extension_roundtrip_begin_game() {
        // 构建 BeginGameRq
        let mut rq = BeginGameRq::default();
        rq.server_id = 1;
        rq.token = "test_token".to_string();
        rq.device_no = "device_123".to_string();
        rq.cur_version = "1.0.0".to_string();
        rq.key_id = 12345;

        // 编码为 Base + extension
        let cmd = 1101i32; // BeginGameRq ext = 1101
        let encoded = GameMessage::build_response(cmd, &rq).expect("build_response failed");

        // 解码
        let msg = GameMessage::decode(encoded).expect("decode failed");
        assert_eq!(msg.base.cmd, cmd);
        assert_eq!(msg.base.code, Some(0));

        // 提取 extension
        let decoded: BeginGameRq = msg.get_payload().expect("get_payload failed");
        assert_eq!(decoded.server_id, rq.server_id);
        assert_eq!(decoded.token, rq.token);
        assert_eq!(decoded.device_no, rq.device_no);
        assert_eq!(decoded.key_id, rq.key_id);
    }

    #[test]
    fn test_error_response() {
        let payload = GameMessage::build_error(1102, 101).expect("build_error failed");
        let msg = GameMessage::decode(payload).expect("decode failed");
        assert_eq!(msg.base.cmd, 1102);
        assert_eq!(msg.base.code, Some(101));
    }

    #[test]
    fn test_function_client_base_activity() {
        use proto::slg::{ActivityFunction, ActivityDataPb};

        let mut act_func = ActivityFunction::default();
        act_func.activity.push(ActivityDataPb {
            activity_id: 101,
            open_times: Some(1),
            ..Default::default()
        });

        let bytes = act_func.to_function_base_bytes();
        assert!(!bytes.is_empty());

        // 解析 FunctionClientBase 的 type 字段
        let base = proto::slg::FunctionClientBase::decode(bytes.as_slice())
            .expect("decode FunctionClientBase failed");
        assert_eq!(base.r#type, Some(func_type::ACTIVITY));

        // 提取 ActivityFunction extension
        let decoded: ActivityFunction =
            GameMessage::get_extension_from_bytes(&bytes, func_tag::ACTIVITY)
                .expect("get_extension_from_bytes failed");
        assert_eq!(decoded.activity.len(), 1);
        assert_eq!(decoded.activity[0].activity_id, 101);
    }

    #[test]
    fn test_function_client_base_lord() {
        use proto::slg::LordDataFunction;

        let mut lord = LordDataFunction::default();
        lord.nick_name = Some("TestLord".to_string());
        lord.diamond = Some(9999);

        let bytes = lord.to_function_base_bytes();
        let base = proto::slg::FunctionClientBase::decode(bytes.as_slice())
            .expect("decode FunctionClientBase failed");
        assert_eq!(base.r#type, Some(func_type::LORD));

        let decoded: LordDataFunction =
            GameMessage::get_extension_from_bytes(&bytes, func_tag::LORD)
                .expect("get_extension_from_bytes failed");
        assert_eq!(decoded.nick_name, Some("TestLord".to_string()));
        assert_eq!(decoded.diamond, Some(9999));
    }

    #[test]
    fn test_get_role_data_rs_assembly() {
        use proto::slg::{GetRoleDataRs, FunctionClientBase, LordDataFunction, ActivityFunction};

        // 模拟 PlayerActor 组装 GetRoleDataRs
        let mut lord = LordDataFunction::default();
        lord.nick_name = Some("Hero".to_string());

        let act = ActivityFunction::default();

        let lord_bytes = lord.to_function_base_bytes();
        let act_bytes = act.to_function_base_bytes();

        // 将字节流解析为 FunctionClientBase（模拟客户端行为）
        let lord_base = FunctionClientBase::decode(lord_bytes.as_slice()).unwrap();
        let act_base = FunctionClientBase::decode(act_bytes.as_slice()).unwrap();

        let rs = GetRoleDataRs {
            function_base: vec![lord_base, act_base],
            ..Default::default()
        };

        // 验证组装结果
        assert_eq!(rs.function_base.len(), 2);
        assert_eq!(rs.function_base[0].r#type, Some(func_type::LORD));
        assert_eq!(rs.function_base[1].r#type, Some(func_type::ACTIVITY));
    }
}
