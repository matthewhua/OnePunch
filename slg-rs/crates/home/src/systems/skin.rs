use super::PlayerSystem;
use anyhow::Result;

pub struct SkinSystem {
    dirty: bool,
    // TODO: 皮肤数据字段
}

impl SkinSystem {
    pub fn new() -> Self {
        Self { dirty: false }
    }

    pub fn use_skin(&mut self, _skin_id: i32) -> Result<()> {
        self.dirty = true;
        Ok(())
    }
}

impl PlayerSystem for SkinSystem {
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
        shared::persistence::col::SKIN
    }
}

impl shared::msg::ToFunctionClientBaseBytes for SkinSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        proto::slg::SkinDataFunction::default().to_function_base_bytes()
    }
}
