//! 背包系统（BackpackSystem）
//!
//! 对应 Java 版 BackpackFunction，管理道具获取、使用、合成、分解。
//! 数据存储在 p_data.backpack_func（protobuf BackpackDataFunction）。

use anyhow::Result;
use prost::Message;
use tracing::{debug, info};

use super::PlayerSystem;
use proto::slg::{AwardPb, BackpackDataFunction, ChangeInfo, PropUseRq, PropUseRs};
use shared::event::GameEvent;
use shared::persistence::col;

const ITEM_AWARD_TYPE: i32 = 4;

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
        self.add_award(AwardPb {
            r#type: ITEM_AWARD_TYPE,
            id: prop_id,
            count,
            safe: Some(true),
            ..Default::default()
        });
    }

    /// 消耗道具（返回是否足够）
    pub fn consume_item(&mut self, prop_id: i32, count: i64) -> bool {
        if count <= 0 || self.get_item_count(prop_id) < count {
            return false;
        }

        let mut remaining = count;
        remaining = consume_from_items(&mut self.items, prop_id, remaining);
        if remaining > 0 {
            consume_from_items(&mut self.unsafe_items, prop_id, remaining);
        }
        self.dirty = true;
        true
    }

    /// 添加道具，返回触发的游戏事件列表
    pub fn add_item_with_event(
        &mut self,
        role_id: i64,
        prop_id: i32,
        count: i64,
    ) -> Vec<GameEvent> {
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
            .chain(self.unsafe_items.iter())
            .filter(|i| i.r#type == ITEM_AWARD_TYPE && i.id == prop_id)
            .map(|i| i.count)
            .sum()
    }

    fn current_award(&self, prop_id: i32) -> AwardPb {
        AwardPb {
            r#type: ITEM_AWARD_TYPE,
            id: prop_id,
            count: self.get_item_count(prop_id),
            safe: Some(true),
            ..Default::default()
        }
    }

    fn add_award(&mut self, award: AwardPb) {
        if award.count <= 0 {
            return;
        }

        let target = if award.safe.unwrap_or(true) {
            &mut self.items
        } else {
            &mut self.unsafe_items
        };

        if let Some(item) = target
            .iter_mut()
            .find(|i| i.r#type == award.r#type && i.id == award.id)
        {
            item.count += award.count;
        } else {
            target.push(award);
        }
        self.dirty = true;
    }

    fn apply_awards(&mut self, awards: &[AwardPb]) -> Vec<AwardPb> {
        let mut changed = Vec::new();
        for award in awards {
            if award.r#type == ITEM_AWARD_TYPE {
                self.add_award(award.clone());
                changed.push(self.current_award(award.id));
            } else {
                debug!(
                    award_type = award.r#type,
                    award_id = award.id,
                    "BackpackSystem ignored non-item award"
                );
            }
        }
        changed
    }

    /// 解析 rewardList 字符串为奖励列表。
    ///
    /// 兼容本仓库里出现的 `[[type,id,count], ...]` 和旧的
    /// `type,id,count;type,id,count` 两种格式。
    fn parse_award_list(award_str: &str) -> Vec<AwardPb> {
        let award_str = award_str.trim();
        if award_str.is_empty() {
            return Vec::new();
        }

        if let Ok(rows) = serde_json::from_str::<Vec<Vec<i64>>>(award_str) {
            return rows
                .into_iter()
                .filter_map(|parts| award_from_parts(&parts))
                .collect();
        }

        award_str
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .filter_map(|seg| {
                let parts: Vec<i64> = seg
                    .split(',')
                    .filter_map(|part| part.trim().parse::<i64>().ok())
                    .collect();
                award_from_parts(&parts)
            })
            .collect()
    }

    fn prop_use_rewards(
        config: &shared::static_config::StaticConfig,
        prop_id: i32,
        use_count: i32,
    ) -> Result<Vec<AwardPb>> {
        let Some(prop) = config.item.props.get(&prop_id) else {
            return Ok(Vec::new());
        };

        if prop.can_use == Some(0) {
            anyhow::bail!("prop {} cannot be used", prop_id);
        }

        let mut rewards = prop
            .reward_list
            .as_deref()
            .map(Self::parse_award_list)
            .unwrap_or_default();
        for award in &mut rewards {
            award.count *= use_count as i64;
        }
        Ok(rewards)
    }

    fn cmd_prop_use(
        &mut self,
        payload: &[u8],
        config: &shared::static_config::StaticConfig,
    ) -> Result<Vec<u8>> {
        let rq = PropUseRq::decode(payload)?;
        let prop_id = rq.prop_id;
        let use_count = rq.use_count;
        if use_count <= 0 {
            anyhow::bail!("invalid prop use count: {}", use_count);
        }

        if !self.consume_item(prop_id, use_count as i64) {
            anyhow::bail!(
                "not enough prop {}: have={}, need={}",
                prop_id,
                self.get_item_count(prop_id),
                use_count
            );
        }

        let rewards = Self::prop_use_rewards(config, prop_id, use_count)?;
        let mut changed_awards = vec![self.current_award(prop_id)];
        changed_awards.extend(self.apply_awards(&rewards));

        let rs = PropUseRs {
            prod_id: Some(prop_id),
            use_count: Some(use_count),
            change_info: Some(ChangeInfo {
                award: changed_awards,
                show_award: rewards,
                show: rq.show,
                ..Default::default()
            }),
        };
        Ok(rs.encode_to_vec())
    }

    pub fn current_item_count(&self, prop_id: i32) -> Option<i64> {
        self.items
            .iter()
            .chain(self.unsafe_items.iter())
            .find(|i| i.r#type == ITEM_AWARD_TYPE && i.id == prop_id)
            .map(|i| i.count)
    }

    /// 命令分发入口（实现 PlayerSystem::handle_command）
    pub fn handle_command(
        &mut self,
        cmd: u32,
        _payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<Vec<u8>> {
        match cmd {
            4001 => {
                /* OneKeyGainTempBagRq */
                Ok(vec![])
            }
            4003 => self.cmd_prop_use(_payload, _config),
            _ => Err(anyhow::anyhow!("Unknown backpack cmd: {}", cmd)),
        }
    }

    pub fn handle_command_with_events_for_role(
        &mut self,
        role_id: i64,
        cmd: u32,
        payload: &[u8],
        config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<(Vec<u8>, Vec<GameEvent>)> {
        let before_items = self.items.clone();
        let before_unsafe_items = self.unsafe_items.clone();
        let resp = self.handle_command(cmd, payload, config)?;
        let events = item_delta_events(role_id, &before_items, &before_unsafe_items, self);
        Ok((resp, events))
    }

    fn to_proto(&self) -> BackpackDataFunction {
        BackpackDataFunction {
            item: self.items.clone(),
            unsafe_item: self.unsafe_items.clone(),
        }
    }
}

fn award_from_parts(parts: &[i64]) -> Option<AwardPb> {
    if parts.len() < 3 {
        return None;
    }
    Some(AwardPb {
        r#type: parts[0] as i32,
        id: parts[1] as i32,
        count: parts[2],
        safe: Some(true),
        ..Default::default()
    })
}

fn consume_from_items(items: &mut Vec<AwardPb>, prop_id: i32, mut remaining: i64) -> i64 {
    for item in items.iter_mut() {
        if remaining == 0 {
            break;
        }
        if item.r#type != ITEM_AWARD_TYPE || item.id != prop_id {
            continue;
        }
        let consumed = item.count.min(remaining);
        item.count -= consumed;
        remaining -= consumed;
    }
    items.retain(|item| item.count > 0);
    remaining
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
        Ok((resp, Vec::new()))
    }
}

impl shared::msg::ToFunctionClientBaseBytes for BackpackSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(func_type::BAG, func_tag::BAG, &self.to_proto())
    }
}

fn item_delta_events(
    role_id: i64,
    before_items: &[AwardPb],
    before_unsafe_items: &[AwardPb],
    after: &BackpackSystem,
) -> Vec<GameEvent> {
    let mut ids: Vec<i32> = before_items
        .iter()
        .chain(before_unsafe_items.iter())
        .chain(after.items.iter())
        .chain(after.unsafe_items.iter())
        .filter(|item| item.r#type == ITEM_AWARD_TYPE)
        .map(|item| item.id)
        .collect();
    ids.sort_unstable();
    ids.dedup();

    ids.into_iter()
        .filter_map(|prop_id| {
            let before = before_items
                .iter()
                .chain(before_unsafe_items.iter())
                .filter(|item| item.r#type == ITEM_AWARD_TYPE && item.id == prop_id)
                .map(|item| item.count)
                .sum::<i64>();
            let after_count = after.get_item_count(prop_id);
            let delta = after_count - before;
            if delta > 0 {
                Some(GameEvent::ItemGain {
                    role_id,
                    prop_id,
                    count: delta,
                })
            } else if delta < 0 {
                Some(GameEvent::ItemConsume {
                    role_id,
                    prop_id,
                    count: -delta,
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::static_config::item::StaticPropConf;
    use shared::static_config::StaticConfig;
    use std::sync::Arc;

    fn prop_conf(prop_id: i32, reward_list: Option<&str>) -> StaticPropConf {
        StaticPropConf {
            prop_id,
            description: String::new(),
            desc2: String::new(),
            asset: None,
            asset_base: None,
            badge: None,
            get_way: None,
            prop_type: 0,
            quality: 0,
            order: 0,
            reward_list: reward_list.map(str::to_string),
            back_type: None,
            can_sell: None,
            duration: None,
            attrs: None,
            jump: None,
            num_display: None,
            dis_position: None,
            can_use: Some(1),
            cli_button: None,
            batch_use: None,
            shop_prop_id: None,
            function_open: None,
            access: None,
            effect: None,
            show_type: None,
            buff_effect_id: None,
            effect_tips_type: 0,
        }
    }

    fn config_with_prop(prop_id: i32, reward_list: Option<&str>) -> Arc<StaticConfig> {
        let mut config = StaticConfig::default();
        config
            .item
            .props
            .insert(prop_id, prop_conf(prop_id, reward_list));
        Arc::new(config)
    }

    #[test]
    fn add_stack_and_consume_items() {
        let mut backpack = BackpackSystem::new();

        backpack.add_item(1001, 2);
        backpack.add_item(1001, 3);
        assert_eq!(backpack.get_item_count(1001), 5);
        assert_eq!(backpack.items.len(), 1);

        assert!(backpack.consume_item(1001, 4));
        assert_eq!(backpack.get_item_count(1001), 1);
        assert!(!backpack.consume_item(1001, 2));
        assert_eq!(backpack.get_item_count(1001), 1);
    }

    #[test]
    fn prop_use_consumes_item_and_grants_reward_list() {
        let mut backpack = BackpackSystem::new();
        backpack.add_item(2001, 3);
        let config = config_with_prop(2001, Some("[[4,3001,2]]"));

        let rq = PropUseRq {
            prop_id: 2001,
            use_count: 2,
            show: Some(0),
            ext_param: Vec::new(),
        };
        let resp = backpack
            .handle_command(4003, &rq.encode_to_vec(), &config)
            .unwrap();
        let rs = PropUseRs::decode(resp.as_slice()).unwrap();

        assert_eq!(rs.prod_id, Some(2001));
        assert_eq!(rs.use_count, Some(2));
        assert_eq!(backpack.get_item_count(2001), 1);
        assert_eq!(backpack.get_item_count(3001), 4);
        assert_eq!(
            rs.change_info
                .unwrap()
                .show_award
                .iter()
                .map(|award| (award.r#type, award.id, award.count))
                .collect::<Vec<_>>(),
            vec![(ITEM_AWARD_TYPE, 3001, 4)]
        );
    }
}
