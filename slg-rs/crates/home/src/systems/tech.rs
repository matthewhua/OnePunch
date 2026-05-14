//! Technology system.
//!
//! Stores TechnologyDataFunction in p_data.technology_func and handles the
//! research, speed-up, cancel, and completion paths owned by this module.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use prost::Message;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{
    TechnologyCancelRq, TechnologyCancelRs, TechnologyDataFunction, TechnologyNode,
    TechnologyResearchQueue, TechnologyResearchRq, TechnologyResearchRs, TechnologySpeedUpRq,
    TechnologySpeedUpRs,
};
use shared::event::GameEvent;
use shared::persistence::col;
use shared::static_config::{tech::StaticTechLv, StaticConfig};

const DEFAULT_RESEARCH_SECONDS: i64 = 60;
const SPEED_UP_SECONDS_PER_ITEM: i64 = 300;

pub struct TechSystem {
    dirty: bool,
    pub nodes: Vec<TechnologyNode>,
    pub queue: Vec<TechnologyResearchQueue>,
}

impl TechSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            nodes: Vec::new(),
            queue: Vec::new(),
        }
    }

    pub fn get_tech_level(&self, tech_id: i32) -> i32 {
        self.nodes
            .iter()
            .filter(|n| n.technology_id == Some(tech_id))
            .filter_map(|n| n.level)
            .max()
            .unwrap_or(0)
    }

    fn get_tech_level_at_stage(&self, tech_id: i32, stage: i32) -> i32 {
        self.nodes
            .iter()
            .find(|n| n.technology_id == Some(tech_id) && n.stage == Some(stage))
            .and_then(|n| n.level)
            .unwrap_or(0)
    }

    pub fn is_researching(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn check_research_complete(&mut self, role_id: i64, now_secs: i64) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let mut completed: Vec<TechnologyResearchQueue> = Vec::new();

        self.queue.retain(|q| {
            let done = q.complete_time.map(|t| t <= now_secs).unwrap_or(false);
            if done {
                completed.push(q.clone());
            }
            !done
        });

        for q in completed {
            let tech_id = q.technology_id.unwrap_or(0);
            let research_level = q.research_level.unwrap_or(1);

            self.apply_completed_queue(&q);
            self.dirty = true;
            events.push(GameEvent::TechResearchComplete {
                role_id,
                tech_id,
                new_level: research_level,
            });
        }

        events
    }

    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        match cmd {
            4201 => self.cmd_research(payload, config),
            4203 => self.cmd_speed_up(payload),
            4205 => self.cmd_cancel(payload),
            _ => Err(anyhow!("Unknown tech cmd: {}", cmd)),
        }
    }

    fn cmd_research(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = TechnologyResearchRq::decode(payload)
            .map_err(|e| anyhow!("Decode TechnologyResearchRq: {}", e))?;
        let tech_id = rq.technology_id;
        let stage = rq.stage;
        let research_type = rq.r#type;

        if tech_id <= 0 {
            return Err(anyhow!("Invalid tech id {}", tech_id));
        }
        if stage <= 0 {
            return Err(anyhow!("Invalid tech stage {}", stage));
        }
        if !matches!(research_type, 1 | 2) {
            return Err(anyhow!("Invalid tech research type {}", research_type));
        }
        if self
            .queue
            .iter()
            .any(|q| q.technology_id == Some(tech_id) && q.research_stage == Some(stage))
        {
            return Err(anyhow!("Tech {} stage {} already in queue", tech_id, stage));
        }

        let next_level = self.get_tech_level_at_stage(tech_id, stage) + 1;
        let target = self.validate_research_target(tech_id, stage, next_level, config)?;
        let now = chrono::Utc::now().timestamp();

        if research_type == 2 {
            let q = TechnologyResearchQueue {
                technology_id: Some(tech_id),
                research_level: Some(next_level),
                complete_time: Some(now),
                research_stage: Some(stage),
                ..Default::default()
            };
            self.apply_completed_queue(&q);
            self.dirty = true;
            return Ok((
                TechnologyResearchRs::default().encode_to_vec(),
                vec![GameEvent::TechResearchComplete {
                    role_id: 0,
                    tech_id,
                    new_level: next_level,
                }],
            ));
        }

        // Resource payment is intentionally not done here; PlayerActor owns cross-system economy.
        let research_time = target
            .map(|t| t.up_time.max(0) as i64)
            .unwrap_or(DEFAULT_RESEARCH_SECONDS);
        self.queue.push(TechnologyResearchQueue {
            technology_id: Some(tech_id),
            research_level: Some(next_level),
            complete_time: Some(now + research_time),
            research_stage: Some(stage),
            ..Default::default()
        });
        self.dirty = true;

        Ok((TechnologyResearchRs::default().encode_to_vec(), Vec::new()))
    }

    fn cmd_speed_up(&mut self, payload: &[u8]) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = TechnologySpeedUpRq::decode(payload)
            .map_err(|e| anyhow!("Decode TechnologySpeedUpRq: {}", e))?;
        let tech_id = rq.technology_id;
        let stage = rq.stage;
        let speed_type = rq.r#type;

        if tech_id <= 0 {
            return Err(anyhow!("Invalid tech id {}", tech_id));
        }
        if stage <= 0 {
            return Err(anyhow!("Invalid tech stage {}", stage));
        }
        if !matches!(speed_type, 1 | 2 | 3) {
            return Err(anyhow!("Invalid tech speed type {}", speed_type));
        }

        let pos = self
            .queue
            .iter()
            .position(|q| q.technology_id == Some(tech_id) && q.research_stage == Some(stage))
            .ok_or_else(|| anyhow!("No active tech queue for {} stage {}", tech_id, stage))?;

        if speed_type == 2 || speed_type == 3 {
            let q = self.queue.remove(pos);
            let new_level = q.research_level.unwrap_or(1);
            self.apply_completed_queue(&q);
            self.dirty = true;
            return Ok((
                TechnologySpeedUpRs::default().encode_to_vec(),
                vec![GameEvent::TechResearchComplete {
                    role_id: 0,
                    tech_id,
                    new_level,
                }],
            ));
        }

        let prop_id = rq
            .prop_id
            .ok_or_else(|| anyhow!("TechnologySpeedUpRq missing propId"))?;
        let use_number = rq
            .use_number
            .ok_or_else(|| anyhow!("TechnologySpeedUpRq missing useNumber"))?;
        if prop_id <= 0 {
            return Err(anyhow!("Invalid speed prop id {}", prop_id));
        }
        if use_number <= 0 {
            return Err(anyhow!("Invalid speed prop count {}", use_number));
        }

        let reduce_secs = i64::from(use_number) * SPEED_UP_SECONDS_PER_ITEM;
        let q = &mut self.queue[pos];
        let complete_time = q.complete_time.unwrap_or(0).saturating_sub(reduce_secs);
        q.complete_time = Some(complete_time);
        self.dirty = true;

        Ok((TechnologySpeedUpRs::default().encode_to_vec(), Vec::new()))
    }

    fn cmd_cancel(&mut self, payload: &[u8]) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = TechnologyCancelRq::decode(payload)
            .map_err(|e| anyhow!("Decode TechnologyCancelRq: {}", e))?;
        let tech_id = rq.technology_id;
        let stage = rq.stage;

        if tech_id <= 0 {
            return Err(anyhow!("Invalid tech id {}", tech_id));
        }
        if stage <= 0 {
            return Err(anyhow!("Invalid tech stage {}", stage));
        }

        let pos = self
            .queue
            .iter()
            .position(|q| q.technology_id == Some(tech_id) && q.research_stage == Some(stage))
            .ok_or_else(|| anyhow!("No active tech queue for {} stage {}", tech_id, stage))?;
        self.queue.remove(pos);
        self.dirty = true;

        // No refund is issued here because resource ownership lives outside TechSystem.
        Ok((TechnologyCancelRs::default().encode_to_vec(), Vec::new()))
    }

    fn validate_research_target<'a>(
        &self,
        tech_id: i32,
        stage: i32,
        level: i32,
        config: &'a StaticConfig,
    ) -> Result<Option<&'a StaticTechLv>> {
        if config.tech.tech_levels.is_empty() {
            return Ok(None);
        }

        let target = config
            .tech
            .tech_levels
            .values()
            .find(|t| t.tech_id == tech_id && tech_stage(t) == Some(stage) && t.level == level)
            .ok_or_else(|| {
                anyhow!(
                    "No tech config for tech {} stage {} level {}",
                    tech_id,
                    stage,
                    level
                )
            })?;

        for prereq_id in parse_config_ids(target.need_tech.as_deref()) {
            let prereq = config
                .tech
                .tech_levels
                .get(&prereq_id)
                .ok_or_else(|| anyhow!("Missing prereq tech config id {}", prereq_id))?;
            let prereq_stage = tech_stage(prereq)
                .ok_or_else(|| anyhow!("Invalid prereq tech stage config id {}", prereq_id))?;
            let owned_level = self.get_tech_level_at_stage(prereq.tech_id, prereq_stage);
            if owned_level < prereq.level {
                return Err(anyhow!(
                    "Tech {} stage {} level {} requires config id {}",
                    tech_id,
                    stage,
                    level,
                    prereq_id
                ));
            }
        }

        Ok(Some(target))
    }

    fn apply_completed_queue(&mut self, q: &TechnologyResearchQueue) {
        let tech_id = q.technology_id.unwrap_or(0);
        let research_level = q.research_level.unwrap_or(1);
        let stage = q.research_stage;

        if let Some(node) = self
            .nodes
            .iter_mut()
            .find(|n| n.technology_id == Some(tech_id) && n.stage == stage)
        {
            node.level = Some(research_level);
        } else {
            self.nodes.push(TechnologyNode {
                technology_id: Some(tech_id),
                level: Some(research_level),
                stage,
            });
        }
    }

    fn to_proto(&self) -> TechnologyDataFunction {
        TechnologyDataFunction {
            node: self.nodes.clone(),
            queue: self.queue.clone(),
        }
    }
}

fn tech_stage(t: &StaticTechLv) -> Option<i32> {
    t.tech_stage.trim().parse::<i32>().ok()
}

fn parse_config_ids(raw: Option<&str>) -> Vec<i32> {
    let Some(raw) = raw.map(str::trim) else {
        return Vec::new();
    };
    if raw.is_empty() || raw == "0" || raw.eq_ignore_ascii_case("null") {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    collect_json_ints(&value, &mut ids);
    ids.into_iter().filter(|id| *id > 0).collect()
}

fn collect_json_ints(value: &serde_json::Value, ids: &mut Vec<i32>) {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(id) = n.as_i64().and_then(|v| i32::try_from(v).ok()) {
                ids.push(id);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_json_ints(item, ids);
            }
        }
        _ => {}
    }
}

impl PlayerSystem for TechSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        let func = TechnologyDataFunction::decode(data)?;
        self.nodes = func.node;
        self.queue = func.queue;
        self.dirty = false;
        info!(
            techs = self.nodes.len(),
            queue = self.queue.len(),
            "TechSystem loaded"
        );
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
        col::TECHNOLOGY
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
        TechSystem::handle_command(self, cmd, payload, config)
    }
}

impl shared::msg::ToFunctionClientBaseBytes for TechSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(
            func_type::TECHNOLOGY,
            func_tag::TECHNOLOGY,
            &self.to_proto(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::FunctionClientBase;
    use shared::msg::{func_tag, func_type, GameMessage, ToFunctionClientBaseBytes};

    fn config_with(rows: Vec<StaticTechLv>) -> Arc<StaticConfig> {
        let mut cfg = StaticConfig::default();
        cfg.tech.tech_levels = rows.into_iter().map(|row| (row.id, row)).collect();
        Arc::new(cfg)
    }

    fn empty_config() -> Arc<StaticConfig> {
        Arc::new(StaticConfig::default())
    }

    fn tech_row(
        id: i32,
        tech_id: i32,
        stage: i32,
        level: i32,
        up_time: i32,
        need: Option<&str>,
    ) -> StaticTechLv {
        StaticTechLv {
            id,
            tech_id,
            tech_type: 1,
            tech_name: format!("tech-{tech_id}"),
            tech_stage: stage.to_string(),
            level,
            max_lv: "3".to_string(),
            cnt: 1,
            up_time,
            up_need_resource: Some("[[2,11,100]]".to_string()),
            reputation_require: Some("0".to_string()),
            need_tech: need.map(str::to_string),
            need_building: None,
            next_id: None,
            buff_effect_id: None,
            fight: None,
            icon: None,
            cond: None,
            description: None,
        }
    }

    fn encode<M: Message>(msg: M) -> Vec<u8> {
        msg.encode_to_vec()
    }

    fn research(tech_id: i32, stage: i32, r#type: i32) -> Vec<u8> {
        encode(TechnologyResearchRq {
            technology_id: tech_id,
            stage,
            r#type,
        })
    }

    fn speed(
        tech_id: i32,
        stage: i32,
        r#type: i32,
        prop_id: Option<i32>,
        use_number: Option<i32>,
    ) -> Vec<u8> {
        encode(TechnologySpeedUpRq {
            technology_id: tech_id,
            stage,
            r#type,
            prop_id,
            use_number,
        })
    }

    fn cancel(tech_id: i32, stage: i32) -> Vec<u8> {
        encode(TechnologyCancelRq {
            technology_id: tech_id,
            stage,
        })
    }

    #[test]
    fn persistence_roundtrips_technology_state() {
        let mut system = TechSystem::new();
        system.nodes.push(TechnologyNode {
            technology_id: Some(10),
            level: Some(2),
            stage: Some(1),
        });
        system.queue.push(TechnologyResearchQueue {
            technology_id: Some(11),
            research_level: Some(1),
            complete_time: Some(123),
            research_stage: Some(1),
            ..Default::default()
        });

        let bytes = system.save_to_bin().unwrap();
        let mut loaded = TechSystem::new();
        loaded.mark_dirty();
        loaded.load_from_bin(&bytes).unwrap();

        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.queue.len(), 1);
        assert_eq!(loaded.get_tech_level_at_stage(10, 1), 2);
        assert!(!loaded.is_dirty());
    }

    #[test]
    fn function_base_bytes_emit_current_technology_state() {
        let mut system = TechSystem::new();
        system.nodes.push(TechnologyNode {
            technology_id: Some(10),
            level: Some(1),
            stage: Some(1),
        });

        let bytes = system.to_function_base_bytes();
        let base = FunctionClientBase::decode(bytes.as_slice()).unwrap();
        assert_eq!(base.r#type, Some(func_type::TECHNOLOGY));

        let decoded: TechnologyDataFunction =
            GameMessage::get_extension_from_bytes(&bytes, func_tag::TECHNOLOGY).unwrap();
        assert_eq!(decoded.node.len(), 1);
        assert_eq!(decoded.node[0].technology_id, Some(10));
        assert_eq!(decoded.node[0].level, Some(1));
    }

    #[test]
    fn research_queues_valid_target_and_rejects_duplicate_target() {
        let config = config_with(vec![tech_row(100101, 10, 1, 1, 48, None)]);
        let mut system = TechSystem::new();

        let resp = system
            .handle_command(4201, &research(10, 1, 1), &config)
            .unwrap()
            .0;
        TechnologyResearchRs::decode(resp.as_slice()).unwrap();

        assert_eq!(system.queue.len(), 1);
        assert_eq!(system.queue[0].technology_id, Some(10));
        assert_eq!(system.queue[0].research_level, Some(1));
        assert_eq!(system.queue[0].research_stage, Some(1));
        assert!(system.is_dirty());

        let err = system
            .handle_command(4201, &research(10, 1, 1), &config)
            .unwrap_err();
        assert!(err.to_string().contains("already in queue"));
    }

    #[test]
    fn research_rejects_unknown_stage_or_level_when_config_present() {
        let config = config_with(vec![tech_row(100101, 10, 1, 1, 48, None)]);
        let mut system = TechSystem::new();

        let err = system
            .handle_command(4201, &research(10, 2, 1), &config)
            .unwrap_err();
        assert!(err.to_string().contains("No tech config"));

        system.nodes.push(TechnologyNode {
            technology_id: Some(10),
            stage: Some(1),
            level: Some(1),
        });
        let err = system
            .handle_command(4201, &research(10, 1, 1), &config)
            .unwrap_err();
        assert!(err.to_string().contains("No tech config"));
    }

    #[test]
    fn research_rejects_missing_prerequisite() {
        let config = config_with(vec![
            tech_row(100101, 10, 1, 1, 48, None),
            tech_row(100201, 11, 1, 1, 60, Some("[[100101]]")),
        ]);
        let mut system = TechSystem::new();

        let err = system
            .handle_command(4201, &research(11, 1, 1), &config)
            .unwrap_err();
        assert!(err.to_string().contains("requires config id 100101"));

        system.nodes.push(TechnologyNode {
            technology_id: Some(10),
            stage: Some(1),
            level: Some(1),
        });
        system
            .handle_command(4201, &research(11, 1, 1), &config)
            .unwrap();
        assert_eq!(system.queue.len(), 1);
    }

    #[test]
    fn immediate_research_completes_and_emits_event() {
        let config = config_with(vec![tech_row(100101, 10, 1, 1, 48, None)]);
        let mut system = TechSystem::new();

        let (resp, events) = system
            .handle_command(4201, &research(10, 1, 2), &config)
            .unwrap();
        TechnologyResearchRs::decode(resp.as_slice()).unwrap();

        assert!(system.queue.is_empty());
        assert_eq!(system.get_tech_level_at_stage(10, 1), 1);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            GameEvent::TechResearchComplete {
                tech_id: 10,
                new_level: 1,
                ..
            }
        ));
    }

    #[test]
    fn speed_up_rejects_missing_queue_and_item_speed_mutates_time() {
        let mut system = TechSystem::new();
        let err = system
            .handle_command(4203, &speed(10, 1, 1, Some(1), Some(1)), &empty_config())
            .unwrap_err();
        assert!(err.to_string().contains("No active tech queue"));

        system.queue.push(TechnologyResearchQueue {
            technology_id: Some(10),
            research_level: Some(1),
            research_stage: Some(1),
            complete_time: Some(1_000),
            ..Default::default()
        });
        let (resp, events) = system
            .handle_command(4203, &speed(10, 1, 1, Some(7001), Some(2)), &empty_config())
            .unwrap();
        TechnologySpeedUpRs::decode(resp.as_slice()).unwrap();

        assert!(events.is_empty());
        assert_eq!(system.queue[0].complete_time, Some(400));
        assert!(system.is_dirty());
    }

    #[test]
    fn speed_up_complete_removes_queue_and_adds_node() {
        let mut system = TechSystem::new();
        system.queue.push(TechnologyResearchQueue {
            technology_id: Some(10),
            research_level: Some(2),
            research_stage: Some(1),
            complete_time: Some(1_000),
            ..Default::default()
        });

        let (resp, events) = system
            .handle_command(4203, &speed(10, 1, 2, None, None), &empty_config())
            .unwrap();
        TechnologySpeedUpRs::decode(resp.as_slice()).unwrap();

        assert!(system.queue.is_empty());
        assert_eq!(system.get_tech_level_at_stage(10, 1), 2);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn cancel_requires_matching_queue_and_does_not_refund_locally() {
        let mut system = TechSystem::new();
        let err = system
            .handle_command(4205, &cancel(10, 1), &empty_config())
            .unwrap_err();
        assert!(err.to_string().contains("No active tech queue"));

        system.queue.push(TechnologyResearchQueue {
            technology_id: Some(10),
            research_level: Some(1),
            research_stage: Some(1),
            complete_time: Some(1_000),
            ..Default::default()
        });
        let (resp, events) = system
            .handle_command(4205, &cancel(10, 1), &empty_config())
            .unwrap();
        TechnologyCancelRs::decode(resp.as_slice()).unwrap();

        assert!(events.is_empty());
        assert!(system.queue.is_empty());
        assert!(system.nodes.is_empty());
        assert!(system.is_dirty());
    }

    #[test]
    fn check_research_complete_keeps_tick_contract() {
        let mut system = TechSystem::new();
        system.queue.push(TechnologyResearchQueue {
            technology_id: Some(10),
            research_level: Some(1),
            research_stage: Some(1),
            complete_time: Some(10),
            ..Default::default()
        });
        system.queue.push(TechnologyResearchQueue {
            technology_id: Some(11),
            research_level: Some(1),
            research_stage: Some(1),
            complete_time: Some(20),
            ..Default::default()
        });

        let events = system.check_research_complete(99, 10);
        assert_eq!(events.len(), 1);
        assert_eq!(system.queue.len(), 1);
        assert_eq!(system.get_tech_level_at_stage(10, 1), 1);
        assert!(matches!(
            events[0],
            GameEvent::TechResearchComplete {
                role_id: 99,
                tech_id: 10,
                new_level: 1
            }
        ));
    }
}
