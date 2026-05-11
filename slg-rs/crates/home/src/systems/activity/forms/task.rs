use std::collections::HashMap;
use anyhow::Result;
use crate::systems::activity::types::ActivityFormType;
use crate::systems::activity::model::PersonalForm;

/// 单个活动任务的状态
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ActivityTaskInfo {
    pub task_id: i32,
    pub progress: i64,
    pub status: i32, // 0: 进行中, 1: 可领取, 2: 已领取
}

/// 任务类活动表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct TaskForm {
    pub tasks: HashMap<i32, ActivityTaskInfo>,
}

impl PersonalForm for TaskForm {
    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::Task
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
        // TODO: 填充具体的 Extension
        Ok(pb)
    }
}
