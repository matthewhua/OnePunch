use std::collections::HashSet;

use anyhow::Result;
use bytes::BytesMut;
use prost::Message;

use crate::systems::activity::model::{ActivityData, PersonalForm};
use crate::systems::activity::types::ActivityFormType;

/// 积分奖励类活动表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ScoreForm {
    /// 当前累计积分
    pub current_score: i64,
    /// 已领取的普通积分档位 ID
    pub claimed_normal_goals: HashSet<i32>,
    /// 已领取的高级积分档位 ID
    pub claimed_advance_goals: HashSet<i32>,
}

impl ScoreForm {
    pub fn add_score(&mut self, delta: i64) -> bool {
        if delta <= 0 {
            return false;
        }
        self.current_score += delta;
        true
    }

    pub fn claim(&mut self, score_goal: i32, advance: bool) -> Result<()> {
        if self.current_score < score_goal as i64 {
            anyhow::bail!("score goal {} not reached", score_goal);
        }

        let claimed = if advance {
            &mut self.claimed_advance_goals
        } else {
            &mut self.claimed_normal_goals
        };

        if !claimed.insert(score_goal) {
            anyhow::bail!("score goal {} already claimed", score_goal);
        }
        Ok(())
    }
}

impl PersonalForm for ScoreForm {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::ScoreAward
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        *self = serde_json::from_slice(data)?;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn to_pb(&self, activity_id: i32, form_id: i32) -> Result<proto::slg::ActivityFormPb> {
        use proto::slg::{ActivityFormPb, ActivityFormScoreAwardPb};
        use shared::msg::GameMessage;

        let mut pb = ActivityFormPb::default();
        pb.activity_id = Some(activity_id);
        pb.form_id = Some(form_id);
        pb.form_type = Some(self.form_type() as i32);

        let mut ext = ActivityFormScoreAwardPb::default();
        ext.total_score = Some(self.current_score.min(i32::MAX as i64) as i32);
        ext.normal_reward_score_goal = self.claimed_normal_goals.iter().copied().collect();
        ext.normal_reward_score_goal.sort_unstable();
        ext.advance_reward_score_goal = self.claimed_advance_goals.iter().copied().collect();
        ext.advance_reward_score_goal.sort_unstable();

        let mut buf = BytesMut::new();
        pb.encode(&mut buf)?;
        // ActivityFormScoreAwardPb 的 extension tag = 18。
        GameMessage::encode_extension(18, &ext, &mut buf);
        Ok(ActivityFormPb::decode(buf.as_ref())?)
    }

    fn on_daily_tick(&mut self, _activity: &ActivityData, _day_num: i32) {}
}
