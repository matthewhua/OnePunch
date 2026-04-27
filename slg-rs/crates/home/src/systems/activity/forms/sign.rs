use anyhow::Result;
use crate::systems::activity::types::ActivityFormType;
use crate::systems::activity::model::{PersonalForm, ActivityData};

/// 签到玩法表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SignForm {
    /// 累计签到天数
    pub sign_days: i32,
    /// 今日是否已签到
    pub signed_today: bool,
    /// 上次签到时间戳
    pub last_sign_time: i64,
}

impl PersonalForm for SignForm {
    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::Sign
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        // TODO: 从 PB 反序列化
        let decoded: Self = serde_json::from_slice(data)?;
        *self = decoded;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        // TODO: 序列化为 PB
        Ok(serde_json::to_vec(self)?)
    }

    fn to_client_pb(&self, _activity: &ActivityData) -> Result<Vec<u8>> {
        // TODO: 构建 ActivityFormSignPb
        Ok(vec![])
    }

    fn on_daily_tick(&mut self, _activity: &ActivityData, _day_num: i32) {
        // 跨天重置今日签到状态
        self.signed_today = false;
    }
}
