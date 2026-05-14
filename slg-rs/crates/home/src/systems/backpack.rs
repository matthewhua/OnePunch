//! 背包系统（BackpackSystem）
//!
//! 对应 Java 版 BackpackFunction，管理道具获取、使用、合成、分解。
//! 数据存储在 p_data.backpack_func（protobuf BackpackDataFunction）。

use anyhow::{anyhow, Result};
use prost::Message;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{
    AwardPb, BackpackDataFunction, ChangeInfo, OneKeyGainTempBagRq, OneKeyGainTempBagRs, PropUseRq,
    PropUseRs,
};
use shared::event::GameEvent;
use shared::persistence::col;

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
        if prop_id <= 0 || count <= 0 {
            return;
        }

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
        if prop_id <= 0 || count <= 0 {
            return false;
        }

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
    pub fn add_item_with_event(
        &mut self,
        role_id: i64,
        prop_id: i32,
        count: i64,
    ) -> Vec<GameEvent> {
        if prop_id <= 0 || count <= 0 {
            return Vec::new();
        }
        self.add_item(prop_id, count);
        vec![GameEvent::ItemGain {
            role_id,
            prop_id,
            count,
        }]
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
            (
                true,
                vec![GameEvent::ItemConsume {
                    role_id,
                    prop_id,
                    count,
                }],
            )
        } else {
            (false, vec![])
        }
    }

    /// 查询道具数量
    pub fn get_item_count(&self, prop_id: i32) -> i64 {
        self.items
            .iter()
            .find(|i| i.id == prop_id)
            .map(|i| i.count)
            .unwrap_or(0)
    }

    /// 命令分发入口（实现 PlayerSystem::handle_command）
    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<Vec<u8>> {
        match cmd {
            4001 => self.cmd_one_key_gain_temp_bag(payload),
            4003 => self.cmd_prop_use(payload),
            _ => Err(anyhow::anyhow!("Unknown backpack cmd: {}", cmd)),
        }
    }

    fn cmd_one_key_gain_temp_bag(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        OneKeyGainTempBagRq::decode(payload)
            .map_err(|e| anyhow!("Decode OneKeyGainTempBagRq: {}", e))?;

        // Temp bag storage is not modeled yet, so this command is an explicit no-op.
        let rs = OneKeyGainTempBagRs {
            info: Some(ChangeInfo::default()),
        };
        Ok(rs.encode_to_vec())
    }

    fn cmd_prop_use(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq = PropUseRq::decode(payload).map_err(|e| anyhow!("Decode PropUseRq: {}", e))?;
        let prop_id = rq.prop_id;
        let use_count = rq.use_count;

        if prop_id <= 0 {
            return Err(anyhow!("Invalid prop id: {}", prop_id));
        }
        if use_count <= 0 {
            return Err(anyhow!("Invalid use count: {}", use_count));
        }

        let item_type = self
            .items
            .iter()
            .find(|i| i.id == prop_id)
            .map(|i| i.r#type)
            .ok_or_else(|| anyhow!("Item {} not found", prop_id))?;

        if !self.consume_item(prop_id, use_count as i64) {
            return Err(anyhow!(
                "Insufficient item {}: have {}, need {}",
                prop_id,
                self.get_item_count(prop_id),
                use_count
            ));
        }

        let current = AwardPb {
            r#type: item_type,
            id: prop_id,
            count: self.get_item_count(prop_id),
            ..Default::default()
        };

        // Reward effects from s_prop_conf are deferred until PlayerActor owns award dispatch.
        let rs = PropUseRs {
            prod_id: Some(prop_id),
            use_count: Some(use_count),
            change_info: Some(ChangeInfo {
                award: vec![current],
                show: rq.show,
                ..Default::default()
            }),
        };
        Ok(rs.encode_to_vec())
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
        self.dirty = false;
        info!(items = self.items.len(), "BackpackSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.to_proto().encode_to_vec())
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
        col::BACKPACK
    }

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
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(func_type::BAG, func_tag::BAG, &self.to_proto())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use prost::Message;
    use shared::msg::{func_tag, func_type, GameMessage, ToFunctionClientBaseBytes};
    use shared::static_config::StaticConfig;

    use super::*;
    use proto::slg::{
        BackpackDataFunction, FunctionClientBase, OneKeyGainTempBagRq, OneKeyGainTempBagRs,
        PropUseRq, PropUseRs,
    };

    fn default_config() -> Arc<StaticConfig> {
        Arc::new(StaticConfig::default())
    }

    #[test]
    fn add_and_consume_helpers_update_item_counts() {
        let mut system = BackpackSystem::new();

        system.add_item(101, 5);
        system.add_item(101, 2);
        assert_eq!(system.get_item_count(101), 7);
        assert!(system.is_dirty());

        system.clear_dirty();
        assert!(system.consume_item(101, 3));
        assert_eq!(system.get_item_count(101), 4);
        assert!(system.is_dirty());

        system.clear_dirty();
        assert!(!system.consume_item(101, 9));
        assert_eq!(system.get_item_count(101), 4);
        assert!(!system.is_dirty());

        assert!(system.add_item_with_event(1, 0, 1).is_empty());
        assert!(system.add_item_with_event(1, 101, 0).is_empty());
        assert_eq!(system.get_item_count(101), 4);
    }

    #[test]
    fn persistence_roundtrip_preserves_items_and_unsafe_items() {
        let mut system = BackpackSystem::new();
        system.items.push(AwardPb {
            r#type: 7,
            id: 201,
            count: 11,
            ..Default::default()
        });
        system.unsafe_items.push(AwardPb {
            r#type: 8,
            id: 202,
            count: 12,
            ..Default::default()
        });

        let bytes = system.save_to_bin().expect("save backpack");
        let mut loaded = BackpackSystem::new();
        loaded.mark_dirty();
        loaded.load_from_bin(&bytes).expect("load backpack");

        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].id, 201);
        assert_eq!(loaded.items[0].count, 11);
        assert_eq!(loaded.unsafe_items.len(), 1);
        assert_eq!(loaded.unsafe_items[0].id, 202);
        assert_eq!(loaded.unsafe_items[0].count, 12);
        assert!(!loaded.is_dirty());
    }

    #[test]
    fn function_base_output_reflects_current_item_state() {
        let mut system = BackpackSystem::new();
        system.add_item(301, 13);

        let bytes = system.to_function_base_bytes();
        let base = FunctionClientBase::decode(bytes.as_slice()).expect("decode function base");
        assert_eq!(base.r#type, Some(func_type::BAG));

        let decoded: BackpackDataFunction =
            GameMessage::get_extension_from_bytes(&bytes, func_tag::BAG)
                .expect("decode backpack extension");
        assert_eq!(decoded.item.len(), 1);
        assert_eq!(decoded.item[0].id, 301);
        assert_eq!(decoded.item[0].count, 13);
    }

    #[test]
    fn one_key_temp_bag_returns_valid_noop_response() {
        let mut system = BackpackSystem::new();
        system.add_item(401, 3);
        system.clear_dirty();

        let rq = OneKeyGainTempBagRq::default().encode_to_vec();
        let resp = system
            .handle_command(4001, &rq, &default_config())
            .expect("temp bag response");
        let rs = OneKeyGainTempBagRs::decode(resp.as_slice()).expect("decode temp bag response");

        assert_eq!(system.get_item_count(401), 3);
        assert!(!system.is_dirty());
        assert!(rs.info.is_some());
        assert!(rs.info.unwrap().show_award.is_empty());
    }

    #[test]
    fn prop_use_consumes_inventory_and_returns_change_info() {
        let mut system = BackpackSystem::new();
        system.add_item(501, 5);
        system.clear_dirty();

        let rq = PropUseRq {
            prop_id: 501,
            use_count: 2,
            show: Some(0),
            ext_param: vec![],
        }
        .encode_to_vec();
        let resp = system
            .handle_command(4003, &rq, &default_config())
            .expect("prop use response");
        let rs = PropUseRs::decode(resp.as_slice()).expect("decode prop use response");

        assert_eq!(system.get_item_count(501), 3);
        assert!(system.is_dirty());
        assert_eq!(rs.prod_id, Some(501));
        assert_eq!(rs.use_count, Some(2));
        let change = rs.change_info.expect("change info");
        assert_eq!(change.show, Some(0));
        assert_eq!(change.award.len(), 1);
        assert_eq!(change.award[0].id, 501);
        assert_eq!(change.award[0].count, 3);
        assert!(change.show_award.is_empty());
    }

    #[test]
    fn prop_use_rejects_insufficient_item_without_mutation() {
        let mut system = BackpackSystem::new();
        system.add_item(601, 1);
        system.clear_dirty();

        let rq = PropUseRq {
            prop_id: 601,
            use_count: 2,
            show: Some(-1),
            ext_param: vec![],
        }
        .encode_to_vec();
        let err = system
            .handle_command(4003, &rq, &default_config())
            .expect_err("insufficient item");

        assert!(err.to_string().contains("Insufficient item"));
        assert_eq!(system.get_item_count(601), 1);
        assert!(!system.is_dirty());
    }

    #[test]
    fn unknown_command_returns_error() {
        let mut system = BackpackSystem::new();
        let err = system
            .handle_command(4999, &[], &default_config())
            .expect_err("unknown command");

        assert!(err.to_string().contains("Unknown backpack cmd: 4999"));
    }
}
