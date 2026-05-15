//! 装备系统（EquipSystem）
//!
//! 对应 Java 版 EquipFunction，管理装备获取、强化、穿戴。
//! 数据存储在 p_data.equip_func（protobuf EquipDataFunction）。

use std::sync::Arc;
use anyhow::{Result, anyhow};
use prost::Message;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{
    EquipDataFunction, Equip,
    EquipWearRq, EquipWearRs,
    EquipUnfixRq, EquipUnfixRs,
    EquipStrengthenRq, EquipStrengthenRs,
    EquipForgingRq, EquipForgingRs,
    EquipLevelResetRq, EquipLevelResetRs,
    ChangeInfo,
};
use shared::persistence::col;
use shared::event::GameEvent;
use shared::static_config::StaticConfig;

/// 装备系统
pub struct EquipSystem {
    dirty: bool,
    /// 装备列表
    pub equips: Vec<Equip>,
}

impl EquipSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            equips: Vec::new(),
        }
    }

    /// 添加装备
    pub fn add_equip(&mut self, equip: Equip) {
        self.equips.push(equip);
        self.dirty = true;
    }

    /// 获取装备（按 equipId 查找）
    pub fn get_equip(&self, equip_id: i32) -> Option<&Equip> {
        self.equips.iter().find(|e| e.equip_id == equip_id)
    }

    /// 装备强化，返回触发的游戏事件列表
    pub fn strengthen(&mut self, role_id: i64, equip_id: i32, add_exp: i32) -> Result<Vec<GameEvent>> {
        let equip = self.equips.iter_mut()
            .find(|e| e.equip_id == equip_id)
            .ok_or_else(|| anyhow!("Equip {} not found", equip_id))?;

        // TODO: 根据 s_equip_lv 配置计算升级
        let cur_exp = equip.exp.unwrap_or(0) + add_exp;
        equip.exp = Some(cur_exp);
        let new_level = equip.level.unwrap_or(0);
        let equip_config_id = equip.equip_config_id;
        self.dirty = true;

        Ok(vec![GameEvent::EquipStrengthen { role_id, equip_config_id, new_level }])
    }

    /// 命令分发入口
    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        match cmd {
            4801 => self.cmd_wear(payload, config),
            4803 => self.cmd_unfix(payload, config),
            4805 => self.cmd_strengthen(payload, config),
            4807 => self.cmd_forging(payload, config),
            4809 => self.cmd_level_reset(payload, config),
            _ => Err(anyhow!("Unknown equip cmd: {}", cmd)),
        }
    }

    // ── 装备穿戴（cmd=4801）────────────────────────────────────────────────────

    fn cmd_wear(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipWearRq::decode(payload)
            .map_err(|e| anyhow!("Decode EquipWearRq: {}", e))?;
        let hero_id = rq.hero_id;

        if let Some(equip_id) = rq.equip_id {
            // 指定装备穿戴
            if let Some(equip) = self.equips.iter_mut().find(|e| e.equip_id == equip_id) {
                equip.wear_hero_id = Some(hero_id);
                self.dirty = true;
            }
        }
        // TODO: 一键穿戴（equip_id 为 None 时）

        let worn: Vec<Equip> = self.equips.iter()
            .filter(|e| e.wear_hero_id == Some(hero_id))
            .cloned().collect();

        let rs = EquipWearRs { hero: vec![], equip: worn, ..Default::default() };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 装备脱下（cmd=4803）────────────────────────────────────────────────────

    fn cmd_unfix(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipUnfixRq::decode(payload)
            .map_err(|e| anyhow!("Decode EquipUnfixRq: {}", e))?;
        let hero_id = rq.hero_id;

        if let Some(equip_id) = rq.equip_id {
            if let Some(equip) = self.equips.iter_mut().find(|e| e.equip_id == equip_id) {
                equip.wear_hero_id = Some(0);
                self.dirty = true;
            }
        } else {
            // 一键脱下
            for equip in &mut self.equips {
                if equip.wear_hero_id == Some(hero_id) {
                    equip.wear_hero_id = Some(0);
                }
            }
            self.dirty = true;
        }

        let rs = EquipUnfixRs { hero: vec![], equip: vec![], ..Default::default() };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 装备强化（cmd=4805）────────────────────────────────────────────────────

    fn cmd_strengthen(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipStrengthenRq::decode(payload)
            .map_err(|e| anyhow!("Decode EquipStrengthenRq: {}", e))?;
        let equip_id = rq.equip_id;

        let equip = self.equips.iter_mut()
            .find(|e| e.equip_id == equip_id)
            .ok_or_else(|| anyhow!("Equip {} not found", equip_id))?;

        let equip_config_id = equip.equip_config_id;
        let cur_level = equip.level.unwrap_or(0);

        // 查找下一级配置，获取所需经验
        let next_level_conf = config.equip.equip_levels.iter()
            .find(|lv| lv.equip_id == equip_config_id && lv.level.parse::<i32>().unwrap_or(0) == cur_level + 1);

        // TODO: 检查消耗的装备/道具是否足够
        // 简化：直接升一级
        equip.level = Some(cur_level + 1);
        let new_level = cur_level + 1;
        self.dirty = true;

        let equip_clone = equip.clone();
        let eaten: Vec<i32> = rq.eat_equip_id.clone();

        // 消耗掉被吃掉的装备
        self.equips.retain(|e| !eaten.contains(&e.equip_id));

        let rs = EquipStrengthenRs {
            equip: Some(equip_clone),
            eat_equip_id: eaten,
            ..Default::default()
        };
        let events = vec![GameEvent::EquipStrengthen { role_id: 0, equip_config_id, new_level }];
        Ok((rs.encode_to_vec(), events))
    }

    // ── 装备专精锻造（cmd=4807）────────────────────────────────────────────────

    fn cmd_forging(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipForgingRq::decode(payload)
            .map_err(|e| anyhow!("Decode EquipForgingRq: {}", e))?;
        let equip_id = rq.equip_id;

        let equip = self.equips.iter_mut()
            .find(|e| e.equip_id == equip_id)
            .ok_or_else(|| anyhow!("Equip {} not found", equip_id))?;

        // TODO: 检查资源消耗
        equip.forging_level = Some(equip.forging_level.unwrap_or(0) + 1);
        self.dirty = true;

        let equip_clone = equip.clone();
        let rs = EquipForgingRs { equip: Some(equip_clone), ..Default::default() };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 装备等级重置（cmd=4809）────────────────────────────────────────────────

    fn cmd_level_reset(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipLevelResetRq::decode(payload)
            .map_err(|e| anyhow!("Decode EquipLevelResetRq: {}", e))?;
        let equip_id = rq.equip_id;
        let reset_type = rq.reset_type; // 1=强化等级, 2=专精等级

        let equip = self.equips.iter_mut()
            .find(|e| e.equip_id == equip_id)
            .ok_or_else(|| anyhow!("Equip {} not found", equip_id))?;

        // TODO: 检查冷却时间、返还资源
        match reset_type {
            1 => { equip.level = Some(0); equip.exp = Some(0); }
            2 => { equip.forging_level = Some(0); }
            _ => return Err(anyhow!("Unknown reset_type: {}", reset_type)),
        }
        self.dirty = true;

        let equip_clone = equip.clone();
        let rs = EquipLevelResetRs {
            reset_type: Some(reset_type),
            equip: Some(equip_clone),
            change_info: Some(ChangeInfo::default()),
            ..Default::default()
        };
        Ok((rs.encode_to_vec(), vec![]))
    }

    fn to_proto(&self) -> EquipDataFunction {
        EquipDataFunction {
            equip: self.equips.clone(),
        }
    }
}

impl PlayerSystem for EquipSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        let func = EquipDataFunction::decode(data)?;
        self.equips = func.equip;
        info!(equips = self.equips.len(), "EquipSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.to_proto().encode_to_vec())
    }

    fn is_dirty(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn column_name(&self) -> &'static str { col::EQUIP }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        let (resp, _) = self.handle_command_with_events(cmd, payload, config)?;
        Ok(resp)
    }

    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        EquipSystem::handle_command(self, cmd, payload, config)
    }
}

impl shared::msg::ToFunctionClientBaseBytes for EquipSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_type, func_tag};
        shared::msg::build_function_base_bytes_pub(func_type::EQUIP, func_tag::EQUIP, &self.to_proto())
    }
}
