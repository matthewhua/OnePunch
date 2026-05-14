//! 将领系统（HeroSystem）
//!
//! 对应 Java 版 HeroFunction，管理将领招募、升级、升星、技能、编队、兵营、医院。
//! 数据存储在 p_data.hero_func（protobuf HeroDataFunction）。

use anyhow::{anyhow, Result};
use prost::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{
    BarracksData, BarracksTrainingRq, BarracksTrainingRs, ChangeInfo, Formation, Hero,
    HeroCompositeRq, HeroCompositeRs, HeroDataFunction, HeroLevelUpRq, HeroLevelUpRs,
    HeroRecruitRq, HeroRecruitRs, HeroSkillLevelUpRq, HeroSkillLevelUpRs, HeroTierUpRq,
    HeroTierUpRs, HospitalDataPb, HospitalTreatRq, HospitalTreatRs, RecruitData, TwoInt,
};
use shared::event::GameEvent;
use shared::persistence::col;
use shared::static_config::StaticConfig;

/// 将领系统
pub struct HeroSystem {
    dirty: bool,
    /// heroId → Hero
    pub heroes: HashMap<i32, Hero>,
    /// 编队列表
    pub formations: Vec<Formation>,
    /// recruitType -> RecruitData
    pub recruits: HashMap<i32, RecruitData>,
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
            recruits: HashMap::new(),
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

    fn validate_positive_id(name: &str, id: i32) -> Result<()> {
        if id <= 0 {
            return Err(anyhow!("Invalid {}: {}", name, id));
        }
        Ok(())
    }

    fn new_hero(hero_id: i32) -> Hero {
        Hero {
            hero_id,
            level: 1,
            original_level: 1,
            tier: 1,
            ..Default::default()
        }
    }

    fn has_next_level(config: &StaticConfig, next_level: i32) -> bool {
        config
            .hero
            .hero_levels
            .iter()
            .any(|lv| lv.level == next_level)
    }

    fn has_next_tier(config: &StaticConfig, hero_id: i32, next_tier: i32) -> bool {
        config
            .hero
            .hero_stars
            .iter()
            .any(|star| star.hero_id == hero_id && star.tier == next_tier)
    }

    /// 将领升级，返回触发的游戏事件列表
    pub fn level_up(&mut self, role_id: i64, hero_id: i32) -> Result<Vec<GameEvent>> {
        let hero = self
            .heroes
            .get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        // TODO: 检查资源消耗、等级上限
        hero.level += 1;
        hero.original_level += 1;
        let new_level = hero.level;
        self.dirty = true;

        Ok(vec![GameEvent::HeroLevelUp {
            role_id,
            hero_id,
            new_level,
        }])
    }

    /// 将领升星，返回触发的游戏事件列表
    pub fn tier_up(&mut self, role_id: i64, hero_id: i32) -> Result<Vec<GameEvent>> {
        let hero = self
            .heroes
            .get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        // TODO: 检查碎片消耗、升星条件
        hero.tier += 1;
        let new_tier = hero.tier;
        self.dirty = true;

        Ok(vec![GameEvent::HeroTierUp {
            role_id,
            hero_id,
            new_tier,
        }])
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
        Self::validate_positive_id("hero_id", hero_id)?;

        // 检查将领配置是否存在
        let _hero_conf = config
            .hero
            .heroes
            .get(&hero_id)
            .ok_or_else(|| anyhow!("Hero config {} not found", hero_id))?;

        // TODO: 检查碎片数量是否足够（s_hero.needFragment）
        // 简化：直接合成
        if self.heroes.contains_key(&hero_id) {
            return Err(anyhow!("Hero {} already owned", hero_id));
        }
        self.heroes.insert(hero_id, Self::new_hero(hero_id));
        self.dirty = true;

        let hero = self.heroes.get(&hero_id).cloned().unwrap_or_default();
        let rs = HeroCompositeRs {
            hero: Some(hero),
            ..Default::default()
        };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 英雄升级（cmd=2003）────────────────────────────────────────────────────

    fn cmd_level_up(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            HeroLevelUpRq::decode(payload).map_err(|e| anyhow!("Decode HeroLevelUpRq: {}", e))?;
        let hero_id = rq.hero_id;
        Self::validate_positive_id("hero_id", hero_id)?;

        let hero = self
            .heroes
            .get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;
        let next_level = hero.level + 1;
        if !Self::has_next_level(config, next_level) {
            return Err(anyhow!(
                "No next hero level config for level {}",
                next_level
            ));
        }

        // TODO: 检查等级上限（s_hero_lv.needBuilding）、消耗资源
        hero.level = next_level;
        hero.original_level = next_level;
        let new_level = hero.level;
        self.dirty = true;

        let hero_clone = hero.clone();
        let rs = HeroLevelUpRs {
            hero: Some(hero_clone),
            ..Default::default()
        };

        // 触发升级事件（驱动任务进度）
        // role_id 在系统层不可知，用 0 占位，PlayerActor 层会补充
        let events = vec![GameEvent::HeroLevelUp {
            role_id: 0,
            hero_id,
            new_level,
        }];
        Ok((rs.encode_to_vec(), events))
    }

    // ── 英雄升星（cmd=2005）────────────────────────────────────────────────────

    fn cmd_tier_up(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            HeroTierUpRq::decode(payload).map_err(|e| anyhow!("Decode HeroTierUpRq: {}", e))?;
        let hero_id = rq.hero_id;
        Self::validate_positive_id("hero_id", hero_id)?;

        let hero = self
            .heroes
            .get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;
        let next_tier = hero.tier + 1;
        if !Self::has_next_tier(config, hero_id, next_tier) {
            return Err(anyhow!(
                "No next hero tier config for hero {} tier {}",
                hero_id,
                next_tier
            ));
        }

        // TODO: 检查碎片消耗（s_hero_star.upNeedResource）
        hero.tier = next_tier;
        let new_tier = hero.tier;
        self.dirty = true;

        let hero_clone = hero.clone();
        let rs = HeroTierUpRs {
            hero: Some(hero_clone),
            ..Default::default()
        };
        let events = vec![GameEvent::HeroTierUp {
            role_id: 0,
            hero_id,
            new_tier,
        }];
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
        Self::validate_positive_id("hero_id", hero_id)?;
        Self::validate_positive_id("skill_id", skill_id)?;

        let hero = self
            .heroes
            .get_mut(&hero_id)
            .ok_or_else(|| anyhow!("Hero {} not found", hero_id))?;

        let skill = hero
            .skill
            .iter_mut()
            .find(|s| s.v1 == skill_id)
            .ok_or_else(|| anyhow!("Hero {} skill {} not unlocked", hero_id, skill_id))?;
        if skill.v2 <= 0 {
            return Err(anyhow!("Invalid skill {} level {}", skill_id, skill.v2));
        }
        if skill.v2 >= hero.level {
            return Err(anyhow!(
                "Skill {} level {} has reached hero {} level bound {}",
                skill_id,
                skill.v2,
                hero_id,
                hero.level
            ));
        }
        skill.v2 += 1;
        self.dirty = true;

        let hero_clone = hero.clone();
        let rs = HeroSkillLevelUpRs {
            hero: Some(hero_clone),
            ..Default::default()
        };
        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 英雄招募（cmd=2101）────────────────────────────────────────────────────

    fn cmd_recruit(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            HeroRecruitRq::decode(payload).map_err(|e| anyhow!("Decode HeroRecruitRq: {}", e))?;
        let recruit_type = rq.recruit_type;
        let recruit_num_index = rq.recruit_num_index;
        Self::validate_positive_id("recruit_type", recruit_type)?;
        Self::validate_positive_id("recruit_num_index", recruit_num_index)?;
        if recruit_type > 10 {
            return Err(anyhow!("Invalid recruit_type: {}", recruit_type));
        }
        if recruit_num_index > 10 {
            return Err(anyhow!("Invalid recruit_num_index: {}", recruit_num_index));
        }
        if let Some(cost_type) = rq.recruit_cost_type {
            if !matches!(cost_type, 1 | 2) {
                return Err(anyhow!("Invalid recruit_cost_type: {}", cost_type));
            }
        }

        let recruit = self
            .recruits
            .entry(recruit_type)
            .or_insert_with(|| RecruitData {
                recruit_type,
                used_free_count: Some(0),
                next_free_time: Some(0),
                total_schedule: Some(0),
                today_recruited_count: Some(0),
            });
        let today_count = recruit.today_recruited_count.unwrap_or(0) + recruit_num_index;
        let total_schedule = recruit.total_schedule.unwrap_or(0) + recruit_num_index;
        recruit.today_recruited_count = Some(today_count);
        recruit.total_schedule = Some(total_schedule);
        self.dirty = true;

        let rs = HeroRecruitRs {
            change_info: Some(ChangeInfo::default()),
            recruit_type: Some(recruit_type),
            recruit_num_index: Some(recruit_num_index),
            used_free_count: recruit.used_free_count,
            next_free_time: recruit.next_free_time,
            today_recruited_count: Some(today_count),
            schedule_change: vec![TwoInt {
                v1: recruit_type,
                v2: total_schedule,
            }],
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
        let training_type = rq.r#type;
        let number = rq.number;
        Self::validate_positive_id("build_id", build_id)?;
        if number <= 0 {
            return Err(anyhow!("Invalid barracks training number: {}", number));
        }
        if !matches!(training_type, 1 | 2) {
            return Err(anyhow!("Invalid barracks training type: {}", training_type));
        }

        let barracks = self
            .barracks
            .entry(build_id)
            .or_insert_with(|| BarracksData {
                build_id: Some(build_id),
                already_army: Some(0),
                training_army: Some(0),
                training_finish_time: Some(0),
            });

        if training_type == 2 {
            barracks.already_army = Some(barracks.already_army.unwrap_or(0) + number);
            barracks.training_army = Some(0);
            barracks.training_finish_time = Some(0);
        } else {
            if barracks.training_army.unwrap_or(0) > 0 {
                return Err(anyhow!("Barracks {} is already training", build_id));
            }
            let now = chrono::Utc::now().timestamp() as i32;
            barracks.training_army = Some(number);
            barracks.training_finish_time = Some(now + number * 10); // 简化：每兵10秒
        }
        self.dirty = true;

        let barracks_clone = barracks.clone();
        let rs = BarracksTrainingRs {
            barracks_data: Some(barracks_clone),
            ..Default::default()
        };
        let events = vec![GameEvent::TroopTrain {
            role_id: 0,
            count: number,
        }];
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
        let treat_type = rq.r#type;
        let number = rq.number;
        if !matches!(treat_type, 1 | 2) {
            return Err(anyhow!("Invalid hospital treat type: {}", treat_type));
        }
        if number <= 0 {
            return Err(anyhow!("Invalid hospital treat number: {}", number));
        }

        let hospital = self.hospital.get_or_insert_with(HospitalDataPb::default);
        let injured = hospital.injured_army.unwrap_or(0);
        if number > injured {
            return Err(anyhow!(
                "Cannot treat {} army; only {} injured army available",
                number,
                injured
            ));
        }
        if treat_type == 2 {
            hospital.injured_army = Some(injured - number);
            hospital.treating_army = Some(0);
            hospital.treating_finish_time = Some(0);
        } else {
            if hospital.treating_army.unwrap_or(0) > 0 {
                return Err(anyhow!("Hospital is already treating"));
            }
            let now = chrono::Utc::now().timestamp() as i32;
            hospital.injured_army = Some(injured - number);
            hospital.treating_army = Some(number);
            hospital.treating_finish_time = Some(now + number * 5); // 简化：每兵5秒
        }
        self.dirty = true;

        let hospital_clone = hospital.clone();
        let rs = HospitalTreatRs {
            hospital_data: Some(hospital_clone),
            ..Default::default()
        };
        let events = vec![GameEvent::TroopHeal {
            role_id: 0,
            count: number,
        }];
        Ok((rs.encode_to_vec(), events))
    }

    /// 构建 HeroDataFunction protobuf
    fn to_proto(&self) -> HeroDataFunction {
        HeroDataFunction {
            hero: self.heroes.values().cloned().collect(),
            hero_recruit: self.recruits.values().cloned().collect(),
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
        self.recruits = func
            .hero_recruit
            .into_iter()
            .map(|r| (r.recruit_type, r))
            .collect();
        self.formations = func.formation;
        self.barracks = func
            .barracks_data
            .into_iter()
            .filter_map(|b| b.build_id.map(|id| (id, b)))
            .collect();
        self.hospital = func.hospital_data;
        self.top_heroes = func.top_heroes;
        self.dirty = false;
        info!(heroes = self.heroes.len(), "HeroSystem loaded");
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
        col::HERO
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
        HeroSystem::handle_command(self, cmd, payload, config)
    }
}

impl shared::msg::ToFunctionClientBaseBytes for HeroSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        let func = self.to_proto();
        shared::msg::build_function_base_bytes_pub(func_type::HERO, func_tag::HERO, &func)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::{FunctionClientBase, TwoInt};
    use shared::msg::{func_tag, func_type, GameMessage, ToFunctionClientBaseBytes};
    use shared::static_config::hero::{StaticHero, StaticHeroLv, StaticHeroStar};

    fn encode<M: Message>(msg: M) -> Vec<u8> {
        msg.encode_to_vec()
    }

    fn config_with_hero() -> Arc<StaticConfig> {
        let mut config = StaticConfig::default();
        config.hero.heroes.insert(
            1001,
            StaticHero {
                hero_id: 1001,
                name: "hero".to_string(),
                unit_type: "infantry".to_string(),
                hero_type: 1,
                atk_type: 1,
                state: None,
                quality: "R".to_string(),
                line: Some(1),
                hero_equip_id: None,
                best_position: None,
                attr: String::new(),
                attr_lv_small_up: None,
                attr_lv_large_up: None,
                td_attr: None,
                td_attr_up: None,
                fragment_id: None,
                need_fragment: None,
                exchange_need_res: None,
                fragment_get: None,
                battle_skill: Some("3001".to_string()),
                outside_skill: None,
                star_skill: None,
                gender: None,
            },
        );
        config.hero.hero_levels.push(StaticHeroLv {
            id: 1,
            level: 2,
            need_building: String::new(),
            up_need_resource: None,
        });
        config.hero.hero_stars.push(StaticHeroStar {
            id: 1,
            hero_id: 1001,
            tier: 2,
            tier_display: 2,
            star_display: 1,
            quality: "R".to_string(),
            up_need_resource: None,
            attr_star_up: None,
            td_attr_up: None,
            skill_unlock: None,
            skill_lv: None,
        });
        Arc::new(config)
    }

    fn hero(id: i32) -> Hero {
        Hero {
            hero_id: id,
            level: 1,
            original_level: 1,
            tier: 1,
            skill: vec![TwoInt { v1: 3001, v2: 1 }],
            ..Default::default()
        }
    }

    #[test]
    fn persistence_roundtrip_and_function_base_preserve_hero_state() {
        let mut system = HeroSystem::new();
        system.add_hero(hero(1001));
        system.barracks.insert(
            11,
            BarracksData {
                build_id: Some(11),
                already_army: Some(3),
                training_army: Some(2),
                training_finish_time: Some(10),
            },
        );
        system.hospital = Some(HospitalDataPb {
            injured_army: Some(7),
            treating_army: Some(1),
            treating_finish_time: Some(12),
            ..Default::default()
        });
        system.top_heroes.push(1001);

        let saved = system.save_to_bin().expect("save hero system");
        let mut loaded = HeroSystem::new();
        loaded.load_from_bin(&saved).expect("load hero system");

        assert_eq!(loaded.heroes.len(), 1);
        assert!(loaded.heroes.contains_key(&1001));
        assert_eq!(loaded.barracks.get(&11).unwrap().already_army, Some(3));
        assert_eq!(loaded.hospital.as_ref().unwrap().injured_army, Some(7));
        assert_eq!(loaded.top_heroes, vec![1001]);

        let bytes = loaded.to_function_base_bytes();
        let base = FunctionClientBase::decode(bytes.as_slice()).expect("decode function base");
        assert_eq!(base.r#type, Some(func_type::HERO));
        let decoded: HeroDataFunction =
            GameMessage::get_extension_from_bytes(&bytes, func_tag::HERO)
                .expect("decode hero extension");
        assert_eq!(decoded.hero.len(), 1);
        assert_eq!(decoded.hero[0].hero_id, 1001);
    }

    #[test]
    fn composite_rejects_duplicate_ownership_without_mutation() {
        let config = config_with_hero();
        let mut system = HeroSystem::new();

        let (resp, _) = system
            .handle_command_with_events(2001, &encode(HeroCompositeRq { hero_id: 1001 }), &config)
            .expect("composite new hero");
        let added = HeroCompositeRs::decode(resp.as_slice())
            .unwrap()
            .hero
            .unwrap();
        assert_eq!(added.hero_id, 1001);
        assert!(system.heroes.contains_key(&1001));
        assert!(system.is_dirty());

        system.clear_dirty();
        let err = system
            .handle_command_with_events(2001, &encode(HeroCompositeRq { hero_id: 1001 }), &config)
            .expect_err("duplicate composite");

        assert!(err.to_string().contains("already owned"));
        assert_eq!(system.heroes.len(), 1);
        assert!(!system.is_dirty());
    }

    #[test]
    fn composite_rejects_duplicate_seeded_ownership_without_mutation() {
        let config = config_with_hero();
        let mut system = HeroSystem::new();
        system.add_hero(hero(1001));
        system.clear_dirty();

        let err = system
            .handle_command_with_events(2001, &encode(HeroCompositeRq { hero_id: 1001 }), &config)
            .expect_err("duplicate composite");

        assert!(err.to_string().contains("already owned"));
        assert_eq!(system.heroes.len(), 1);
        assert!(!system.is_dirty());
    }

    #[test]
    fn level_tier_and_skill_up_validate_owned_targets_and_bounds() {
        let config = config_with_hero();
        let mut system = HeroSystem::new();
        system.add_hero(hero(1001));
        system.clear_dirty();

        let (resp, events) = system
            .handle_command_with_events(2003, &encode(HeroLevelUpRq { hero_id: 1001 }), &config)
            .expect("level up");
        let leveled = HeroLevelUpRs::decode(resp.as_slice())
            .unwrap()
            .hero
            .unwrap();
        assert_eq!(leveled.level, 2);
        assert_eq!(events.len(), 1);

        let err = system
            .handle_command_with_events(2003, &encode(HeroLevelUpRq { hero_id: 1001 }), &config)
            .expect_err("level cap");
        assert!(err.to_string().contains("No next hero level"));

        let (resp, _) = system
            .handle_command_with_events(2005, &encode(HeroTierUpRq { hero_id: 1001 }), &config)
            .expect("tier up");
        let tiered = HeroTierUpRs::decode(resp.as_slice()).unwrap().hero.unwrap();
        assert_eq!(tiered.tier, 2);

        let resp = <HeroSystem as PlayerSystem>::handle_command(
            &mut system,
            2007,
            &encode(HeroSkillLevelUpRq {
                hero_id: 1001,
                skill_id: 3001,
            }),
            &config,
        )
        .expect("skill up");
        let skilled = HeroSkillLevelUpRs::decode(resp.as_slice())
            .unwrap()
            .hero
            .unwrap();
        assert_eq!(skilled.skill[0], TwoInt { v1: 3001, v2: 2 });

        let err = system
            .handle_command_with_events(
                2007,
                &encode(HeroSkillLevelUpRq {
                    hero_id: 1001,
                    skill_id: 9999,
                }),
                &config,
            )
            .expect_err("unknown skill");
        assert!(err.to_string().contains("not unlocked"));
    }

    #[test]
    fn recruit_response_echoes_request_and_updates_smoke_refresh_fields() {
        let config = config_with_hero();
        let mut system = HeroSystem::new();

        let (resp, _) = system
            .handle_command_with_events(
                2101,
                &encode(HeroRecruitRq {
                    recruit_type: 1,
                    recruit_num_index: 1,
                    recruit_cost_type: Some(1),
                }),
                &config,
            )
            .expect("recruit");
        let rs = HeroRecruitRs::decode(resp.as_slice()).expect("decode recruit");

        assert!(rs.change_info.is_some());
        assert_eq!(rs.recruit_type, Some(1));
        assert_eq!(rs.recruit_num_index, Some(1));
        assert_eq!(rs.today_recruited_count, Some(1));
        assert_eq!(rs.schedule_change.len(), 1);
        assert_eq!(rs.schedule_change[0], TwoInt { v1: 1, v2: 1 });
    }

    #[test]
    fn barracks_training_rejects_invalid_type_and_supports_immediate_complete() {
        let config = config_with_hero();
        let mut system = HeroSystem::new();

        let err = system
            .handle_command_with_events(
                2161,
                &encode(BarracksTrainingRq {
                    build_id: 1,
                    r#type: 9,
                    number: 5,
                }),
                &config,
            )
            .expect_err("invalid training type");
        assert!(err.to_string().contains("Invalid barracks training type"));
        assert!(system.barracks.is_empty());

        let (resp, _) = system
            .handle_command_with_events(
                2161,
                &encode(BarracksTrainingRq {
                    build_id: 1,
                    r#type: 2,
                    number: 5,
                }),
                &config,
            )
            .expect("immediate training");
        let trained = BarracksTrainingRs::decode(resp.as_slice())
            .unwrap()
            .barracks_data
            .unwrap();
        assert_eq!(trained.already_army, Some(5));
        assert_eq!(trained.training_army, Some(0));
    }

    #[test]
    fn hospital_treat_rejects_invalid_inputs_and_clamps_to_injured_army() {
        let config = config_with_hero();
        let mut system = HeroSystem::new();
        system.hospital = Some(HospitalDataPb {
            injured_army: Some(3),
            ..Default::default()
        });

        let err = system
            .handle_command_with_events(
                2173,
                &encode(HospitalTreatRq {
                    r#type: 3,
                    number: 1,
                }),
                &config,
            )
            .expect_err("invalid treat type");
        assert!(err.to_string().contains("Invalid hospital treat type"));

        let err = system
            .handle_command_with_events(
                2173,
                &encode(HospitalTreatRq {
                    r#type: 1,
                    number: 4,
                }),
                &config,
            )
            .expect_err("too many injured");
        assert!(err.to_string().contains("only 3 injured"));

        let (resp, _) = system
            .handle_command_with_events(
                2173,
                &encode(HospitalTreatRq {
                    r#type: 2,
                    number: 3,
                }),
                &config,
            )
            .expect("immediate treat");
        let hospital = HospitalTreatRs::decode(resp.as_slice())
            .unwrap()
            .hospital_data
            .unwrap();
        assert_eq!(hospital.injured_army, Some(0));
        assert_eq!(hospital.treating_army, Some(0));
    }
}
