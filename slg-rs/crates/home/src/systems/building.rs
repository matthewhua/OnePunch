use super::PlayerSystem;
use anyhow::Result;
use prost::Message;
use shared::event::GameEvent;

pub struct BuildingSystem {
    dirty: bool,
    data: proto::slg::SimDataFunction,
}

impl BuildingSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            data: proto::slg::SimDataFunction {
                landform: Some(default_landform()),
                ..Default::default()
            },
        }
    }

    /// 建筑升级完成，返回触发的游戏事件列表
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

    /// 命令分发入口（实现 PlayerSystem::handle_command）
    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<Vec<u8>> {
        match cmd {
            1501 => self.cmd_build_lv_up(payload, config),
            1503 => {
                /* BuildPosChangeRq */
                Ok(vec![])
            }
            1505 => {
                /* BuildRemoveRq */
                Ok(vec![])
            }
            1507 => {
                /* AddSurvivorRq */
                Ok(vec![])
            }
            1513 => {
                /* MapResChangeRq */
                Ok(vec![])
            }
            1515 => {
                /* MapStateChangeRq */
                Ok(vec![])
            }
            1517 => self.cmd_build_start(payload, config),
            1531 => self.cmd_build_speed(payload),
            1535 => {
                /* GainOfflineAwardRq */
                Ok(vec![])
            }
            _ => Err(anyhow::anyhow!("Unknown building cmd: {}", cmd)),
        }
    }

    fn landform_mut(&mut self) -> &mut proto::slg::LandformData {
        self.data.landform.get_or_insert_with(default_landform)
    }

    fn building_type(
        config: &shared::static_config::StaticConfig,
        build_id: i32,
        current: Option<&proto::slg::BaseBuildData>,
    ) -> i32 {
        config
            .building
            .buildings
            .get(&build_id)
            .map(|building| building.building_type)
            .or_else(|| current.map(|building| building.r#type))
            .unwrap_or_default()
    }

    fn find_build_mut(&mut self, build_id: i32) -> Option<&mut proto::slg::BaseBuildData> {
        self.landform_mut()
            .builds
            .iter_mut()
            .find(|build| build.id == build_id)
    }

    fn upsert_build(&mut self, build: proto::slg::BaseBuildData) -> proto::slg::BaseBuildData {
        let landform = self.landform_mut();
        if let Some(existing) = landform
            .builds
            .iter_mut()
            .find(|existing| existing.id == build.id)
        {
            *existing = build.clone();
        } else {
            landform.builds.push(build.clone());
        }
        if !landform.builded_build_id.contains(&build.id) {
            landform.builded_build_id.push(build.id);
        }
        self.dirty = true;
        build
    }

    fn build_response(build: proto::slg::BaseBuildData, cmd: u32) -> Result<Vec<u8>> {
        match cmd {
            1502 => Ok(proto::slg::BuildLvUpRs {
                build_data: Some(build),
            }
            .encode_to_vec()),
            1518 => Ok(proto::slg::BuildStartRs {
                build_data: Some(build),
            }
            .encode_to_vec()),
            1532 => Ok(proto::slg::BuildSpeedRs {
                build_data: Some(build),
            }
            .encode_to_vec()),
            _ => unreachable!("invalid building response cmd"),
        }
    }

    fn cmd_build_start(
        &mut self,
        payload: &[u8],
        config: &shared::static_config::StaticConfig,
    ) -> Result<Vec<u8>> {
        let rq = proto::slg::BuildStartRq::decode(payload)?;
        let existing = self
            .landform_mut()
            .builds
            .iter()
            .find(|build| build.id == rq.build_id)
            .cloned();
        let now = now_secs();
        let fast_finish = rq.is_fast_finish.unwrap_or(false);
        let mut build = existing
            .clone()
            .unwrap_or_else(|| proto::slg::BaseBuildData {
                id: rq.build_id,
                pos: rq.pos.unwrap_or_default(),
                level: 0,
                broken: Some(false),
                r#type: Self::building_type(config, rq.build_id, None),
                ..Default::default()
            });
        if let Some(pos) = rq.pos {
            build.pos = pos;
        }
        build.r#type = Self::building_type(config, rq.build_id, existing.as_ref());

        if fast_finish {
            build.level = (build.level + 1).max(1);
            build.bd_queue = None;
        } else {
            build.bd_queue = Some(proto::slg::BuildQueue {
                filled_res: Vec::new(),
                working_time: Some(0),
                finish_time: Some(now + 60),
                start_time: Some(now),
                auto_build: Some(false),
            });
        }

        let build = self.upsert_build(build);
        Self::build_response(build, 1518)
    }

    fn cmd_build_lv_up(
        &mut self,
        payload: &[u8],
        config: &shared::static_config::StaticConfig,
    ) -> Result<Vec<u8>> {
        let rq = proto::slg::BuildLvUpRq::decode(payload)?;
        let existing = self.find_build_mut(rq.build_id).cloned();
        let mut build = existing
            .clone()
            .unwrap_or_else(|| proto::slg::BaseBuildData {
                id: rq.build_id,
                pos: rq.pos,
                level: 0,
                broken: Some(false),
                r#type: Self::building_type(config, rq.build_id, None),
                ..Default::default()
            });
        build.pos = rq.pos;
        build.r#type = Self::building_type(config, rq.build_id, existing.as_ref());
        build.level = if rq.is_up.unwrap_or(false) || build.level > 0 {
            build.level + 1
        } else {
            1
        };
        build.bd_queue = None;

        let build = self.upsert_build(build);
        Self::build_response(build, 1502)
    }

    fn cmd_build_speed(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq = proto::slg::BuildSpeedRq::decode(payload)?;
        let build_id = rq
            .build_id
            .ok_or_else(|| anyhow::anyhow!("BuildSpeedRq missing build_id"))?;
        let now = now_secs();
        let build = self
            .find_build_mut(build_id)
            .ok_or_else(|| anyhow::anyhow!("building {} not found", build_id))?;
        let Some(queue) = build.bd_queue.as_mut() else {
            return Self::build_response(build.clone(), 1532);
        };

        let finish_time = queue.finish_time.unwrap_or(now);
        let speed_secs = rq.item_num.unwrap_or_default().max(0) * 60;
        let finish_now = rq.is_gold_speed == Some(1) || speed_secs == 0;
        let new_finish = if finish_now {
            now
        } else {
            (finish_time - speed_secs).max(now)
        };
        queue.finish_time = Some(new_finish);
        queue.working_time = queue.start_time.map(|start| (now - start).max(0));

        if new_finish <= now {
            build.level = (build.level + 1).max(1);
            build.bd_queue = None;
        }

        let build = build.clone();
        self.dirty = true;
        Self::build_response(build, 1532)
    }

    fn to_proto(&self) -> proto::slg::SimDataFunction {
        self.data.clone()
    }
}

impl PlayerSystem for BuildingSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        self.data = proto::slg::SimDataFunction::decode(data)?;
        self.data.landform.get_or_insert_with(default_landform);
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
        shared::persistence::col::SIM
    }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<Vec<u8>> {
        BuildingSystem::handle_command(self, cmd, payload, config)
    }

    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<(Vec<u8>, Vec<shared::event::GameEvent>)> {
        let resp = BuildingSystem::handle_command(self, cmd, payload, config)?;
        Ok((resp, vec![]))
    }
}

impl shared::msg::ToFunctionClientBaseBytes for BuildingSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        shared::msg::ToFunctionClientBaseBytes::to_function_base_bytes(&self.to_proto())
    }
}

fn default_landform() -> proto::slg::LandformData {
    proto::slg::LandformData {
        map_id: 1,
        ..Default::default()
    }
}

fn now_secs() -> i32 {
    chrono::Utc::now().timestamp() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::{
        BuildLvUpRq, BuildLvUpRs, BuildSpeedRq, BuildSpeedRs, BuildStartRq, BuildStartRs,
    };
    use shared::static_config::StaticConfig;
    use std::sync::Arc;

    #[test]
    fn load_save_roundtrip_preserves_sim_data() {
        let mut system = BuildingSystem::new();
        system.upsert_build(proto::slg::BaseBuildData {
            id: 101,
            pos: 202,
            level: 3,
            r#type: 4,
            ..Default::default()
        });

        let bytes = system.save_to_bin().unwrap();
        let mut loaded = BuildingSystem::new();
        loaded.load_from_bin(&bytes).unwrap();

        let landform = loaded.to_proto().landform.unwrap();
        assert_eq!(landform.builds.len(), 1);
        assert_eq!(landform.builds[0].id, 101);
        assert_eq!(landform.builds[0].level, 3);
    }

    #[test]
    fn build_start_level_up_and_speed_update_minimal_state() {
        let mut system = BuildingSystem::new();
        let config = Arc::new(StaticConfig::default());

        let start = BuildStartRq {
            build_id: 1001,
            pos: Some(20),
            is_full: Some(true),
            is_fast_finish: Some(false),
        };
        let start_rs = BuildStartRs::decode(
            system
                .handle_command(1517, &start.encode_to_vec(), &config)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let started = start_rs.build_data.unwrap();
        assert_eq!(started.id, 1001);
        assert_eq!(started.pos, 20);
        assert_eq!(started.level, 0);
        assert!(started.bd_queue.is_some());

        let speed = BuildSpeedRq {
            map_id: Some(1),
            build_id: Some(1001),
            is_gold_speed: Some(1),
            item_id: None,
            item_num: None,
        };
        let speed_rs = BuildSpeedRs::decode(
            system
                .handle_command(1531, &speed.encode_to_vec(), &config)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let sped = speed_rs.build_data.unwrap();
        assert_eq!(sped.level, 1);
        assert!(sped.bd_queue.is_none());

        let lv_up = BuildLvUpRq {
            build_id: 1001,
            pos: 20,
            is_up: Some(true),
        };
        let lv_up_rs = BuildLvUpRs::decode(
            system
                .handle_command(1501, &lv_up.encode_to_vec(), &config)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert_eq!(lv_up_rs.build_data.unwrap().level, 2);
    }
}
