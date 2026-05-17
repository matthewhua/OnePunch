//! 将领系统（HeroSystem）
//!
//! 对应 Java 版 HeroFunction，管理将领招募、升级、升星、技能、编队、兵营、医院。
//! 数据存储在 p_data.hero_func（protobuf HeroDataFunction）。

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use prost::Message;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{
    HeroDataFunction, Hero, BarracksData, HospitalDataPb, Formation,
    HeroCompositeRq, HeroCompositeRs,
    HeroLevelUpRq, HeroLevelUpRs,
    HeroTierUpRq, HeroTierUpRs,
    HeroSkillLevelUpRq, HeroSkillLevelUpRs,
    HeroRecruitRq, HeroRecruitRs,
    BarracksTrainingRq, BarracksTrainingRs,
    HospitalTreatRq, HospitalTreatRs,
    ChangeInfo,
};
use shared::persistence::col;
use shared::event::GameEvent;
use shared::static_config::StaticConfig;

/// 将领系统
pub struct HeroSystem {
    dirty: bool,
    /// heroId → Hero
    pub heroes: HashMap<i32, Hero>,
    /// 编队列表
    pub formations: Vec<Formation>,
    /// 兵营数据（buildId → BarracksData）
    pub barracks: HashMap<i32, BarracksData>,
    /// 医院数据
    pub hospital: Option<HospitalDataPb>,
    /// 共鸣上阵英雄ID列表
    pub top_heroes: Vec<i32>,
}

impl HeroSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            heroes: HashMap::new(),
            formations: Vec::new(),
            barracks: HashMap::new(),
            hospital: None,
            top_heroes: Vec::new(),
        }
    }

    /// 获取将领
    pub fn get_hero(&self, hero_id: i32) -> Option<&Hero> {
        self.heroes.get(&hero_id)
    }

    /// 添加将领（招募/合成获得）
    pub fn add_hero(&mut self, hero: Hero) {
        self.heroes.insert(hero.hero_id, hero);
        self.dirty = true;
    }

    pub fn set_formation_state(&mut self, formation_id: i32, state: i32) -> bool {
        let Some(formation) = self.formations.iter_mut().find(|f| f.id == formation_id) else {
            return false;
        };
        if formation.state == state {
            return false;
        }
        formation.state = state;
        self.dirty = true;
        true
    }

    /// 将领升级，返回触发的游戏事件列表
    pub fn level_up(&mut self, role_id: i64, hero_id: i32) -> Result<Vec<GameEvent>> {
        let hero = self.heroes.get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        // TODO: 检查资源消耗、等级上限
        hero.level += 1;
        hero.original_level += 1;
        let new_level = hero.level;
        self.dirty = true;

        Ok(vec![GameEvent::HeroLevelUp { role_id, hero_id, new_level }])
    }

    /// 将领升星，返回触发的游戏事件列表
    pub fn tier_up(&mut self, role_id: i64, hero_id: i32) -> Result<Vec<GameEvent>> {
        let hero = self.heroes.get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        // TODO: 检查碎片消耗、升星条件
        hero.tier += 1;
        let new_tier = hero.tier;
        self.dirty = true;

        Ok(vec![GameEvent::HeroTierUp { role_id, hero_id, new_tier }])
    }

    /// 命令分发入口（实现 PlayerSystem::handle_command）
    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        match cmd {
            2001 => self.cmd_composite(payload, config),
            2003 => self.cmd_level_up(payload, config),
            2005 => self.cmd_tier_up(payload, config),
            2007 => self.cmd_skill_level_up(payload, config),
            2101 => self.cmd_recruit(payload, config),
            2161 => self.cmd_barracks_training(payload, config),
            2173 => self.cmd_hospital_treat(payload, config),
            _ => Err(anyhow!("Unknown hero cmd: {}", cmd)),
        }
    }

    // ── 英雄合成（cmd=2001）────────────────────────────────────────────────────

    fn cmd_composite(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = HeroCompositeRq::decode(payload)
            .map_err(|e| anyhow!("Decode HeroCompositeRq: {}", e))?;
        let hero_id = rq.hero_id;

        // 检查将领配置是否存在
        let _hero_conf = config.hero.heroes.get(&hero_id)
            .ok_or_else(|| anyhow!("Hero config {} not found", hero_id))?;

        // TODO: 检查碎片数量是否足够（s_hero.needFragment）
        // 简化：直接合成
        if !self.heroes.contains_key(&hero_id) {
            let hero = Hero {
                hero_id,
                level: 1,
                original_level: 1,
                tier: 1,
                ..Default::default()
            };
            self.heroes.insert(hero_id, hero);
            self.dirty = true;
        }

        let hero = self.heroes.get(&hero_id).cloned().unwrap_or_default();
        let rs = HeroCompositeRs { hero: Some(hero), ..Default::default() };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 英雄升级（cmd=2003）────────────────────────────────────────────────────

    fn cmd_level_up(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = HeroLevelUpRq::decode(payload)
            .map_err(|e| anyhow!("Decode HeroLevelUpRq: {}", e))?;
        let hero_id = rq.hero_id;

        let hero = self.heroes.get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        // TODO: 检查等级上限（s_hero_lv.needBuilding）、消耗资源
        hero.level += 1;
        hero.original_level += 1;
        let new_level = hero.level;
        self.dirty = true;

        let hero_clone = hero.clone();
        let rs = HeroLevelUpRs { hero: Some(hero_clone), ..Default::default() };

        // 触发升级事件（驱动任务进度）
        // role_id 在系统层不可知，用 0 占位，PlayerActor 层会补充
        let events = vec![GameEvent::HeroLevelUp { role_id: 0, hero_id, new_level }];
        Ok((rs.encode_to_vec(), events))
    }

    // ── 英雄升星（cmd=2005）────────────────────────────────────────────────────

    fn cmd_tier_up(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = HeroTierUpRq::decode(payload)
            .map_err(|e| anyhow!("Decode HeroTierUpRq: {}", e))?;
        let hero_id = rq.hero_id;

        let hero = self.heroes.get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        // TODO: 检查碎片消耗（s_hero_star.upNeedResource）
        hero.tier += 1;
        let new_tier = hero.tier;
        self.dirty = true;

        let hero_clone = hero.clone();
        let rs = HeroTierUpRs { hero: Some(hero_clone), ..Default::default() };
        let events = vec![GameEvent::HeroTierUp { role_id: 0, hero_id, new_tier }];
        Ok((rs.encode_to_vec(), events))
    }

    // ── 技能升级（cmd=2007）────────────────────────────────────────────────────

    fn cmd_skill_level_up(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = HeroSkillLevelUpRq::decode(payload)
            .map_err(|e| anyhow!("Decode HeroSkillLevelUpRq: {}", e))?;
        let hero_id = rq.hero_id;
        let skill_id = rq.skill_id;

        let hero = self.heroes.get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        // 找到技能并升级
        if let Some(skill) = hero.skill.iter_mut().find(|s| s.v1 == skill_id) {
            skill.v2 += 1;
        } else {
            hero.skill.push(proto::slg::TwoInt { v1: skill_id, v2: 1 });
        }
        self.dirty = true;

        let hero_clone = hero.clone();
        let rs = HeroSkillLevelUpRs { hero: Some(hero_clone), ..Default::default() };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 英雄招募（cmd=2101）────────────────────────────────────────────────────

    fn cmd_recruit(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let _rq = HeroRecruitRq::decode(payload)
            .map_err(|e| anyhow!("Decode HeroRecruitRq: {}", e))?;

        // TODO: 实现招募抽卡逻辑（消耗道具/货币，随机获得英雄碎片）
        let rs = HeroRecruitRs {
            change_info: Some(ChangeInfo::default()),
            ..Default::default()
        };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 兵营训练（cmd=2161）────────────────────────────────────────────────────

    fn cmd_barracks_training(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = BarracksTrainingRq::decode(payload)
            .map_err(|e| anyhow!("Decode BarracksTrainingRq: {}", e))?;
        let build_id = rq.build_id;
        let number = rq.number;

        let barracks = self.barracks.entry(build_id).or_insert_with(|| BarracksData {
            build_id: Some(build_id),
            already_army: Some(0),
            training_army: Some(0),
            training_finish_time: Some(0),
        });

        // TODO: 检查资源消耗、训练时间计算
        let now = chrono::Utc::now().timestamp() as i32;
        barracks.training_army = Some(number);
        barracks.training_finish_time = Some(now + number * 10); // 简化：每兵10秒
        self.dirty = true;

        let barracks_clone = barracks.clone();
        let rs = BarracksTrainingRs { barracks_data: Some(barracks_clone), ..Default::default() };
        let events = vec![GameEvent::TroopTrain { role_id: 0, count: number }];
        Ok((rs.encode_to_vec(), events))
    }

    // ── 医院治疗（cmd=2173）────────────────────────────────────────────────────

    fn cmd_hospital_treat(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = HospitalTreatRq::decode(payload)
            .map_err(|e| anyhow!("Decode HospitalTreatRq: {}", e))?;
        let number = rq.number;

        let hospital = self.hospital.get_or_insert_with(HospitalDataPb::default);
        let now = chrono::Utc::now().timestamp() as i32;
        hospital.treating_army = Some(number);
        hospital.treating_finish_time = Some(now + number * 5); // 简化：每兵5秒
        self.dirty = true;

        let hospital_clone = hospital.clone();
        let rs = HospitalTreatRs { hospital_data: Some(hospital_clone), ..Default::default() };
        let events = vec![GameEvent::TroopHeal { role_id: 0, count: number }];
        Ok((rs.encode_to_vec(), events))
    }

    /// 构建 HeroDataFunction protobuf
    fn to_proto(&self) -> HeroDataFunction {
        HeroDataFunction {
            hero: self.heroes.values().cloned().collect(),
            formation: self.formations.clone(),
            barracks_data: self.barracks.values().cloned().collect(),
            hospital_data: self.hospital.clone(),
            top_heroes: self.top_heroes.clone(),
            ..Default::default()
        }
    }
}

impl PlayerSystem for HeroSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        let func = HeroDataFunction::decode(data)?;
        self.heroes = func.hero.into_iter().map(|h| (h.hero_id, h)).collect();
        self.formations = func.formation;
        self.barracks = func.barracks_data.into_iter()
            .filter_map(|b| b.build_id.map(|id| (id, b)))
            .collect();
        self.hospital = func.hospital_data;
        self.top_heroes = func.top_heroes;
        info!(heroes = self.heroes.len(), "HeroSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.to_proto().encode_to_vec())
    }

    fn is_dirty(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn column_name(&self) -> &'static str { col::HERO }

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
        HeroSystem::handle_command(self, cmd, payload, config)
    }
}

impl shared::msg::ToFunctionClientBaseBytes for HeroSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_type, func_tag};
        let func = self.to_proto();
        shared::msg::build_function_base_bytes_pub(func_type::HERO, func_tag::HERO, &func)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_formation_state_changes_state_and_marks_dirty() {
        let mut system = HeroSystem::new();
        system.formations.push(Formation {
            id: 7,
            state: 1,
            hero_id: vec![101],
            ..Default::default()
        });

        assert!(system.set_formation_state(7, 0));
        assert_eq!(system.formations[0].state, 0);
        assert!(system.dirty);
    }

    #[test]
    fn set_formation_state_noops_when_state_is_same_or_missing() {
        let mut system = HeroSystem::new();
        system.formations.push(Formation {
            id: 7,
            state: 0,
            hero_id: vec![101],
            ..Default::default()
        });

        assert!(!system.set_formation_state(7, 0));
        assert!(!system.dirty);
        assert!(!system.set_formation_state(8, 0));
        assert!(!system.dirty);
    }
}
