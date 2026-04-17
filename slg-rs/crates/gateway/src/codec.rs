use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use shared::cmd::GameCmd;
use shared::GameError;
use std::io;

/// 通讯包结构：
/// [Length: u32] (包含 Cmd + Payload 的长度)
/// [Cmd ID: u32]
/// [Protobuf Payload: Bytes]
pub struct GameCodec;

#[derive(Debug)]
pub struct GamePacket {
    pub cmd: GameCmd,
    pub payload: Vec<u8>,
}

impl Decoder for GameCodec {
    type Item = GamePacket;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        // 1. 读取总长度
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&src[..4]);
        let total_len = u32::from_be_bytes(len_bytes) as usize;

        if src.len() < 4 + total_len {
            // 数据包不完整
            return Ok(None);
        }

        // 2. 移除长度头
        src.advance(4);

        // 3. 读取 Cmd ID (4字节)
        if total_len < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Packet too short for CMD ID"));
        }
        let cmd_id = src.get_u32_be();
        let cmd = GameCmd::from(cmd_id);

        // 4. 读取剩余 Payload
        let payload_len = total_len - 4;
        let payload = src.split_to(payload_len).to_vec();

        Ok(Some(GamePacket { cmd, payload }))
    }
}

impl Encoder<GamePacket> for GameCodec {
    type Error = io::Error;

    fn encode(&mut self, item: GamePacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let total_len = (4 + item.payload.len()) as u32;
        
        dst.put_u32_be(total_len);
        dst.put_u32_be(item.cmd.into());
        dst.put_slice(&item.payload);
        
        Ok(())
    }
}
