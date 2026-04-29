use super::PlayerSystem;
use anyhow::Result;
// use proto::slg::BuildingPb; // 假设 proto 里有相关的结构

pub struct BuildingSystem {
    // 内存中的建筑数据，例如映射：建筑ID -> 建筑信息
    // schools: Vec<Building>,
}

impl BuildingSystem {
    pub fn new() -> Self {
        Self { }
    }

    /// 升级建筑业务逻辑
    pub fn upgrade_building(&mut self, _building_id: i32) -> Result<()> {
        // TODO: 实现升级逻辑
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

    fn column_name(&self) -> &'static str {
        "building_func"
    }
}
