use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct StaticActivityPlan {
    pub activity_id: i32,
    pub open_duration: i64,
    pub display_duration: i64,
    pub form_ids: Vec<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct ActivityConfig {
    pub plans: HashMap<i32, StaticActivityPlan>,
}

impl ActivityConfig {
    pub async fn load(_pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        // TODO: 使用 sqlx::query_as 真实加载 s_activity_plan 表。当前返回空壳：
        Ok(Self::default())
    }
}
