use std::collections::HashSet;
use anyhow::Result;
use crate::systems::activity::types::ActivityFormType;
use crate::systems::activity::model::PersonalForm;

/// 积分奖励类活动表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ScoreForm {
    /// 当前累计积分
    pub current_score: i64,
    /// 已领取的积分档位 ID
    pub claimed_ids: HashSet<i32>,
}

impl PersonalForm for ScoreForm {
    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::ScoreAward
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        let decoded: Self = serde_json::from_slice(data)?;
        *self = decoded;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn to_pb(&self, activity_id: i32, form_id: i32) -> Result<proto::slg::ActivityFormPb> {
        let mut pb = proto::slg::ActivityFormPb::default();
        pb.activity_id = Some(activity_id);
        pb.form_id = Some(form_id);
        pb.form_type = Some(self.form_type() as i32);
        Ok(pb)
    }
}
