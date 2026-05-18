//! Chat system baseline.
//!
//! Implements a minimal server-channel chat loop:
//! - `7601 GetChatRoomMsgRq` returns the recent in-memory server-channel cache.
//! - `7603 SendPublicChatRq` validates and appends one public server-channel message.
//! Per-player chat metadata (`ChatFunction`, currently last send time) is persisted in
//! `p_data.chat_func`; the shared room cache is intentionally process-local for this
//! baseline and can be moved to a dedicated actor/p_global persistence later.

use anyhow::Result;
use prost::Message;
use proto::slg::{
    BaseChat, BaseChatRoom, ChatCostTypeDefine, ChatFunction, ChatMsgTypeDefine, ChatRoomDefine,
    GetChatRoomMsgRs, IntLong, SendPublicChatRq, SendPublicChatRs, SerGlobalChat,
};
use shared::persistence::col;
use std::sync::{LazyLock, Mutex};
use tracing::info;

use super::PlayerSystem;

const CMD_GET_CHAT_ROOM_MSG_RQ: u32 = 7601;
const CMD_SEND_PUBLIC_CHAT_RQ: u32 = 7603;
const SERVER_ROOM_ID: i32 = 1;
const MAX_CHAT_CONTENT_CHARS: usize = 200;
const MAX_SERVER_ROOM_CACHE: usize = 50;

static GLOBAL_CHAT: LazyLock<Mutex<SerGlobalChat>> = LazyLock::new(|| {
    Mutex::new(SerGlobalChat {
        room: vec![empty_server_room()],
        generate: Some(0),
    })
});

#[derive(Debug, Clone)]
pub struct ChatSystem {
    dirty: bool,
    data: ChatFunction,
}

impl ChatSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            data: ChatFunction::default(),
        }
    }

    pub fn to_proto(&self) -> ChatFunction {
        self.data.clone()
    }

    pub fn last_chat_time_for_room(&self, room_type: i32) -> Option<i64> {
        self.data
            .last_chat_time
            .iter()
            .find(|entry| entry.v1 == room_type)
            .map(|entry| entry.v2)
    }

    pub fn handle_command_for_role(
        &mut self,
        role_id: i64,
        cmd: u32,
        payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> Result<Vec<u8>> {
        match cmd {
            CMD_GET_CHAT_ROOM_MSG_RQ => self.cmd_get_chat_room_msg(),
            CMD_SEND_PUBLIC_CHAT_RQ => self.cmd_send_public_chat(role_id, payload),
            _ => Err(anyhow::anyhow!("Unknown chat cmd: {}", cmd)),
        }
    }

    fn cmd_get_chat_room_msg(&self) -> Result<Vec<u8>> {
        let snapshot = GLOBAL_CHAT
            .lock()
            .map_err(|_| anyhow::anyhow!("global chat lock poisoned"))?
            .room
            .clone();
        Ok(GetChatRoomMsgRs { room: snapshot }.encode_to_vec())
    }

    fn cmd_send_public_chat(&mut self, role_id: i64, payload: &[u8]) -> Result<Vec<u8>> {
        if role_id <= 0 {
            anyhow::bail!("invalid chat sender role_id={}", role_id);
        }

        let rq = SendPublicChatRq::decode(payload)?;
        validate_chat_cost(rq.chat_cost_type)?;
        let mut chat = rq
            .chat
            .ok_or_else(|| anyhow::anyhow!("missing public chat payload"))?;

        normalize_and_validate_public_chat(&mut chat)?;

        let now_ms = chrono::Utc::now().timestamp_millis();
        let now_secs = (now_ms / 1000).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;

        let mut global = GLOBAL_CHAT
            .lock()
            .map_err(|_| anyhow::anyhow!("global chat lock poisoned"))?;
        let msg_id = next_message_id(&mut global);

        chat.msg_id = Some(msg_id);
        chat.msg_type = Some(ChatMsgTypeDefine::ChatMsgTypePublic as i32);
        chat.room_type = Some(ChatRoomDefine::ServerChannel as i32);
        chat.room_id = Some(SERVER_ROOM_ID);
        chat.time = Some(now_secs);

        let room = ensure_server_room(&mut global);
        room.message_cache.push(chat);
        trim_room_cache(room);
        drop(global);

        self.set_last_chat_time(ChatRoomDefine::ServerChannel as i32, now_ms);
        self.dirty = true;

        Ok(SendPublicChatRs {
            last_chat_time: Some(IntLong {
                v1: ChatRoomDefine::ServerChannel as i32,
                v2: now_ms,
            }),
        }
        .encode_to_vec())
    }

    fn set_last_chat_time(&mut self, room_type: i32, time_ms: i64) {
        if let Some(entry) = self
            .data
            .last_chat_time
            .iter_mut()
            .find(|entry| entry.v1 == room_type)
        {
            entry.v2 = time_ms;
        } else {
            self.data.last_chat_time.push(IntLong {
                v1: room_type,
                v2: time_ms,
            });
        }
    }

    #[cfg(test)]
    fn reset_global_for_test() {
        let mut global = GLOBAL_CHAT.lock().unwrap();
        *global = SerGlobalChat {
            room: vec![empty_server_room()],
            generate: Some(0),
        };
    }
}

impl Default for ChatSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerSystem for ChatSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        self.data = ChatFunction::decode(data)?;
        info!(
            last_chat_time = self.data.last_chat_time.len(),
            "ChatSystem loaded"
        );
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.data.encode_to_vec())
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn column_name(&self) -> &'static str {
        col::CHAT
    }

    fn handle_command(
        &mut self,
        cmd: u32,
        _payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!(
            "ChatSystem requires role context for cmd {}",
            cmd
        ))
    }
}

impl shared::msg::ToFunctionClientBaseBytes for ChatSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(func_type::CHAT, func_tag::CHAT, &self.data)
    }
}

fn validate_chat_cost(chat_cost_type: Option<i32>) -> Result<()> {
    match chat_cost_type.unwrap_or(ChatCostTypeDefine::ChatCostTypeNormal as i32) {
        value if value == ChatCostTypeDefine::ChatCostTypeNormal as i32 => Ok(()),
        other => anyhow::bail!("unsupported chat cost type: {}", other),
    }
}

fn normalize_and_validate_public_chat(chat: &mut BaseChat) -> Result<()> {
    let room_type = chat
        .room_type
        .unwrap_or(ChatRoomDefine::ServerChannel as i32);
    if room_type != ChatRoomDefine::ServerChannel as i32 {
        anyhow::bail!("unsupported public chat room_type: {}", room_type);
    }

    let msg_type = chat
        .msg_type
        .unwrap_or(ChatMsgTypeDefine::ChatMsgTypePublic as i32);
    if msg_type != ChatMsgTypeDefine::ChatMsgTypePublic as i32 {
        anyhow::bail!("unsupported public chat msg_type: {}", msg_type);
    }

    let content = chat.content.as_deref().unwrap_or_default().trim();
    if content.is_empty() {
        anyhow::bail!("chat content is empty");
    }
    if content.chars().count() > MAX_CHAT_CONTENT_CHARS {
        anyhow::bail!(
            "chat content too long: chars={}, max={}",
            content.chars().count(),
            MAX_CHAT_CONTENT_CHARS
        );
    }
    if content.contains('\0') {
        anyhow::bail!("chat content contains invalid null character");
    }

    chat.content = Some(content.to_string());
    Ok(())
}

fn next_message_id(global: &mut SerGlobalChat) -> i64 {
    let next = global.generate.unwrap_or_default().saturating_add(1);
    global.generate = Some(next);
    i64::from(next)
}

fn ensure_server_room(global: &mut SerGlobalChat) -> &mut BaseChatRoom {
    if let Some(index) = global.room.iter().position(is_server_room) {
        return &mut global.room[index];
    }
    global.room.push(empty_server_room());
    global.room.last_mut().expect("server room just inserted")
}

fn is_server_room(room: &BaseChatRoom) -> bool {
    room.room_type == ChatRoomDefine::ServerChannel as i32 && room.room_id == SERVER_ROOM_ID
}

fn empty_server_room() -> BaseChatRoom {
    BaseChatRoom {
        room_type: ChatRoomDefine::ServerChannel as i32,
        room_id: SERVER_ROOM_ID,
        message_cache: Vec::new(),
    }
}

fn trim_room_cache(room: &mut BaseChatRoom) {
    let overflow = room
        .message_cache
        .len()
        .saturating_sub(MAX_SERVER_ROOM_CACHE);
    if overflow > 0 {
        room.message_cache.drain(0..overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::FunctionClientBase;
    use shared::msg::{func_type, ToFunctionClientBaseBytes};
    use shared::static_config::StaticConfig;
    use std::sync::{Arc, Mutex};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn config() -> Arc<StaticConfig> {
        Arc::new(StaticConfig::default())
    }

    fn send_payload(content: &str, room_type: i32) -> Vec<u8> {
        SendPublicChatRq {
            chat: Some(BaseChat {
                room_type: Some(room_type),
                msg_type: Some(ChatMsgTypeDefine::ChatMsgTypePublic as i32),
                content: Some(content.to_string()),
                ..Default::default()
            }),
            chat_cost_type: Some(ChatCostTypeDefine::ChatCostTypeNormal as i32),
        }
        .encode_to_vec()
    }

    #[test]
    fn send_public_chat_appends_server_room_message() {
        let _guard = TEST_LOCK.lock().unwrap();
        ChatSystem::reset_global_for_test();
        let mut system = ChatSystem::new();

        let response = system
            .handle_command_for_role(
                42,
                CMD_SEND_PUBLIC_CHAT_RQ,
                &send_payload(" hello ", 1),
                &config(),
            )
            .unwrap();
        let rs = SendPublicChatRs::decode(response.as_slice()).unwrap();
        assert_eq!(
            rs.last_chat_time.unwrap().v1,
            ChatRoomDefine::ServerChannel as i32
        );
        assert!(system.is_dirty());
        assert!(system.last_chat_time_for_room(1).is_some());

        let query = system
            .handle_command_for_role(42, CMD_GET_CHAT_ROOM_MSG_RQ, &[], &config())
            .unwrap();
        let rooms = GetChatRoomMsgRs::decode(query.as_slice()).unwrap().room;
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].message_cache.len(), 1);
        assert_eq!(rooms[0].message_cache[0].content.as_deref(), Some("hello"));
        assert_eq!(rooms[0].message_cache[0].msg_id, Some(1));
    }

    #[test]
    fn rejects_empty_too_long_and_invalid_target() {
        let _guard = TEST_LOCK.lock().unwrap();
        ChatSystem::reset_global_for_test();
        let mut system = ChatSystem::new();
        let cfg = config();

        assert!(system
            .handle_command_for_role(42, CMD_SEND_PUBLIC_CHAT_RQ, &send_payload("   ", 1), &cfg)
            .is_err());

        let too_long = "x".repeat(MAX_CHAT_CONTENT_CHARS + 1);
        assert!(system
            .handle_command_for_role(
                42,
                CMD_SEND_PUBLIC_CHAT_RQ,
                &send_payload(&too_long, 1),
                &cfg
            )
            .is_err());

        assert!(system
            .handle_command_for_role(
                42,
                CMD_SEND_PUBLIC_CHAT_RQ,
                &send_payload("camp", ChatRoomDefine::CampChannel as i32),
                &cfg,
            )
            .is_err());

        let query = system
            .handle_command_for_role(42, CMD_GET_CHAT_ROOM_MSG_RQ, &[], &cfg)
            .unwrap();
        let rooms = GetChatRoomMsgRs::decode(query.as_slice()).unwrap().room;
        assert!(rooms[0].message_cache.is_empty());
        assert!(!system.is_dirty());
    }

    #[test]
    fn save_load_and_function_base_roundtrip() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut system = ChatSystem::new();
        system.set_last_chat_time(ChatRoomDefine::ServerChannel as i32, 1234);

        let bin = system.save_to_bin().unwrap();
        let mut loaded = ChatSystem::new();
        loaded.load_from_bin(&bin).unwrap();
        assert_eq!(loaded.last_chat_time_for_room(1), Some(1234));

        let base = FunctionClientBase::decode(loaded.to_function_base_bytes().as_slice()).unwrap();
        assert_eq!(base.r#type, Some(func_type::CHAT));
    }
}
