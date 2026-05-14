use super::PlayerSystem;
use anyhow::Result;
use shared::event::GameEvent;

pub struct BuildingSystem {
    dirty: bool,
    // TODO: 建筑数据字段（对应 SimDataFunction protobuf）
}

impl BuildingSystem {
    pub fn new() -> Self {
        Self { dirty: false }
    }

    /// 建筑升级完成，返回触发的游戏事件列表
    pub fn on_building_upgraded(
        &mut self,
        role_id: i64,
        build_type: i32,
        new_level: i32,
    ) -> Vec<GameEvent> {
        self.dirty = true;
        vec![GameEvent::BuildingUpgrade { role_id, build_type, new_level }]
    }

    /// 命令分发入口（实现 PlayerSystem::handle_command）
    pub fn handle_command(
        &mut self,
        cmd: u32,
        _payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> anyhow::Result<Vec<u8>> {
        match cmd {
            1501 => { /* BuildLvUpRq */ Ok(vec![]) }
            1503 => { /* BuildPosChangeRq */ Ok(vec![]) }
            1505 => { /* BuildRemoveRq */ Ok(vec![]) }
            1507 => { /* AddSurvivorRq */ Ok(vec![]) }
            1513 => { /* MapResChangeRq */ Ok(vec![]) }
            1515 => { /* MapStateChangeRq */ Ok(vec![]) }
            1517 => { /* BuildStartRq */ Ok(vec![]) }
            1531 => { /* BuildSpeedRq */ Ok(vec![]) }
            1535 => { /* GainOfflineAwardRq */ Ok(vec![]) }
            _ => Err(anyhow::anyhow!("Unknown building cmd: {}", cmd)),
        }
    }
}

impl PlayerSystem for BuildingSystem {
    fn load_from_bin(&mut self, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn is_dirty(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }

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
        proto::slg::SimDataFunction::default().to_function_base_bytes()
    }
}
