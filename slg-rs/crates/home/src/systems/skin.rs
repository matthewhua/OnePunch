use super::PlayerSystem;
use anyhow::Result;

pub struct SkinSystem {
    // owned_skins: Vec<i32>,
}

impl SkinSystem {
    pub fn new() -> Self {
        Self { }
    }

    pub fn use_skin(&mut self, _skin_id: i32) -> Result<()> {
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

    fn column_name(&self) -> &'static str {
        "skin_func"
    }
}
