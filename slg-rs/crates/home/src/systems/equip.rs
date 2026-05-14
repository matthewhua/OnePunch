use std::sync::Arc;

use anyhow::{anyhow, Result};
use prost::Message;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{
    ChangeInfo, Equip, EquipDataFunction, EquipForgingRq, EquipForgingRs, EquipLevelResetRq,
    EquipLevelResetRs, EquipStrengthenRq, EquipStrengthenRs, EquipUnfixRq, EquipUnfixRs,
    EquipWearRq, EquipWearRs,
};
use shared::event::GameEvent;
use shared::persistence::col;
use shared::static_config::StaticConfig;

pub struct EquipSystem {
    dirty: bool,
    pub equips: Vec<Equip>,
}

impl EquipSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            equips: Vec::new(),
        }
    }

    pub fn add_equip(&mut self, equip: Equip) {
        self.equips.push(equip);
        self.dirty = true;
    }

    pub fn get_equip(&self, equip_id: i32) -> Option<&Equip> {
        self.equips.iter().find(|e| e.equip_id == equip_id)
    }

    fn validate_hero_id(hero_id: i32) -> Result<()> {
        if hero_id <= 0 {
            return Err(anyhow!("Invalid hero id: {}", hero_id));
        }
        Ok(())
    }

    fn validate_equip_id(equip_id: i32) -> Result<()> {
        if equip_id <= 0 {
            return Err(anyhow!("Invalid equip id: {}", equip_id));
        }
        Ok(())
    }

    fn equip_index(&self, equip_id: i32) -> Result<usize> {
        Self::validate_equip_id(equip_id)?;
        self.equips
            .iter()
            .position(|e| e.equip_id == equip_id)
            .ok_or_else(|| anyhow!("Equip {} not found", equip_id))
    }

    fn equip_slot(config: &StaticConfig, equip: &Equip) -> Result<i32> {
        if equip.equip_config_id <= 0 {
            return Err(anyhow!(
                "Invalid equip config id {} for equip {}",
                equip.equip_config_id,
                equip.equip_id
            ));
        }
        let equip_conf = config
            .equip
            .equips
            .get(&equip.equip_config_id)
            .ok_or_else(|| anyhow!("Equip config {} not found", equip.equip_config_id))?;
        let slot = equip_conf
            .slot_type
            .ok_or_else(|| anyhow!("Equip config {} missing slot", equip.equip_config_id))?;
        if slot <= 0 {
            return Err(anyhow!(
                "Invalid slot {} for equip config {}",
                slot,
                equip.equip_config_id
            ));
        }
        Ok(slot)
    }

    fn validate_equip_config(config: &StaticConfig, equip: &Equip) -> Result<()> {
        Self::equip_slot(config, equip).map(|_| ())
    }

    fn next_level_config<'a>(
        config: &'a StaticConfig,
        equip_config_id: i32,
        next_level: i32,
    ) -> Result<&'a shared::static_config::equip::StaticEquipLv> {
        config
            .equip
            .equip_levels
            .iter()
            .find(|lv| {
                lv.equip_id == equip_config_id && lv.level.parse::<i32>().ok() == Some(next_level)
            })
            .ok_or_else(|| {
                anyhow!(
                    "Equip config {} next level {} not found",
                    equip_config_id,
                    next_level
                )
            })
    }

    pub fn strengthen(
        &mut self,
        role_id: i64,
        equip_id: i32,
        add_exp: i32,
    ) -> Result<Vec<GameEvent>> {
        let equip = self
            .equips
            .iter_mut()
            .find(|e| e.equip_id == equip_id)
            .ok_or_else(|| anyhow!("Equip {} not found", equip_id))?;

        let cur_exp = equip.exp.unwrap_or(0) + add_exp;
        equip.exp = Some(cur_exp);
        let new_level = equip.level.unwrap_or(0);
        let equip_config_id = equip.equip_config_id;
        self.dirty = true;

        Ok(vec![GameEvent::EquipStrengthen {
            role_id,
            equip_config_id,
            new_level,
        }])
    }

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

    fn cmd_wear(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipWearRq::decode(payload).map_err(|e| anyhow!("Decode EquipWearRq: {}", e))?;
        let hero_id = rq.hero_id;
        Self::validate_hero_id(hero_id)?;

        let equip_id = rq
            .equip_id
            .ok_or_else(|| anyhow!("One-key equip wear is not supported yet"))?;
        let index = self.equip_index(equip_id)?;
        let slot = Self::equip_slot(config, &self.equips[index])?;

        match self.equips[index].wear_hero_id.unwrap_or(0) {
            0 => {}
            worn if worn == hero_id => {
                return Err(anyhow!(
                    "Equip {} already worn by hero {}",
                    equip_id,
                    hero_id
                ));
            }
            worn => return Err(anyhow!("Equip {} already worn by hero {}", equip_id, worn)),
        }

        for equip in &self.equips {
            if equip.equip_id == equip_id || equip.wear_hero_id != Some(hero_id) {
                continue;
            }
            let worn_slot = Self::equip_slot(config, equip)?;
            if worn_slot == slot {
                return Err(anyhow!(
                    "Hero {} already has equip {} in slot {}",
                    hero_id,
                    equip.equip_id,
                    slot
                ));
            }
        }

        self.equips[index].wear_hero_id = Some(hero_id);
        self.dirty = true;

        let worn: Vec<Equip> = self
            .equips
            .iter()
            .filter(|e| e.wear_hero_id == Some(hero_id))
            .cloned()
            .collect();

        let rs = EquipWearRs {
            hero: vec![],
            equip: worn,
            ..Default::default()
        };
        Ok((rs.encode_to_vec(), vec![]))
    }

    fn cmd_unfix(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            EquipUnfixRq::decode(payload).map_err(|e| anyhow!("Decode EquipUnfixRq: {}", e))?;
        let hero_id = rq.hero_id;
        Self::validate_hero_id(hero_id)?;

        if let Some(equip_id) = rq.equip_id {
            let index = self.equip_index(equip_id)?;
            Self::validate_equip_config(config, &self.equips[index])?;
            if self.equips[index].wear_hero_id != Some(hero_id) {
                return Err(anyhow!("Equip {} not worn by hero {}", equip_id, hero_id));
            }
            self.equips[index].wear_hero_id = Some(0);
            self.dirty = true;
        } else {
            let mut changed = false;
            for equip in &mut self.equips {
                if equip.wear_hero_id == Some(hero_id) {
                    equip.wear_hero_id = Some(0);
                    changed = true;
                }
            }
            if !changed {
                return Err(anyhow!("Hero {} has no worn equips", hero_id));
            }
            self.dirty = true;
        }

        let refreshed: Vec<Equip> = if let Some(equip_id) = rq.equip_id {
            self.equips
                .iter()
                .filter(|e| e.equip_id == equip_id)
                .cloned()
                .collect()
        } else {
            self.equips
                .iter()
                .filter(|e| e.wear_hero_id == Some(0))
                .cloned()
                .collect()
        };
        let rs = EquipUnfixRs {
            hero: vec![],
            equip: refreshed,
            ..Default::default()
        };
        Ok((rs.encode_to_vec(), vec![]))
    }

    fn cmd_strengthen(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipStrengthenRq::decode(payload)
            .map_err(|e| anyhow!("Decode EquipStrengthenRq: {}", e))?;
        let equip_id = rq.equip_id;
        let index = self.equip_index(equip_id)?;
        Self::validate_equip_config(config, &self.equips[index])?;

        let equip_config_id = self.equips[index].equip_config_id;
        let cur_level = self.equips[index].level.unwrap_or(0);
        if cur_level < 0 {
            return Err(anyhow!(
                "Invalid equip level {} for equip {}",
                cur_level,
                equip_id
            ));
        }
        let new_level = cur_level + 1;
        let _next_level_conf = Self::next_level_config(config, equip_config_id, new_level)?;

        let mut eaten = Vec::new();
        for eaten_id in &rq.eat_equip_id {
            Self::validate_equip_id(*eaten_id)?;
            if *eaten_id == equip_id {
                return Err(anyhow!("Equip {} cannot eat itself", equip_id));
            }
            if eaten.contains(eaten_id) {
                return Err(anyhow!("Duplicate eaten equip id: {}", eaten_id));
            }
            let eaten_index = self.equip_index(*eaten_id)?;
            Self::validate_equip_config(config, &self.equips[eaten_index])?;
            if self.equips[eaten_index].wear_hero_id.unwrap_or(0) != 0 {
                return Err(anyhow!("Eaten equip {} is currently worn", eaten_id));
            }
            eaten.push(*eaten_id);
        }

        self.equips[index].level = Some(new_level);
        self.equips[index].exp = Some(0);
        let equip_clone = self.equips[index].clone();
        self.equips.retain(|e| !eaten.contains(&e.equip_id));
        self.dirty = true;

        let rs = EquipStrengthenRs {
            equip: Some(equip_clone),
            eat_equip_id: eaten,
            ..Default::default()
        };
        let events = vec![GameEvent::EquipStrengthen {
            role_id: 0,
            equip_config_id,
            new_level,
        }];
        Ok((rs.encode_to_vec(), events))
    }

    fn cmd_forging(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            EquipForgingRq::decode(payload).map_err(|e| anyhow!("Decode EquipForgingRq: {}", e))?;
        let equip_id = rq.equip_id;
        let index = self.equip_index(equip_id)?;
        Self::validate_equip_config(config, &self.equips[index])?;

        let cur_forging_level = self.equips[index].forging_level.unwrap_or(0);
        if cur_forging_level < 0 {
            return Err(anyhow!(
                "Invalid forging level {} for equip {}",
                cur_forging_level,
                equip_id
            ));
        }
        self.equips[index].forging_level = Some(cur_forging_level + 1);
        self.dirty = true;

        let rs = EquipForgingRs {
            equip: Some(self.equips[index].clone()),
            ..Default::default()
        };
        Ok((rs.encode_to_vec(), vec![]))
    }

    fn cmd_level_reset(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = EquipLevelResetRq::decode(payload)
            .map_err(|e| anyhow!("Decode EquipLevelResetRq: {}", e))?;
        let equip_id = rq.equip_id;
        let reset_type = rq.reset_type;
        let index = self.equip_index(equip_id)?;
        Self::validate_equip_config(config, &self.equips[index])?;

        match reset_type {
            1 => {
                if self.equips[index].level.unwrap_or(0) <= 0
                    && self.equips[index].exp.unwrap_or(0) <= 0
                {
                    return Err(anyhow!("Equip {} strengthen level already reset", equip_id));
                }
                self.equips[index].level = Some(0);
                self.equips[index].exp = Some(0);
            }
            2 => {
                if self.equips[index].forging_level.unwrap_or(0) <= 0 {
                    return Err(anyhow!("Equip {} forging level already reset", equip_id));
                }
                self.equips[index].forging_level = Some(0);
            }
            _ => return Err(anyhow!("Unknown reset_type: {}", reset_type)),
        }
        self.dirty = true;

        let rs = EquipLevelResetRs {
            reset_type: Some(reset_type),
            equip: Some(self.equips[index].clone()),
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
        self.dirty = false;
        info!(equips = self.equips.len(), "EquipSystem loaded");
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
        col::EQUIP
    }

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
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(
            func_type::EQUIP,
            func_tag::EQUIP,
            &self.to_proto(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use prost::Message;
    use shared::msg::{func_tag, func_type, GameMessage, ToFunctionClientBaseBytes};
    use shared::static_config::equip::{EquipConfig, StaticEquip, StaticEquipLv};
    use shared::static_config::StaticConfig;

    use super::*;
    use proto::slg::{
        EquipDataFunction, EquipForgingRq, EquipForgingRs, EquipLevelResetRq, EquipLevelResetRs,
        EquipStrengthenRq, EquipStrengthenRs, EquipUnfixRq, EquipUnfixRs, EquipWearRq, EquipWearRs,
        FunctionClientBase,
    };

    fn test_config() -> Arc<StaticConfig> {
        let mut equips = HashMap::new();
        equips.insert(1001, static_equip(1001, 1));
        equips.insert(1002, static_equip(1002, 2));
        equips.insert(1003, static_equip(1003, 1));

        Arc::new(StaticConfig {
            equip: EquipConfig {
                equips,
                equip_levels: vec![
                    static_level(1001, 1),
                    static_level(1001, 2),
                    static_level(1002, 1),
                    static_level(1003, 1),
                ],
                levels_by_equip_idx: HashMap::new(),
            },
            ..Default::default()
        })
    }

    fn static_equip(equip_id: i32, slot_type: i32) -> StaticEquip {
        StaticEquip {
            equip_id,
            name: format!("equip-{equip_id}"),
            unit_type: None,
            slot_type: Some(slot_type),
            quality: Some(1),
            can_provide_exp: None,
            attr: None,
            dec: None,
            icon: None,
        }
    }

    fn static_level(equip_id: i32, level: i32) -> StaticEquipLv {
        StaticEquipLv {
            id: equip_id * 100 + level,
            equip_id,
            level: level.to_string(),
            up_need_resource: None,
            attr: None,
        }
    }

    fn equip(equip_id: i32, config_id: i32) -> Equip {
        Equip {
            equip_id,
            equip_config_id: config_id,
            wear_hero_id: Some(0),
            level: Some(0),
            exp: Some(0),
            forging_level: Some(0),
            ..Default::default()
        }
    }

    #[test]
    fn persistence_roundtrip_preserves_equips_and_clears_dirty() {
        let mut system = EquipSystem::new();
        system.add_equip(equip(1, 1001));

        let bytes = system.save_to_bin().expect("save equip data");
        let mut loaded = EquipSystem::new();
        loaded.mark_dirty();
        loaded.load_from_bin(&bytes).expect("load equip data");

        assert_eq!(loaded.equips.len(), 1);
        assert_eq!(loaded.equips[0].equip_id, 1);
        assert_eq!(loaded.equips[0].equip_config_id, 1001);
        assert!(!loaded.is_dirty());
    }

    #[test]
    fn function_base_output_reflects_current_equip_state() {
        let mut system = EquipSystem::new();
        system.add_equip(equip(11, 1001));

        let bytes = system.to_function_base_bytes();
        let base = FunctionClientBase::decode(bytes.as_slice()).expect("decode function base");
        assert_eq!(base.r#type, Some(func_type::EQUIP));

        let decoded: EquipDataFunction =
            GameMessage::get_extension_from_bytes(&bytes, func_tag::EQUIP)
                .expect("decode equip extension");
        assert_eq!(decoded.equip.len(), 1);
        assert_eq!(decoded.equip[0].equip_id, 11);
        assert_eq!(decoded.equip[0].equip_config_id, 1001);
    }

    #[test]
    fn wear_equips_owned_item_and_returns_hero_refresh_equips() {
        let mut system = EquipSystem::new();
        system.add_equip(equip(21, 1001));
        system.clear_dirty();

        let rq = EquipWearRq {
            hero_id: 7,
            equip_id: Some(21),
        }
        .encode_to_vec();
        let (resp, _) = system
            .handle_command(4801, &rq, &test_config())
            .expect("wear equip");
        let rs = EquipWearRs::decode(resp.as_slice()).expect("decode wear response");

        assert_eq!(system.get_equip(21).unwrap().wear_hero_id, Some(7));
        assert!(system.is_dirty());
        assert_eq!(rs.equip.len(), 1);
        assert_eq!(rs.equip[0].equip_id, 21);
        assert_eq!(rs.equip[0].wear_hero_id, Some(7));
    }

    #[test]
    fn wear_rejects_same_slot_conflict_without_mutation() {
        let mut system = EquipSystem::new();
        let mut worn = equip(31, 1001);
        worn.wear_hero_id = Some(7);
        system.add_equip(worn);
        system.add_equip(equip(32, 1003));
        system.clear_dirty();

        let rq = EquipWearRq {
            hero_id: 7,
            equip_id: Some(32),
        }
        .encode_to_vec();
        let err = system
            .handle_command(4801, &rq, &test_config())
            .expect_err("slot conflict");

        assert!(err.to_string().contains("slot"));
        assert_eq!(system.get_equip(32).unwrap().wear_hero_id, Some(0));
        assert!(!system.is_dirty());
    }

    #[test]
    fn unfix_rejects_equipment_worn_by_another_hero() {
        let mut system = EquipSystem::new();
        let mut worn = equip(41, 1001);
        worn.wear_hero_id = Some(8);
        system.add_equip(worn);
        system.clear_dirty();

        let rq = EquipUnfixRq {
            hero_id: 7,
            equip_id: Some(41),
        }
        .encode_to_vec();
        let err = system
            .handle_command(4803, &rq, &test_config())
            .expect_err("wrong hero");

        assert!(err.to_string().contains("not worn by hero"));
        assert_eq!(system.get_equip(41).unwrap().wear_hero_id, Some(8));
        assert!(!system.is_dirty());
    }

    #[test]
    fn unfix_clears_matching_equipment_and_returns_refresh_equips() {
        let mut system = EquipSystem::new();
        let mut worn = equip(42, 1001);
        worn.wear_hero_id = Some(7);
        system.add_equip(worn);
        system.clear_dirty();

        let rq = EquipUnfixRq {
            hero_id: 7,
            equip_id: Some(42),
        }
        .encode_to_vec();
        let (resp, _) = system
            .handle_command(4803, &rq, &test_config())
            .expect("unfix equip");
        let rs = EquipUnfixRs::decode(resp.as_slice()).expect("decode unfix response");

        assert_eq!(system.get_equip(42).unwrap().wear_hero_id, Some(0));
        assert!(system.is_dirty());
        assert_eq!(rs.equip.len(), 1);
        assert_eq!(rs.equip[0].equip_id, 42);
        assert_eq!(rs.equip[0].wear_hero_id, Some(0));
    }

    #[test]
    fn strengthen_uses_next_level_config_and_removes_eaten_equips() {
        let mut system = EquipSystem::new();
        system.add_equip(equip(51, 1001));
        system.add_equip(equip(52, 1002));
        system.clear_dirty();

        let rq = EquipStrengthenRq {
            equip_id: 51,
            eat_equip_id: vec![52],
            eat_prop: vec![],
        }
        .encode_to_vec();
        let (resp, events) = system
            .handle_command(4805, &rq, &test_config())
            .expect("strengthen equip");
        let rs = EquipStrengthenRs::decode(resp.as_slice()).expect("decode strengthen response");

        assert_eq!(system.get_equip(51).unwrap().level, Some(1));
        assert!(system.get_equip(52).is_none());
        assert!(system.is_dirty());
        assert_eq!(rs.equip.unwrap().level, Some(1));
        assert_eq!(rs.eat_equip_id, vec![52]);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn strengthen_rejects_missing_next_level_config_without_mutation() {
        let mut system = EquipSystem::new();
        let mut maxed = equip(61, 1002);
        maxed.level = Some(1);
        system.add_equip(maxed);
        system.clear_dirty();

        let rq = EquipStrengthenRq {
            equip_id: 61,
            eat_equip_id: vec![],
            eat_prop: vec![],
        }
        .encode_to_vec();
        let err = system
            .handle_command(4805, &rq, &test_config())
            .expect_err("missing next level");

        assert!(err.to_string().contains("next level"));
        assert_eq!(system.get_equip(61).unwrap().level, Some(1));
        assert!(!system.is_dirty());
    }

    #[test]
    fn forging_rejects_missing_config_without_mutation() {
        let mut system = EquipSystem::new();
        system.add_equip(equip(71, 9999));
        system.clear_dirty();

        let rq = EquipForgingRq { equip_id: 71 }.encode_to_vec();
        let err = system
            .handle_command(4807, &rq, &test_config())
            .expect_err("missing config");

        assert!(err.to_string().contains("config"));
        assert_eq!(system.get_equip(71).unwrap().forging_level, Some(0));
        assert!(!system.is_dirty());
    }

    #[test]
    fn forging_increments_level_and_returns_equip_refresh() {
        let mut system = EquipSystem::new();
        system.add_equip(equip(72, 1001));
        system.clear_dirty();

        let rq = EquipForgingRq { equip_id: 72 }.encode_to_vec();
        let (resp, _) = system
            .handle_command(4807, &rq, &test_config())
            .expect("forging equip");
        let rs = EquipForgingRs::decode(resp.as_slice()).expect("decode forging response");

        assert_eq!(system.get_equip(72).unwrap().forging_level, Some(1));
        assert!(system.is_dirty());
        assert_eq!(rs.equip.unwrap().forging_level, Some(1));
    }

    #[test]
    fn level_reset_rejects_noop_target_without_mutation() {
        let mut system = EquipSystem::new();
        system.add_equip(equip(81, 1001));
        system.clear_dirty();

        let rq = EquipLevelResetRq {
            reset_type: 1,
            equip_id: 81,
        }
        .encode_to_vec();
        let err = system
            .handle_command(4809, &rq, &test_config())
            .expect_err("noop reset");

        assert!(err.to_string().contains("already reset"));
        assert_eq!(system.get_equip(81).unwrap().level, Some(0));
        assert!(!system.is_dirty());
    }

    #[test]
    fn level_reset_clears_strengthen_state_and_returns_change_info() {
        let mut system = EquipSystem::new();
        let mut upgraded = equip(82, 1001);
        upgraded.level = Some(2);
        upgraded.exp = Some(33);
        system.add_equip(upgraded);
        system.clear_dirty();

        let rq = EquipLevelResetRq {
            reset_type: 1,
            equip_id: 82,
        }
        .encode_to_vec();
        let (resp, _) = system
            .handle_command(4809, &rq, &test_config())
            .expect("reset level");
        let rs = EquipLevelResetRs::decode(resp.as_slice()).expect("decode reset response");

        assert_eq!(system.get_equip(82).unwrap().level, Some(0));
        assert_eq!(system.get_equip(82).unwrap().exp, Some(0));
        assert!(system.is_dirty());
        assert_eq!(rs.reset_type, Some(1));
        assert!(rs.change_info.is_some());
        assert_eq!(rs.equip.unwrap().level, Some(0));
    }
}
