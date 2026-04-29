use super::PlayerSystem;
use anyhow::Result;

pub struct BuildingSystem {
    dirty: bool,
    // TODO: 建筑数据字段
}

impl BuildingSystem {
    pub fn new() -> Self {
        Self { dirty: false }
    }

    pub fn upgrade_building(&mut self, _building_id: i32) -> Result<()> {
        // TODO: 实现升级逻辑
        self.dirty = true;
        Ok(())
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
        shared::persistence::col::SIM  // 建筑对应 sim_func（模拟经营）
    }
}
