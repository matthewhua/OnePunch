use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use prost::Message;

use super::PlayerSystem;
use shared::event::{ActivityTriggerEvent, EventHandler, GameEvent, MissionEvent, MissionType, PlayerContext};
use shared::static_config::StaticConfig;

pub mod forms;
pub mod lifecycle;
pub mod model;
pub mod settle;
pub mod types;

use forms::{score::ScoreForm, sign::SignForm, supreme_lord::SupremeLordForm, task::TaskForm};
use model::{ActivityPersistent, PersonalActivity, PersonalForm};
use types::ActivityFormType;

/// 玩家活动系统（对应 Java ActivityFunction）
pub struct ActivitySystem {
    /// 玩家当前参与的所有活动数据：activityId -> PersonalActivity
    pub activities: HashMap<i32, PersonalActivity>,
    /// 跨赛季持久化数据
    pub persistent: ActivityPersistent,
    /// 脏数据标记
    pub dirty: bool,
}

impl ActivitySystem {
    pub fn new() -> Self {
        Self {
            activities: HashMap::new(),
            persistent: ActivityPersistent::default(),
            dirty: false,
        }
    }

    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        // 活动协议命令分发入口。
        match cmd {
            8001 => self.get_activity_func_data(),
            8007 => self.activity_sign(payload),
            8009 => self.gain_task_award(payload),
            8013 => self.gain_score_award(payload, config),
            8033 => self.supreme_lord_info(payload),
            8035 => self.supreme_lord_claim_award(payload, config),
            8037 => self.supreme_lord_rank(payload),
            _ => Err(anyhow!("unknown activity cmd: {}", cmd)),
        }
    }

    fn decode_request<T: Message + Default>(cmd: u32, payload: &[u8]) -> Result<T> {
        if let Ok(msg) = shared::msg::GameMessage::decode(payload.to_vec()) {
            if msg.base.cmd == cmd as i32 {
                return shared::msg::GameMessage::get_extension_from_bytes::<T>(&msg.raw_data, cmd)
                    .map_err(|e| anyhow!("decode activity request {} extension failed: {}", cmd, e));
            }
        }

        T::decode(payload).map_err(|e| anyhow!("decode activity request {} failed: {}", cmd, e))
    }

    fn build_response<T: Message>(cmd: i32, msg: &T) -> Result<Vec<u8>> {
        shared::msg::GameMessage::build_response(cmd, msg)
    }

    fn get_activity_func_data(&mut self) -> Result<Vec<u8>> {
        // 构建全量活动数据 PB。
        let mut activity_func = proto::slg::ActivityFunction::default();
        for (activity_id, personal) in &self.activities {
            let mut data_pb = proto::slg::ActivityDataPb::default();
            data_pb.activity_id = *activity_id;
            data_pb.open_times = Some(personal.open_times);
            for (form_id, form) in &personal.forms {
                data_pb.form.push(form.to_pb(*activity_id, *form_id)?);
            }
            activity_func.activity.push(data_pb);
        }
        Self::build_response(8002, &proto::slg::GetActivityFuncDataRs {
            activity_func,
        })
    }

    fn activity_sign(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq: proto::slg::ActivitySignRq = Self::decode_request(8007, payload)?;
        let form = self.ensure_form::<SignForm>(rq.activity_id, rq.form_id, ActivityFormType::Sign)?;
        form.sign(rq.day_num, chrono::Utc::now().timestamp())?;
        self.dirty = true;
        Self::build_response(8008, &proto::slg::ActivitySignRs::default())
    }

    fn gain_task_award(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq: proto::slg::ActivityGainTaskAwardRq = Self::decode_request(8009, payload)?;
        let task_id = rq.task_id.ok_or_else(|| anyhow!("missing activity task_id"))?;
        let form = self.ensure_form::<TaskForm>(rq.activity_id, rq.form_id, ActivityFormType::Task)?;
        form.claim(task_id)?;
        self.dirty = true;
        Self::build_response(8010, &proto::slg::ActivityGainTaskAwardRs::default())
    }

    fn gain_score_award(&mut self, payload: &[u8], config: &StaticConfig) -> Result<Vec<u8>> {
        let rq: proto::slg::ActivityGainScoreAwardRq = Self::decode_request(8013, payload)?;
        let score_goal = rq.score_goal.ok_or_else(|| anyhow!("missing score_goal"))?;
        let advance = rq.advance.unwrap_or(false);
        self.validate_score_goal(config, rq.activity_id, rq.form_id, score_goal, advance)?;

        let form = self.ensure_form::<ScoreForm>(rq.activity_id, rq.form_id, ActivityFormType::ScoreAward)?;
        form.claim(score_goal, advance)?;
        self.dirty = true;
        Self::build_response(8014, &proto::slg::ActivityGainScoreAwardRs::default())
    }

    fn supreme_lord_info(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq: proto::slg::SupremeLordInfoRq = Self::decode_request(8033, payload)?;
        let activity_id = rq.activity_id.ok_or_else(|| anyhow!("missing activity_id"))?;
        let form_id = self.find_form_id(activity_id, ActivityFormType::SupremeLord).unwrap_or(0);
        let form = self.ensure_form::<SupremeLordForm>(activity_id, form_id, ActivityFormType::SupremeLord)?;
        let stage = rq.stage.unwrap_or(form.now_stage.max(1));
        Self::build_response(8034, &proto::slg::SupremeLordInfoRs {
            info: Some(form.info_for_stage(stage)),
            activity_id: Some(activity_id),
        })
    }

    fn supreme_lord_claim_award(&mut self, payload: &[u8], config: &StaticConfig) -> Result<Vec<u8>> {
        let rq: proto::slg::SupremeLordClaimAwardRq = Self::decode_request(8035, payload)?;
        let activity_id = rq.activity_id.ok_or_else(|| anyhow!("missing activity_id"))?;
        let index = rq.index.ok_or_else(|| anyhow!("missing supreme lord award index"))?;
        let form_id = self.find_form_id(activity_id, ActivityFormType::SupremeLord).unwrap_or(0);
        self.validate_supreme_lord_award(config, activity_id, form_id, index)?;

        let form = self.ensure_form::<SupremeLordForm>(activity_id, form_id, ActivityFormType::SupremeLord)?;
        let stage = form.now_stage.max(1);
        form.claim(stage, index)?;
        let lord_info = form.info_for_stage(stage);
        self.dirty = true;
        Self::build_response(8036, &proto::slg::SupremeLordClaimAwardRs {
            lord_info: Some(lord_info),
            activity_id: Some(activity_id),
        })
    }

    fn supreme_lord_rank(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq: proto::slg::SupremeLordRankRq = Self::decode_request(8037, payload)?;
        let activity_id = rq.activity_id.unwrap_or_default();
        let stage = rq.stage.unwrap_or(1);
        let rank = proto::slg::SupremeLordRankItem {
            stage: Some(stage),
            rank_item: Vec::new(),
            my_rank: None,
            my_info: None,
        };
        Self::build_response(8038, &proto::slg::SupremeLordRankRs {
            rank: Some(rank),
            activity_id: Some(activity_id),
        })
    }

    fn ensure_form<T: PersonalForm + Default + 'static>(
        &mut self,
        activity_id: i32,
        form_id: i32,
        form_type: ActivityFormType,
    ) -> Result<&mut T> {
        let activity = self.activities.entry(activity_id).or_insert_with(|| PersonalActivity {
            activity_id,
            open_times: 1,
            entrance_closed: false,
            forms: HashMap::new(),
        });
        activity.forms.entry(form_id).or_insert_with(|| Box::<T>::default());
        activity.forms.get_mut(&form_id)
            .and_then(|f| f.as_any_mut().downcast_mut::<T>())
            .ok_or_else(|| anyhow!("activity {} form {} is not {:?}", activity_id, form_id, form_type))
    }

    fn find_form_id(&self, activity_id: i32, form_type: ActivityFormType) -> Option<i32> {
        self.activities.get(&activity_id).and_then(|activity| {
            activity.forms.iter()
                .find(|(_, form)| form.form_type() == form_type)
                .map(|(form_id, _)| *form_id)
        })
    }

    fn validate_score_goal(
        &self,
        config: &StaticConfig,
        activity_id: i32,
        form_id: i32,
        score_goal: i32,
        advance: bool,
    ) -> Result<()> {
        let ok = config.activity.score_gears_by_form_idx
            .get(&form_id)
            .into_iter()
            .flatten()
            .map(|idx| &config.activity.score_gears[*idx])
            .any(|gear| {
                gear.activity_id == activity_id
                    && gear.score_goal == score_goal
                    && (!advance || gear.advance_award.as_deref().unwrap_or_default().trim().len() > 0)
            });
        if !ok {
            anyhow::bail!("score goal {} not configured for activity {} form {}", score_goal, activity_id, form_id);
        }
        Ok(())
    }

    fn validate_supreme_lord_award(
        &self,
        config: &StaticConfig,
        activity_id: i32,
        form_id: i32,
        index: i32,
    ) -> Result<()> {
        let configured = config.activity.score_gears.iter().any(|gear| {
            gear.activity_id == activity_id && gear.form_id == form_id && gear.id == index
        }) || config.activity.awards.iter().any(|award| {
            award.activity_id == Some(activity_id) && award.key_id == index
        });

        if !configured && !config.activity.score_gears.is_empty() {
            anyhow::bail!("supreme lord award {} not configured for activity {}", index, activity_id);
        }
        Ok(())
    }
}

impl PlayerSystem for ActivitySystem {
    fn load_from_bin(&mut self, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn is_dirty(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn column_name(&self) -> &'static str { shared::persistence::col::ACTIVITY }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        ActivitySystem::handle_command(self, cmd, payload, config)
    }

    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let resp = ActivitySystem::handle_command(self, cmd, payload, config)?;
        Ok((resp, vec![]))
    }
}

impl crate::systems::ToFunctionClientBase for ActivitySystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        // 构建全量活动数据 PB。
        let mut activity_func = proto::slg::ActivityFunction::default();
        for (activity_id, personal) in &self.activities {
            let mut data_pb = proto::slg::ActivityDataPb::default();
            data_pb.activity_id = *activity_id;
            data_pb.open_times = Some(personal.open_times);
            // 填充表单。
            for (form_id, form) in &personal.forms {
                match form.to_pb(*activity_id, *form_id) {
                    Ok(f_pb) => data_pb.form.push(f_pb),
                    Err(e) => tracing::error!("failed to encode activity form: {}", e),
                }
            }
            activity_func.activity.push(data_pb);
        }

        // 使用 shared::msg 中统一的 ToFunctionClientBaseBytes 实现。
        activity_func.to_function_base_bytes()
    }
}

impl EventHandler for ActivitySystem {
    fn interested_in(&self, event: &GameEvent) -> bool {
        matches!(event, GameEvent::Mission(_) | GameEvent::ActivityTrigger(_))
    }

    fn handle(&mut self, event: &GameEvent, _ctx: &mut PlayerContext) {
        match event {
            GameEvent::Mission(mission_event) => self.on_mission_event(mission_event),
            GameEvent::ActivityTrigger(trigger_event) => self.on_activity_trigger(trigger_event),
            _ => {}
        }
    }
}

impl ActivitySystem {
    pub fn on_mission_event(&mut self, event: &MissionEvent) {
        let mission_type = event.mission_type.as_i32();
        let delta = event.params.last().copied().unwrap_or(1);
        let mut changed = false;

        for activity in self.activities.values_mut() {
            for form in activity.forms.values_mut() {
                if let Some(task_form) = form.as_any_mut().downcast_mut::<TaskForm>() {
                    changed |= task_form.add_progress(mission_type, delta);
                }
                if event.mission_type == MissionType::ActivityScore {
                    if let Some(score_form) = form.as_any_mut().downcast_mut::<ScoreForm>() {
                        changed |= score_form.add_score(delta);
                    }
                    if let Some(lord_form) = form.as_any_mut().downcast_mut::<SupremeLordForm>() {
                        let stage = event.params.first().copied().unwrap_or(1) as i32;
                        changed |= lord_form.add_score(stage, delta);
                    }
                }
            }
        }

        if changed {
            self.dirty = true;
        }
    }

    pub fn on_activity_trigger(&mut self, event: &ActivityTriggerEvent) {
        let delta = event.params.last().copied().unwrap_or(1);
        let stage = event.params.first().copied().unwrap_or(1) as i32;
        let mut changed = false;
        for activity in self.activities.values_mut() {
            for form in activity.forms.values_mut() {
                if let Some(score_form) = form.as_any_mut().downcast_mut::<ScoreForm>() {
                    changed |= score_form.add_score(delta);
                }
                if let Some(lord_form) = form.as_any_mut().downcast_mut::<SupremeLordForm>() {
                    changed |= lord_form.add_score(stage, delta);
                }
            }
        }
        if changed {
            self.dirty = true;
        }
    }

    /// 外层封装调用
    pub fn handle_event(&mut self, event: &GameEvent, ctx: &mut PlayerContext) {
        if self.interested_in(event) {
            self.handle(event, ctx);
        }
    }
}

#[allow(dead_code)]
/// 辅助函数：由于 proto2 extension 不被 prost 直接支持，此函数用于手动分发 Extension 字段编解码
fn decode_form_extension(form_type: ActivityFormType, raw_bytes: &[u8]) -> Result<Box<dyn PersonalForm>> {
    let mut form: Box<dyn PersonalForm> = match form_type {
        ActivityFormType::Sign => Box::new(SignForm::default()),
        ActivityFormType::Task => Box::new(TaskForm::default()),
        ActivityFormType::ScoreAward => Box::new(ScoreForm::default()),
        ActivityFormType::SupremeLord => Box::new(SupremeLordForm::default()),
        _ => return Err(anyhow!("unsupported activity form type: {:?}", form_type)),
    };

    form.deserialize(raw_bytes)?;
    Ok(form)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::static_config::activity::StaticActivityScoreGear;

    fn config_with_score_gear(activity_id: i32, form_id: i32, goal: i32) -> Arc<StaticConfig> {
        let mut config = StaticConfig::default();
        config.activity.score_gears.push(StaticActivityScoreGear {
            id: 7,
            activity_id,
            form_id,
            cycle_id: None,
            description: None,
            activity_stage: None,
            score_goal: goal,
            score_award: Some("1,100,1".to_string()),
            normal_cond: None,
            advance_award: Some("1,101,1".to_string()),
            advance_cond: None,
            value: None,
        });
        config.activity.score_gears_by_form_idx.insert(form_id, vec![0]);
        Arc::new(config)
    }

    #[test]
    fn sign_command_marks_today_claimed() {
        let config = Arc::new(StaticConfig::default());
        let mut system = ActivitySystem::new();
        let rq = proto::slg::ActivitySignRq {
            activity_id: 1001,
            form_id: 2001,
            day_num: Some(1),
        };
        let payload = shared::msg::GameMessage::build_response(8007, &rq).unwrap();

        let resp = system.handle_command(8007, &payload, &config).unwrap();
        let msg = shared::msg::GameMessage::decode(resp).unwrap();
        assert_eq!(msg.base.cmd, 8008);

        let form = system.activities.get(&1001).unwrap().forms.get(&2001).unwrap();
        let sign = form.as_any().downcast_ref::<SignForm>().unwrap();
        assert_eq!(sign.sign_days, 1);
        assert!(sign.signed_today);
        assert!(system.dirty);
    }

    #[test]
    fn score_award_requires_reached_configured_goal() {
        let config = config_with_score_gear(1001, 2002, 50);
        let mut system = ActivitySystem::new();
        let score = system.ensure_form::<ScoreForm>(1001, 2002, ActivityFormType::ScoreAward).unwrap();
        score.add_score(60);

        let rq = proto::slg::ActivityGainScoreAwardRq {
            activity_id: 1001,
            form_id: 2002,
            score_goal: Some(50),
            advance: Some(false),
        };
        let payload = shared::msg::GameMessage::build_response(8013, &rq).unwrap();

        let resp = system.handle_command(8013, &payload, &config).unwrap();
        let msg = shared::msg::GameMessage::decode(resp).unwrap();
        assert_eq!(msg.base.cmd, 8014);

        let form = system.activities.get(&1001).unwrap().forms.get(&2002).unwrap();
        let score = form.as_any().downcast_ref::<ScoreForm>().unwrap();
        assert!(score.claimed_normal_goals.contains(&50));
    }

    #[test]
    fn supreme_lord_claim_records_stage_award() {
        let config = config_with_score_gear(1001, 2003, 100);
        let mut system = ActivitySystem::new();
        let form = system.ensure_form::<SupremeLordForm>(1001, 2003, ActivityFormType::SupremeLord).unwrap();
        form.add_score(1, 120);

        let rq = proto::slg::SupremeLordClaimAwardRq {
            index: Some(7),
            activity_id: Some(1001),
        };
        let payload = shared::msg::GameMessage::build_response(8035, &rq).unwrap();

        let resp = system.handle_command(8035, &payload, &config).unwrap();
        let msg = shared::msg::GameMessage::decode(resp).unwrap();
        assert_eq!(msg.base.cmd, 8036);

        let form = system.activities.get(&1001).unwrap().forms.get(&2003).unwrap();
        let lord = form.as_any().downcast_ref::<SupremeLordForm>().unwrap();
        assert_eq!(lord.claimed_score_awards.get(&1).cloned().unwrap_or_default(), vec![7]);
    }
}
