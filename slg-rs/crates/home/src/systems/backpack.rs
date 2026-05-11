//! 背包系统（BackpackSystem）
//!
//! 对应 Java 版 BackpackFunction，管理道具获取、使用、合成、分解。
//! 数据存储在 p_data.backpack_func（protobuf BackpackDataFunction）。

use anyhow::Result;
use prost::Message;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{BackpackDataFunction, AwardPb};
use shared::persistence::col;
use shared::event::GameEvent;

/// 背包系统
pub struct BackpackSystem {
    dirty: bool,
    /// (type, id) → AwardPb（堆叠道具）
    pub items: Vec<AwardPb>,
    /// 非保护物品
    pub unsafe_items: Vec<AwardPb>,
}

impl BackpackSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            items: Vec::new(),
            unsafe_items: Vec::new(),
        }
    }

    /// 添加道具
    pub fn add_item(&mut self, prop_id: i32, count: i64) {
        // 查找已有堆叠
        if let Some(item) = self.items.iter_mut().find(|i| i.id == prop_id) {
            item.count += count;
        } else {
            self.items.push(AwardPb {
                r#type: 0, // 道具类型，后续从配置读取
                id: prop_id,
                count,
                ..Default::default()
            });
        }
        self.dirty = true;
    }

    /// 消耗道具（返回是否足够）
    pub fn consume_item(&mut self, prop_id: i32, count: i64) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == prop_id) {
            if item.count >= count {
                item.count -= count;
                if item.count == 0 {
                    self.items.retain(|i| i.id != prop_id || i.count > 0);
                }
                self.dirty = true;
                return true;
            }
        }
        false
    }

    /// 添加道具，返回触发的游戏事件列表
    pub fn add_item_with_event(&mut self, role_id: i64, prop_id: i32, count: i64) -> Vec<GameEvent> {
        self.add_item(prop_id, count);
        vec![GameEvent::ItemGain { role_id, prop_id, count }]
    }

    /// 消耗道具，返回 (是否成功, 触发的游戏事件列表)
    pub fn consume_item_with_event(
        &mut self,
        role_id: i64,
        prop_id: i32,
        count: i64,
    ) -> (bool, Vec<GameEvent>) {
        let ok = self.consume_item(prop_id, count);
        if ok {
            (true, vec![GameEvent::ItemConsume { role_id, prop_id, count }])
        } else {
            (false, vec![])
        }
    }

    /// 查询道具数量
    pub fn get_item_count(&self, prop_id: i32) -> i64 {
        self.items.iter()
            .find(|i| i.id == prop_id)
            .map(|i| i.count)
            .unwrap_or(0)
    }

    /// 命令分发入口（实现 PlayerSystem::handle_command）
    pub fn handle_command(
        &mut self,
        cmd: u32,
        _payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<Vec<u8>> {
        match cmd {
            4001 => { /* OneKeyGainTempBagRq */ Ok(vec![]) }
            4003 => { /* PropUseRq */ Ok(vec![]) }
            _ => Err(anyhow::anyhow!("Unknown backpack cmd: {}", cmd)),
        }
    }

    fn to_proto(&self) -> BackpackDataFunction {
        BackpackDataFunction {
            item: self.items.clone(),
            unsafe_item: self.unsafe_items.clone(),
        }
    }
}

impl PlayerSystem for BackpackSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        let func = BackpackDataFunction::decode(data)?;
        self.items = func.item;
        self.unsafe_items = func.unsafe_item;
        info!(items = self.items.len(), "BackpackSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.to_proto().encode_to_vec())
    }

    fn is_dirty(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn column_name(&self) -> &'static str { col::BACKPACK }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<Vec<u8>> {
        BackpackSystem::handle_command(self, cmd, payload, config)
    }

    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<(Vec<u8>, Vec<shared::event::GameEvent>)> {
        let resp = BackpackSystem::handle_command(self, cmd, payload, config)?;
        Ok((resp, vec![]))
    }
}

impl shared::msg::ToFunctionClientBaseBytes for BackpackSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_type, func_tag};
        shared::msg::build_function_base_bytes_pub(func_type::BAG, func_tag::BAG, &self.to_proto())
    }
}
