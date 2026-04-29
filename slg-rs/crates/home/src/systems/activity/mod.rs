use std::collections::HashMap;
use anyhow::{anyhow, Result};
use super::PlayerSystem;
use shared::event::{EventHandler, GameEvent, MissionEvent, ActivityTriggerEvent, PlayerContext};

pub mod types;
pub mod model;
pub mod lifecycle;
pub mod forms;
pub mod settle;

use model::{PersonalActivity, ActivityPersistent, PersonalForm};
use forms::{sign::SignForm, task::TaskForm, score::ScoreForm, supreme_lord::SupremeLordForm};

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

    /// 活动协议命令分发入口
    pub fn handle_command(&mut self, cmd: u32, payload: &[u8]) -> Result<Vec<u8>> {
        match cmd {
            8001 => self.get_activity_func_data(),
            8007 => self.activity_sign(payload),
            // ... 后续根据玩法逐渐补充
            _ => Err(anyhow!("未知的活动命令号: {}", cmd)),
        }
    }

    fn get_activity_func_data(&mut self) -> Result<Vec<u8>> {
        // TODO: 返回全量活动数据 PB
        Ok(vec![])
    }

    fn activity_sign(&mut self, _payload: &[u8]) -> Result<Vec<u8>> {
        // TODO: 处理签到逻辑
        Ok(vec![])
    }
}

impl PlayerSystem for ActivitySystem {
    fn load_from_bin(&mut self, _data: &[u8]) -> Result<()> {
        // TODO: 使用 Prost 解析 ActivityFunction PB
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        // TODO: 序列化为二进制
        Ok(vec![])
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
        shared::persistence::col::ACTIVITY
    }
}

impl crate::systems::ToFunctionClientBase for ActivitySystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use proto::slg::{ActivityFunction, ActivityDataPb};
        use shared::msg::ToFunctionClientBaseBytes;

        // 构建全量活动数据 PB
        let mut activity_func = ActivityFunction::default();

        for (activity_id, personal) in &self.activities {
            let mut data_pb = ActivityDataPb::default();
            data_pb.activity_id = *activity_id;
            data_pb.open_times = Some(personal.open_times);

            // 填充表单
            for (form_id, form) in &personal.forms {
                match form.to_pb(*activity_id, *form_id) {
                    Ok(f_pb) => data_pb.form.push(f_pb),
                    Err(e) => tracing::error!("Failed to encode activity form: {}", e),
                }
            }
            activity_func.activity.push(data_pb);
        }

        // 使用 shared::msg 中统一的 ToFunctionClientBaseBytes 实现
        activity_func.to_function_base_bytes()
    }
}

impl EventHandler for ActivitySystem {
    fn interested_in(&self, event: &GameEvent) -> bool {
        match event {
            GameEvent::Mission(_) | GameEvent::ActivityTrigger(_) => true,
            _ => false,
        }
    }

    fn handle(&mut self, event: &GameEvent, ctx: &mut PlayerContext) {
        match event {
            GameEvent::Mission(mission_event) => self.on_mission_event(mission_event),
            GameEvent::ActivityTrigger(trigger_event) => self.on_activity_trigger(trigger_event),
            _ => {}
        }
    }
}

impl ActivitySystem {
    pub fn on_mission_event(&mut self, _event: &MissionEvent) {
        // TODO: 遍历活动，更新任务玩法（TaskForm）进度
    }

    pub fn on_activity_trigger(&mut self, _event: &ActivityTriggerEvent) {
        // TODO: 根据触发类型，激活/更新活动状态
    }

    /// 外层封装调用
    pub fn handle_event(&mut self, event: &GameEvent, ctx: &mut PlayerContext) {
        if self.interested_in(event) {
            self.handle(event, ctx);
        }
    }
}


/// 辅助函数：由于 proto2 extension 不被 prost 直接支持，此函数用于手动分发 Extension 字段编解码
#[allow(dead_code)]
fn decode_form_extension(form_type: types::ActivityFormType, raw_bytes: &[u8]) -> Result<Box<dyn model::PersonalForm>> {
    let mut form: Box<dyn PersonalForm> = match form_type {
        types::ActivityFormType::Sign => Box::new(SignForm::default()),
        types::ActivityFormType::Task => Box::new(TaskForm::default()),
        types::ActivityFormType::ScoreAward => Box::new(ScoreForm::default()),
        types::ActivityFormType::SupremeLord => Box::new(SupremeLordForm::default()),
        // ... 继续补充
        _ => return Err(anyhow!("不支持的玩法类型: {:?}", form_type)),
    };
    
    form.deserialize(raw_bytes)?;
    Ok(form)
}
