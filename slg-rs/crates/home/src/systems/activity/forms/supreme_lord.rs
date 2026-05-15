use std::collections::HashMap;

use anyhow::Result;
use bytes::BytesMut;
use prost::Message;

use crate::systems::activity::model::{ActivityData, PersonalForm};
use crate::systems::activity::types::ActivityFormType;

/// 最强领主（分阶段排行）活动表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SupremeLordForm {
    /// 当前阶段
    pub now_stage: i32,
    /// 各个阶段的积分详情：stageIdx -> score
    pub stage_scores: HashMap<i32, i64>,
    /// 总积分
    pub total_score: i64,
    /// 积分档位领取状态：stageIdx -> awardIds
    pub claimed_score_awards: HashMap<i32, Vec<i32>>,
    /// 各阶段排名：stageIdx -> rank
    pub stage_ranks: HashMap<i32, i32>,
    /// 总排名
    pub total_rank: i32,
}

impl SupremeLordForm {
    pub fn info_for_stage(&self, stage: i32) -> proto::slg::SupremeLordInfo {
        let claimed = self
            .claimed_score_awards
            .get(&stage)
            .cloned()
            .unwrap_or_default();
        proto::slg::SupremeLordInfo {
            stage: Some(stage),
            score: Some(self.stage_scores.get(&stage).copied().unwrap_or(0).min(i32::MAX as i64) as i32),
            claimed_key: claimed,
            rank: self.stage_ranks.get(&stage).copied(),
        }
    }

    pub fn add_score(&mut self, stage: i32, delta: i64) -> bool {
        if delta <= 0 {
            return false;
        }
        self.now_stage = stage;
        *self.stage_scores.entry(stage).or_default() += delta;
        self.total_score += delta;
        true
    }

    pub fn claim(&mut self, stage: i32, index: i32) -> Result<()> {
        let claimed = self.claimed_score_awards.entry(stage).or_default();
        if claimed.contains(&index) {
            anyhow::bail!("supreme lord award {} already claimed at stage {}", index, stage);
        }
        claimed.push(index);
        claimed.sort_unstable();
        Ok(())
    }
}

impl PersonalForm for SupremeLordForm {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::SupremeLord
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        *self = serde_json::from_slice(data)?;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn to_pb(&self, activity_id: i32, form_id: i32) -> Result<proto::slg::ActivityFormPb> {
        use proto::slg::{ActivityFormPb, ActivityFormSupremeLordPb};
        use shared::msg::GameMessage;

        let mut pb = ActivityFormPb::default();
        pb.activity_id = Some(activity_id);
        pb.form_id = Some(form_id);
        pb.form_type = Some(self.form_type() as i32);

        let mut stages: Vec<i32> = self.stage_scores.keys().copied().collect();
        stages.sort_unstable();

        let ext = ActivityFormSupremeLordPb {
            lord_info: stages.into_iter().map(|stage| self.info_for_stage(stage)).collect(),
            total_score: Some(self.total_score.min(i32::MAX as i64) as i32),
            total_rank: (self.total_rank > 0).then_some(self.total_rank),
            now_stage: Some(self.now_stage),
        };

        let mut buf = BytesMut::new();
        pb.encode(&mut buf)?;
        // ActivityFormSupremeLordPb 的 extension tag = 23。
        GameMessage::encode_extension(23, &ext, &mut buf);
        Ok(ActivityFormPb::decode(buf.as_ref())?)
    }

    fn on_daily_tick(&mut self, _activity: &ActivityData, day_num: i32) {
        self.now_stage = day_num;
    }
}
