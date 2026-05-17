//! VIP system baseline.
//!
//! Keeps per-player VIP data in `p_data.vip_func` and exposes it through
//! `GetRoleData`. Business actions such as privilege-card rewards are left for
//! follow-up tasks; this step only provides the load/save/downstream foundation.

use anyhow::Result;
use prost::Message;
use proto::slg::{LordDataFunction, VipDataFunction};
use shared::persistence::col;
use tracing::info;

use super::PlayerSystem;

#[derive(Debug, Clone)]
pub struct VipSystem {
    dirty: bool,
    data: VipDataFunction,
}

impl VipSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            data: VipDataFunction::default(),
        }
    }

    pub fn sync_from_lord(&mut self, lord: &LordDataFunction) {
        self.data.vip_level = lord.vip_level;
        self.data.exp = lord.vip_exp.map(i64::from);
        self.data.vip_open.get_or_insert(false);
    }

    pub fn sync_from_lord_row(&mut self, lord: &shared::persistence::LordRow) {
        self.data.vip_level = lord.vip_level;
        self.data.exp = lord.vip_exp.map(i64::from);
        self.data.vip_open.get_or_insert(false);
    }

    pub fn to_proto(&self) -> VipDataFunction {
        self.data.clone()
    }

    pub fn vip_level(&self) -> Option<i32> {
        self.data.vip_level
    }

    pub fn exp(&self) -> Option<i64> {
        self.data.exp
    }
}

impl Default for VipSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerSystem for VipSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        self.data = VipDataFunction::decode(data)?;
        info!(vip_level = ?self.data.vip_level, "VipSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.data.encode_to_vec())
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
        col::VIP
    }

    fn handle_command(
        &mut self,
        cmd: u32,
        _payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Unsupported vip cmd: {}", cmd))
    }
}

impl shared::msg::ToFunctionClientBaseBytes for VipSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(func_type::VIP, func_tag::VIP, &self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use proto::slg::FunctionClientBase;
    use shared::msg::{ToFunctionClientBaseBytes, func_type};

    #[test]
    fn save_load_roundtrip_preserves_vip_data() {
        let mut system = VipSystem::new();
        system.data = VipDataFunction {
            exp: Some(123),
            vip_level: Some(4),
            vip_open: Some(true),
            ..Default::default()
        };

        let bin = system.save_to_bin().unwrap();
        let mut loaded = VipSystem::new();
        loaded.load_from_bin(&bin).unwrap();

        assert_eq!(loaded.exp(), Some(123));
        assert_eq!(loaded.vip_level(), Some(4));
        assert_eq!(loaded.to_proto().vip_open, Some(true));
    }

    #[test]
    fn sync_from_lord_sets_baseline_without_dirtying() {
        let mut system = VipSystem::new();
        system.sync_from_lord(&LordDataFunction {
            vip_level: Some(3),
            vip_exp: Some(88),
            ..Default::default()
        });

        assert_eq!(system.vip_level(), Some(3));
        assert_eq!(system.exp(), Some(88));
        assert!(!system.is_dirty());
    }

    #[test]
    fn function_base_uses_vip_type() {
        let mut system = VipSystem::new();
        system.data.vip_level = Some(2);

        let base = FunctionClientBase::decode(system.to_function_base_bytes().as_slice()).unwrap();
        assert_eq!(base.r#type, Some(func_type::VIP));
    }
}
