use std::collections::HashMap;

use anyhow::Result;
use bytes::BytesMut;
use prost::Message;

use crate::systems::activity::model::{ActivityData, PersonalForm};
use crate::systems::activity::types::ActivityFormType;

/// 任务进行中。
pub const STATUS_UNDONE: i32 = 0;
/// 任务已完成，可领取。
pub const STATUS_AVAILABLE: i32 = 1;
/// 任务奖励已领取。
pub const STATUS_RECEIVED: i32 = 2;

/// 单个活动任务的状态
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ActivityTaskInfo {
    pub task_id: i32,
    pub day_num: i32,
    pub progress: i64,
    pub target: i64,
    pub mission_type: i32,
    /// 任务状态，取值见 STATUS_* 常量。
    pub status: i32,
}

/// 任务类活动表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct TaskForm {
    pub tasks: HashMap<i32, ActivityTaskInfo>,
}

impl TaskForm {
    pub fn ensure_task(&mut self, task_id: i32, day_num: i32, mission_type: i32, target: i64) {
        self.tasks.entry(task_id).or_insert(ActivityTaskInfo {
            task_id,
            day_num,
            progress: 0,
            target,
            mission_type,
            status: STATUS_UNDONE,
        });
    }

    pub fn add_progress(&mut self, mission_type: i32, delta: i64) -> bool {
        let mut changed = false;
        for task in self.tasks.values_mut() {
            if task.mission_type != mission_type || task.status == STATUS_RECEIVED {
                continue;
            }
            task.progress = (task.progress + delta).min(task.target.max(1));
            if task.progress >= task.target.max(1) {
                task.status = STATUS_AVAILABLE;
            }
            changed = true;
        }
        changed
    }

    pub fn claim(&mut self, task_id: i32) -> Result<()> {
        let task = self
            .tasks
            .get_mut(&task_id)
            .ok_or_else(|| anyhow::anyhow!("activity task {} not found", task_id))?;
        if task.status != STATUS_AVAILABLE {
            anyhow::bail!("activity task {} is not claimable", task_id);
        }
        task.status = STATUS_RECEIVED;
        Ok(())
    }
}

impl PersonalForm for TaskForm {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::Task
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        *self = serde_json::from_slice(data)?;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn to_pb(&self, activity_id: i32, form_id: i32) -> Result<proto::slg::ActivityFormPb> {
        use proto::slg::{ActivityFormPb, ActivityFormTaskPb, BaseMission};
        use shared::msg::GameMessage;

        let mut pb = ActivityFormPb::default();
        pb.activity_id = Some(activity_id);
        pb.form_id = Some(form_id);
        pb.form_type = Some(self.form_type() as i32);

        let mut ext = ActivityFormTaskPb::default();
        let mut tasks: Vec<_> = self.tasks.values().collect();
        tasks.sort_by_key(|t| (t.day_num, t.task_id));
        for task in tasks {
            ext.task.push(BaseMission {
                mission_define: Some(task.day_num),
                config_id: Some(task.task_id),
                cur_schedule: Some(task.progress),
                status: Some(task.status),
            });
        }

        let mut buf = BytesMut::new();
        pb.encode(&mut buf)?;
        // ActivityFormTaskPb 的 extension tag = 15。
        GameMessage::encode_extension(15, &ext, &mut buf);
        Ok(ActivityFormPb::decode(buf.as_ref())?)
    }

    fn on_daily_tick(&mut self, _activity: &ActivityData, day_num: i32) {
        for task in self.tasks.values_mut() {
            if task.day_num == day_num && task.status != STATUS_RECEIVED {
                task.progress = 0;
                task.status = STATUS_UNDONE;
            }
        }
    }
}
