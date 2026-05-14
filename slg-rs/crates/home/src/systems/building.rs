use std::sync::Arc;

use anyhow::{anyhow, Result};
use prost::Message;
use proto::slg::{
    BaseBuildData, BuildLvUpRq, BuildLvUpRs, BuildPosChangeRq, BuildPosChangeRs, BuildQueue,
    BuildRemoveRq, BuildRemoveRs, BuildSpeedRq, BuildSpeedRs, BuildStartRq, BuildStartRs,
    LandformData, SimDataFunction,
};
use shared::event::GameEvent;
use shared::msg::ToFunctionClientBaseBytes;
use shared::persistence::col;
use shared::static_config::StaticConfig;

use super::PlayerSystem;

const DEFAULT_BUILD_SECONDS: i32 = 60;
const SPEED_UP_SECONDS: i32 = 300;

pub struct BuildingSystem {
    dirty: bool,
    data: SimDataFunction,
}

impl BuildingSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            data: SimDataFunction::default(),
        }
    }

    pub fn on_building_upgraded(
        &mut self,
        role_id: i64,
        build_type: i32,
        new_level: i32,
    ) -> Vec<GameEvent> {
        self.dirty = true;
        vec![GameEvent::BuildingUpgrade {
            role_id,
            build_type,
            new_level,
        }]
    }

    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        let (resp, _) = self.handle_command_inner(cmd, payload, config)?;
        Ok(resp)
    }

    fn handle_command_inner(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        match cmd {
            1501 => self.cmd_level_up(payload, config),
            1503 => self.cmd_pos_change(payload),
            1505 => self.cmd_remove(payload),
            1507 => Ok((Vec::new(), Vec::new())),
            1513 => Ok((Vec::new(), Vec::new())),
            1515 => Ok((Vec::new(), Vec::new())),
            1517 => self.cmd_start(payload, config),
            1531 => self.cmd_speed(payload),
            1535 => Ok((Vec::new(), Vec::new())),
            _ => Err(anyhow!("Unknown building cmd: {}", cmd)),
        }
    }

    fn cmd_start(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            BuildStartRq::decode(payload).map_err(|e| anyhow!("Decode BuildStartRq: {}", e))?;
        validate_build_id(rq.build_id, config)?;
        if self.find_build(rq.build_id).is_some() {
            return Err(anyhow!("Building {} already exists", rq.build_id));
        }

        let level = if rq.is_fast_finish.unwrap_or(false) {
            1
        } else {
            0
        };
        let build = BaseBuildData {
            id: rq.build_id,
            pos: rq.pos.unwrap_or(0),
            level,
            broken: Some(false),
            r#type: build_type_for(rq.build_id, config),
            bd_queue: if level == 0 {
                Some(new_build_queue())
            } else {
                None
            },
            help_id: None,
            help_num: None,
        };

        self.landform_mut().builds.push(build.clone());
        self.landform_mut().builded_build_id.push(rq.build_id);
        self.dirty = true;

        let events = if level > 0 {
            vec![GameEvent::BuildingUpgrade {
                role_id: 0,
                build_type: build.r#type,
                new_level: level,
            }]
        } else {
            Vec::new()
        };
        Ok((
            BuildStartRs {
                build_data: Some(build),
            }
            .encode_to_vec(),
            events,
        ))
    }

    fn cmd_level_up(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = BuildLvUpRq::decode(payload).map_err(|e| anyhow!("Decode BuildLvUpRq: {}", e))?;
        validate_build_id(rq.build_id, config)?;

        let build = self
            .find_build_mut(rq.build_id)
            .ok_or_else(|| anyhow!("Building {} not found", rq.build_id))?;
        if build.pos != rq.pos {
            build.pos = rq.pos;
        }

        let old_level = build.level;
        build.level = if rq.is_up.unwrap_or(true) {
            build.level + 1
        } else {
            build.level.max(1)
        };
        build.bd_queue = None;
        let upgraded = build.level > old_level;
        let event = GameEvent::BuildingUpgrade {
            role_id: 0,
            build_type: build.r#type,
            new_level: build.level,
        };
        let resp_build = build.clone();

        self.dirty = true;
        let events = if upgraded { vec![event] } else { Vec::new() };
        Ok((
            BuildLvUpRs {
                build_data: Some(resp_build),
            }
            .encode_to_vec(),
            events,
        ))
    }

    fn cmd_pos_change(&mut self, payload: &[u8]) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = BuildPosChangeRq::decode(payload)
            .map_err(|e| anyhow!("Decode BuildPosChangeRq: {}", e))?;
        let build = self
            .find_build_mut(rq.build_id)
            .ok_or_else(|| anyhow!("Building {} not found", rq.build_id))?;
        build.pos = rq.pos;
        self.dirty = true;
        Ok((BuildPosChangeRs {}.encode_to_vec(), Vec::new()))
    }

    fn cmd_remove(&mut self, payload: &[u8]) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            BuildRemoveRq::decode(payload).map_err(|e| anyhow!("Decode BuildRemoveRq: {}", e))?;
        let builds = &mut self.landform_mut().builds;
        let before = builds.len();
        builds.retain(|b| b.id != rq.build_id);
        if builds.len() == before {
            return Err(anyhow!("Building {} not found", rq.build_id));
        }
        self.dirty = true;
        Ok((BuildRemoveRs {}.encode_to_vec(), Vec::new()))
    }

    fn cmd_speed(&mut self, payload: &[u8]) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq =
            BuildSpeedRq::decode(payload).map_err(|e| anyhow!("Decode BuildSpeedRq: {}", e))?;
        let build_id = rq
            .build_id
            .ok_or_else(|| anyhow!("BuildSpeedRq missing buildId"))?;
        let build = self
            .find_build_mut(build_id)
            .ok_or_else(|| anyhow!("Building {} not found", build_id))?;

        if let Some(queue) = build.bd_queue.as_mut() {
            let finish_time = queue.finish_time.unwrap_or(DEFAULT_BUILD_SECONDS);
            queue.finish_time = Some((finish_time - SPEED_UP_SECONDS).max(0));
            if queue.finish_time == Some(0) {
                build.bd_queue = None;
            }
        }

        let resp_build = build.clone();
        self.dirty = true;
        Ok((
            BuildSpeedRs {
                build_data: Some(resp_build),
            }
            .encode_to_vec(),
            Vec::new(),
        ))
    }

    fn to_proto(&self) -> SimDataFunction {
        self.data.clone()
    }

    fn landform_mut(&mut self) -> &mut LandformData {
        self.data.landform.get_or_insert_with(|| LandformData {
            map_id: 0,
            ..Default::default()
        })
    }

    fn find_build(&self, build_id: i32) -> Option<&BaseBuildData> {
        self.data
            .landform
            .as_ref()
            .and_then(|landform| landform.builds.iter().find(|b| b.id == build_id))
    }

    fn find_build_mut(&mut self, build_id: i32) -> Option<&mut BaseBuildData> {
        self.landform_mut()
            .builds
            .iter_mut()
            .find(|b| b.id == build_id)
    }
}

impl PlayerSystem for BuildingSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        self.data = SimDataFunction::decode(data)?;
        self.dirty = false;
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
        col::SIM
    }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        BuildingSystem::handle_command(self, cmd, payload, config)
    }

    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        self.handle_command_inner(cmd, payload, config)
    }
}

impl ToFunctionClientBaseBytes for BuildingSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(func_type::SIM, func_tag::SIM, &self.to_proto())
    }
}

fn validate_build_id(build_id: i32, config: &StaticConfig) -> Result<()> {
    if build_id <= 0 {
        return Err(anyhow!("Invalid building id {}", build_id));
    }
    if !config.building.buildings.is_empty() && !config.building.buildings.contains_key(&build_id) {
        return Err(anyhow!("Unknown building config id {}", build_id));
    }
    Ok(())
}

fn build_type_for(build_id: i32, config: &StaticConfig) -> i32 {
    config
        .building
        .buildings
        .get(&build_id)
        .map(|b| b.building_type)
        .unwrap_or(build_id)
}

fn new_build_queue() -> BuildQueue {
    BuildQueue {
        working_time: Some(0),
        finish_time: Some(DEFAULT_BUILD_SECONDS),
        start_time: Some(0),
        auto_build: Some(false),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::FunctionClientBase;
    use shared::msg::{func_tag, func_type, GameMessage};

    fn config() -> Arc<StaticConfig> {
        Arc::new(StaticConfig::default())
    }

    fn encode<M: Message>(msg: M) -> Vec<u8> {
        msg.encode_to_vec()
    }

    fn start_build(
        system: &mut BuildingSystem,
        build_id: i32,
        pos: i32,
        fast: bool,
    ) -> BaseBuildData {
        let payload = encode(BuildStartRq {
            build_id,
            pos: Some(pos),
            is_full: Some(true),
            is_fast_finish: Some(fast),
        });
        let resp = system.handle_command(1517, &payload, &config()).unwrap();
        BuildStartRs::decode(resp.as_slice())
            .unwrap()
            .build_data
            .unwrap()
    }

    #[test]
    fn persistence_roundtrips_sim_data_function() {
        let mut system = BuildingSystem::new();
        start_build(&mut system, 101, 9, true);

        let bytes = system.save_to_bin().unwrap();
        let mut loaded = BuildingSystem::new();
        loaded.load_from_bin(&bytes).unwrap();

        let saved = loaded.save_to_bin().unwrap();
        let decoded = SimDataFunction::decode(saved.as_slice()).unwrap();
        let builds = decoded.landform.unwrap().builds;
        assert_eq!(builds.len(), 1);
        assert_eq!(builds[0].id, 101);
        assert_eq!(builds[0].pos, 9);
        assert_eq!(builds[0].level, 1);
        assert!(!loaded.is_dirty());
    }

    #[test]
    fn function_base_bytes_emit_current_sim_state() {
        let mut system = BuildingSystem::new();
        start_build(&mut system, 102, 12, true);

        let bytes = system.to_function_base_bytes();
        let base = FunctionClientBase::decode(bytes.as_slice()).unwrap();
        assert_eq!(base.r#type, Some(func_type::SIM));

        let decoded: SimDataFunction =
            GameMessage::get_extension_from_bytes(&bytes, func_tag::SIM).unwrap();
        let builds = decoded.landform.unwrap().builds;
        assert_eq!(builds[0].id, 102);
        assert_eq!(builds[0].pos, 12);
    }

    #[test]
    fn start_and_level_up_mutate_building_and_emit_upgrade_event() {
        let mut system = BuildingSystem::new();
        let started = start_build(&mut system, 103, 3, false);
        assert_eq!(started.level, 0);
        assert!(started.bd_queue.is_some());

        let payload = encode(BuildLvUpRq {
            build_id: 103,
            pos: 3,
            is_up: Some(true),
        });
        let (resp, events) = system
            .handle_command_with_events(1501, &payload, &config())
            .unwrap();
        let leveled = BuildLvUpRs::decode(resp.as_slice())
            .unwrap()
            .build_data
            .unwrap();
        assert_eq!(leveled.level, 1);
        assert!(leveled.bd_queue.is_none());
        assert!(matches!(
            events.as_slice(),
            [GameEvent::BuildingUpgrade {
                build_type: 103,
                new_level: 1,
                ..
            }]
        ));
    }

    #[test]
    fn position_change_and_remove_update_state() {
        let mut system = BuildingSystem::new();
        start_build(&mut system, 104, 1, true);

        let move_payload = encode(BuildPosChangeRq {
            build_id: 104,
            pos: 22,
        });
        let resp = system
            .handle_command(1503, &move_payload, &config())
            .unwrap();
        BuildPosChangeRs::decode(resp.as_slice()).unwrap();
        assert_eq!(system.find_build(104).unwrap().pos, 22);

        let remove_payload = encode(BuildRemoveRq {
            build_id: 104,
            survivors: Vec::new(),
            remove_survivor: Vec::new(),
        });
        let resp = system
            .handle_command(1505, &remove_payload, &config())
            .unwrap();
        BuildRemoveRs::decode(resp.as_slice()).unwrap();
        assert!(system.find_build(104).is_none());
    }

    #[test]
    fn speed_up_reduces_queue_and_completes_at_zero() {
        let mut system = BuildingSystem::new();
        start_build(&mut system, 105, 1, false);
        system
            .find_build_mut(105)
            .unwrap()
            .bd_queue
            .as_mut()
            .unwrap()
            .finish_time = Some(200);

        let payload = encode(BuildSpeedRq {
            map_id: Some(0),
            build_id: Some(105),
            is_gold_speed: Some(1),
            item_id: None,
            item_num: None,
        });
        let resp = system.handle_command(1531, &payload, &config()).unwrap();
        let sped = BuildSpeedRs::decode(resp.as_slice())
            .unwrap()
            .build_data
            .unwrap();
        assert!(sped.bd_queue.is_none());
    }

    #[test]
    fn invalid_command_and_missing_build_ids_error() {
        let mut system = BuildingSystem::new();
        assert!(system.handle_command(1999, &[], &config()).is_err());
        assert!(system
            .handle_command(
                1503,
                &encode(BuildPosChangeRq {
                    build_id: 999,
                    pos: 1,
                }),
                &config(),
            )
            .is_err());
        assert!(system
            .handle_command(
                1531,
                &encode(BuildSpeedRq {
                    map_id: None,
                    build_id: None,
                    is_gold_speed: None,
                    item_id: None,
                    item_num: None,
                }),
                &config(),
            )
            .is_err());
    }

    #[test]
    fn support_commands_remain_explicit_noop_smoke_responses() {
        let mut system = BuildingSystem::new();
        for cmd in [1507, 1513, 1515, 1535] {
            let resp = system.handle_command(cmd, &[], &config()).unwrap();
            assert!(resp.is_empty(), "cmd {cmd} should remain an explicit no-op");
        }

        let snapshot = system.save_to_bin().unwrap();
        let decoded = SimDataFunction::decode(snapshot.as_slice()).unwrap();
        assert!(decoded.landform.is_none());
    }
}
