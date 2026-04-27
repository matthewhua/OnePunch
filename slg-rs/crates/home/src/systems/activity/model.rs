use std::collections::HashMap;
use anyhow::Result;
use super::types::{ActivityFormType, ActivityStage};

/// 个人活动数据上下文
pub struct ActivityData {
    pub activity_id: i32,
    pub open_times: i32,
}

/// 全服活动数据上下文
pub struct GlobalActivityData {
    pub activity_id: i32,
    pub stage: ActivityStage,
    pub begin_time: i64,
    pub end_time: i64,
    pub display_end_time: i64,
    pub open_times: i32,
    pub day_num: i32,
}

/// 个人活动表单 trait（对应 Java PersonalActivityForm）
pub trait PersonalForm: Send + Sync {
    fn form_type(&self) -> ActivityFormType;
    
    /// 从二进制数据加载
    fn deserialize(&mut self, data: &[u8]) -> Result<()>;
    
    /// 序列化为二进制数据（save_db=true 时包含服务端专用字段）
    fn serialize(&self, save_db: bool) -> Result<Vec<u8>>;
    
    /// 构建客户端推送的 PB 数据
    fn to_client_pb(&self, activity: &ActivityData) -> Result<Vec<u8>>;
    
    /// 每日心跳处理
    fn on_daily_tick(&mut self, _activity: &ActivityData, _day_num: i32) {}
}

/// 公共活动表单 trait（对应 Java CommonActivityForm）
pub trait CommonForm: Send + Sync {
    fn form_type(&self) -> ActivityFormType;
    fn deserialize(&mut self, data: &[u8]) -> Result<()>;
    fn serialize(&self, save_db: bool) -> Result<Vec<u8>>;
    fn on_daily_tick(&mut self, _activity: &GlobalActivityData, _day_num: i32) {}
}

/// 玩家侧的活动实例（对应 Java PersonalActivity）
pub struct PersonalActivity {
    pub activity_id: i32,
    pub open_times: i32,
    pub entrance_closed: bool,
    /// 各个玩法表单：formId -> Form
    pub forms: HashMap<i32, Box<dyn PersonalForm>>,
}

/// 持久化时保留的数据（例如跨季积分等）
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ActivityPersistent {
    // TODO: 实现具体的持久化字段
}
