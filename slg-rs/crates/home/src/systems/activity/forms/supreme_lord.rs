use std::collections::HashMap;
use anyhow::Result;
use crate::systems::activity::types::ActivityFormType;
use crate::systems::activity::model::{PersonalForm, ActivityData};

/// 最强领主（分阶段排行）活动表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SupremeLordForm {
    /// 各个阶段的积分详情：stageIdx -> score
    pub stage_scores: HashMap<i32, i64>,
    /// 总积分
    pub total_score: i64,
    /// 积分档位领取状态
    pub claimed_score_awards: HashMap<i32, Vec<i32>>, // stageIdx -> awardIds
}

impl PersonalForm for SupremeLordForm {
    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::SupremeLord
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        let decoded: Self = serde_json::from_slice(data)?;
        *self = decoded;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn to_client_pb(&self, _activity: &ActivityData) -> Result<Vec<u8>> {
        // TODO: 构建 ActivityFormSupremeLordPb
        Ok(vec![])
    }
}
