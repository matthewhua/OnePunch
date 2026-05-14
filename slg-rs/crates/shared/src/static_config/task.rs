//! 任务系统静态配置
//!
//! 对应数据库表：
//! - `s_task`：任务定义
//! - `s_task_chapter`：章节任务
//! - `s_task_daily`：日常任务
//! - `s_task_daily_award`：日常任务奖励

use sqlx::FromRow;
use std::collections::HashMap;

/// 任务定义（s_task）
#[derive(Debug, Clone, FromRow)]
pub struct StaticTask {
    pub id: i32,
    #[sqlx(rename = "type")]
    pub task_type: Option<i32>,
    #[sqlx(rename = "triggerId")]
    pub trigger_id: Option<i32>,
    pub create_open: Option<i8>,
    pub mission_type: Option<i32>,
    pub params: Option<String>,
    pub schedule: Option<i32>,
    pub award_list: Option<String>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    #[sqlx(rename = "gotoStates")]
    pub goto_states: Option<i32>,
    #[sqlx(rename = "txtId")]
    pub txt_id: Option<i32>,
}

/// 章节任务（s_task_chapter）
#[derive(Debug, Clone, FromRow)]
pub struct StaticTaskChapter {
    pub id: i32,
    pub sort_id: Option<i32>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub reward: Option<String>,
    pub task: Option<String>,
    pub banner: Option<String>,
}

/// 日常任务（s_task_daily）
#[derive(Debug, Clone, FromRow)]
pub struct StaticTaskDaily {
    pub id: i32,
    pub rank: Option<i32>,
    #[sqlx(rename = "openDays")]
    pub open_days: Option<String>,
    #[sqlx(rename = "taskId")]
    pub task_id: Option<String>,
}

/// 日常活跃度奖励（s_task_daily_award）
#[derive(Debug, Clone, FromRow)]
pub struct StaticTaskDailyAward {
    pub id: i32,
    #[sqlx(rename = "openDays")]
    pub open_days: Option<String>,
    pub liveness: Option<i32>,
    pub award: Option<String>,
    #[sqlx(rename = "warToken")]
    pub war_token: Option<String>,
}

/// 任务系统完整配置
#[derive(Debug, Clone, Default)]
pub struct TaskConfig {
    /// id → StaticTask
    pub tasks: HashMap<i32, StaticTask>,
    /// id → StaticTaskChapter
    pub chapters: HashMap<i32, StaticTaskChapter>,
    /// id → StaticTaskDaily
    pub dailies: Vec<StaticTaskDaily>,
    /// id → StaticTaskDailyAward
    pub daily_awards: HashMap<i32, StaticTaskDailyAward>,
    // ── 二级索引 ──
    /// mission_type → Vec<task_id>
    pub tasks_by_mission_type_idx: HashMap<i32, Vec<i32>>,
}

impl TaskConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (task_rows, chapter_rows, daily_rows, daily_award_rows) = tokio::try_join!(
            sqlx::query_as::<_, StaticTask>("SELECT * FROM s_task").fetch_all(pool),
            sqlx::query_as::<_, StaticTaskChapter>("SELECT * FROM s_task_chapter").fetch_all(pool),
            sqlx::query_as::<_, StaticTaskDaily>("SELECT * FROM s_task_daily").fetch_all(pool),
            sqlx::query_as::<_, StaticTaskDailyAward>("SELECT * FROM s_task_daily_award")
                .fetch_all(pool),
        )?;

        let tasks: HashMap<i32, StaticTask> = task_rows.into_iter().map(|r| (r.id, r)).collect();
        let chapters: HashMap<i32, StaticTaskChapter> =
            chapter_rows.into_iter().map(|r| (r.id, r)).collect();
        let daily_awards: HashMap<i32, StaticTaskDailyAward> =
            daily_award_rows.into_iter().map(|r| (r.id, r)).collect();

        let mut cfg = Self {
            tasks,
            chapters,
            dailies: daily_rows,
            daily_awards,
            ..Default::default()
        };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        self.dailies
            .sort_by_key(|d| (d.rank.unwrap_or(i32::MAX), d.id));
        for (tid, t) in &self.tasks {
            if let Some(mt) = t.mission_type {
                self.tasks_by_mission_type_idx
                    .entry(mt)
                    .or_default()
                    .push(*tid);
            }
        }
        for ids in self.tasks_by_mission_type_idx.values_mut() {
            ids.sort_unstable();
        }
    }
}
