//! 活动系统静态配置
//!
//! 对应数据库表：
//! - `s_activity_plan`：活动计划（开启条件、时间、循环）
//! - `s_activity_form_plan`：玩法计划（formId → formType）
//! - `s_activity_form_define`：玩法类型定义
//! - `s_activity_cycle`：活动周期/赛季
//! - `s_activity_task`：活动任务
//! - `s_activity_form_sign`：签到配置
//! - `s_activity_form_score`：积分配置
//! - `s_activity_score_gear`：积分档位
//! - `s_activity_rank`：排行榜配置
//! - `s_activity_award`：活动奖励

use std::collections::HashMap;
use sqlx::FromRow;

// ─── 数据库行结构 ─────────────────────────────────────────────────────────────

/// 活动计划（s_activity_plan）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityPlan {
    pub activity_id: i32,
    pub form_id_list: String,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub trigger_cond: Option<String>,
    pub pre_cond: Option<String>,
    pub server_open_day: Option<i32>,
    pub open_duration: Option<i32>,
    pub first_begin_time: Option<chrono::NaiveDateTime>,
    pub end_time: Option<chrono::NaiveDateTime>,
    #[sqlx(rename = "loopCycle")]
    pub loop_cycle: Option<String>,
    #[sqlx(rename = "loopCnt")]
    pub loop_cnt: Option<i32>,
    pub pre_display_duration: Option<i32>,
    pub end_display_duration: Option<i32>,
    pub server_open_city: Option<String>,
    pub server_open_id: Option<String>,
    pub end_time_fix: Option<bool>,
    pub reward_all_close: Option<bool>,
}

/// 活动玩法计划（s_activity_form_plan）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityFormPlan {
    pub form_id: i32,
    pub form_type: i32,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub form_open_time: Option<i32>,
    pub form_end_time: Option<i32>,
    pub pre_operate: Option<bool>,
    pub end_operate: Option<bool>,
    pub end_mail_id: Option<i32>,
    pub end_auto_gain: Option<bool>,
    pub end_recycle: Option<String>,
    pub end_convert: Option<String>,
    pub end_recycle_mail_id: Option<i32>,
    pub daily_reset: Option<bool>,
    pub daily_auto_gain: Option<bool>,
}

/// 活动玩法类型定义（s_activity_form_define）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityFormDefine {
    pub form_type: i32,
    #[sqlx(rename = "desc")]
    pub description: String,
    pub bind_table: Option<String>,
}

/// 活动周期/赛季（s_activity_cycle）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityCycle {
    pub id: i32,
    pub cycle_id: Option<i32>,
    pub dsc: Option<String>,
    pub activity_id: i32,
    #[sqlx(rename = "group")]
    pub group_str: Option<String>,
    #[sqlx(rename = "strongestLord_hero")]
    pub strongest_lord_hero: Option<String>,
}

/// 活动任务（s_activity_task）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityTask {
    pub id: i32,
    pub activity_id: Option<i32>,
    pub form_id: i32,
    pub day_num: Option<i32>,
    pub create_open: Option<i8>,
    pub sort_order: Option<i32>,
    pub mission_type: Option<i32>,
    pub params: Option<String>,
    pub schedule: Option<i32>,
    pub double_award: Option<String>,
    pub award_list: Option<String>,
    #[sqlx(rename = "pointReward")]
    pub point_reward: Option<String>,
    pub diamond: Option<i32>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub note: Option<String>,
    #[sqlx(rename = "txtId")]
    pub txt_id: Option<i32>,
    pub update_timing: Option<bool>,
    pub function_id: Option<i32>,
    pub board_icon: Option<String>,
}

/// 活动签到配置（s_activity_form_sign）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityFormSign {
    pub form_id: i32,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub note: Option<String>,
    pub day_num: Option<String>,
    pub award_list: Option<String>,
    pub allow_interrupt: Option<bool>,
    pub pay_id: Option<i32>,
}

/// 活动积分配置（s_activity_form_score）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityFormScore {
    pub form_id: i32,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    pub note: Option<String>,
    pub score_award_type: String,
    pub normal_cond: Option<String>,
    pub score_gain_type: Option<i32>,
    pub add_init_num: Option<i8>,
    pub end_clear: Option<i8>,
    pub end_convert_award: Option<String>,
    pub convert_ratio: Option<i32>,
    pub reach_auto_reward: Option<bool>,
}

/// 活动积分档位（s_activity_score_gear）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityScoreGear {
    pub id: i32,
    pub activity_id: i32,
    pub form_id: i32,
    pub cycle_id: Option<i32>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    #[sqlx(rename = "activityStage")]
    pub activity_stage: Option<i32>,
    pub score_goal: i32,
    pub score_award: Option<String>,
    pub normal_cond: Option<String>,
    pub advance_award: Option<String>,
    pub advance_cond: Option<String>,
    pub value: Option<i32>,
}

/// 活动排行榜配置（s_activity_rank）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityRank {
    pub id: i32,
    pub activity_id: i32,
    pub form_id: i32,
    pub cycle_id: Option<i32>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    #[sqlx(rename = "activityStage")]
    pub activity_stage: Option<i32>,
    pub rank: String,
    pub reward: Option<String>,
}

/// 活动奖励（s_activity_award）
#[derive(Debug, Clone, FromRow)]
pub struct StaticActivityAward {
    #[sqlx(rename = "keyId")]
    pub key_id: i32,
    #[sqlx(rename = "activityId")]
    pub activity_id: Option<i32>,
    #[sqlx(rename = "type")]
    pub award_type: Option<i32>,
    pub ranking: Option<String>,
    pub points: Option<String>,
    #[sqlx(rename = "awardList")]
    pub award_list: Option<String>,
    #[sqlx(rename = "desc")]
    pub description: Option<String>,
    #[sqlx(rename = "gotoUi")]
    pub goto_ui: Option<String>,
    #[sqlx(rename = "openDays")]
    pub open_days: Option<String>,
}

// ─── 聚合配置 ─────────────────────────────────────────────────────────────────

/// 活动系统完整配置
#[derive(Debug, Clone, Default)]
pub struct ActivityConfig {
    /// activity_id → StaticActivityPlan
    pub plans: HashMap<i32, StaticActivityPlan>,
    /// form_id → StaticActivityFormPlan
    pub form_plans: HashMap<i32, StaticActivityFormPlan>,
    /// form_type → StaticActivityFormDefine
    pub form_defines: HashMap<i32, StaticActivityFormDefine>,
    /// id → StaticActivityCycle
    pub cycles: Vec<StaticActivityCycle>,
    /// id → StaticActivityTask
    pub tasks: Vec<StaticActivityTask>,
    /// form_id → StaticActivityFormSign
    pub form_signs: HashMap<i32, StaticActivityFormSign>,
    /// form_id → StaticActivityFormScore
    pub form_scores: HashMap<i32, StaticActivityFormScore>,
    /// 积分档位列表
    pub score_gears: Vec<StaticActivityScoreGear>,
    /// 排行榜配置列表
    pub ranks: Vec<StaticActivityRank>,
    /// 活动奖励列表
    pub awards: Vec<StaticActivityAward>,

    // ── 二级索引 ──
    /// activity_id → Vec<form_id>（从 plan.form_id_list 解析）
    pub activity_forms_idx: HashMap<i32, Vec<i32>>,
    /// form_type → Vec<form_id>
    pub form_type_idx: HashMap<i32, Vec<i32>>,
    /// form_id → Vec<StaticActivityTask>（按 form_id 分组的任务）
    pub tasks_by_form_idx: HashMap<i32, Vec<usize>>,
    /// form_id → Vec<StaticActivityScoreGear>（按 form_id 分组的积分档位）
    pub score_gears_by_form_idx: HashMap<i32, Vec<usize>>,
}

impl ActivityConfig {
    pub async fn load(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let (plans_rows, form_plans_rows, form_defines_rows, cycles, tasks, form_signs_rows,
             form_scores_rows, score_gears, ranks, awards) = tokio::try_join!(
            sqlx::query_as::<_, StaticActivityPlan>("SELECT * FROM s_activity_plan").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityFormPlan>("SELECT * FROM s_activity_form_plan").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityFormDefine>("SELECT * FROM s_activity_form_define").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityCycle>("SELECT * FROM s_activity_cycle").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityTask>("SELECT * FROM s_activity_task").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityFormSign>("SELECT * FROM s_activity_form_sign").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityFormScore>("SELECT * FROM s_activity_form_score").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityScoreGear>("SELECT * FROM s_activity_score_gear").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityRank>("SELECT * FROM s_activity_rank").fetch_all(pool),
            sqlx::query_as::<_, StaticActivityAward>("SELECT * FROM s_activity_award").fetch_all(pool),
        )?;

        let plans: HashMap<i32, StaticActivityPlan> = plans_rows
            .into_iter().map(|r| (r.activity_id, r)).collect();
        let form_plans: HashMap<i32, StaticActivityFormPlan> = form_plans_rows
            .into_iter().map(|r| (r.form_id, r)).collect();
        let form_defines: HashMap<i32, StaticActivityFormDefine> = form_defines_rows
            .into_iter().map(|r| (r.form_type, r)).collect();
        let form_signs: HashMap<i32, StaticActivityFormSign> = form_signs_rows
            .into_iter().map(|r| (r.form_id, r)).collect();
        let form_scores: HashMap<i32, StaticActivityFormScore> = form_scores_rows
            .into_iter().map(|r| (r.form_id, r)).collect();

        let mut cfg = Self {
            plans, form_plans, form_defines, cycles, tasks,
            form_signs, form_scores, score_gears, ranks, awards,
            ..Default::default()
        };
        cfg.build_indexes();
        Ok(cfg)
    }

    fn build_indexes(&mut self) {
        // activity_id → Vec<form_id>
        for (aid, plan) in &self.plans {
            let form_ids: Vec<i32> = plan.form_id_list
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            self.activity_forms_idx.insert(*aid, form_ids);
        }

        // form_type → Vec<form_id>
        for (fid, fp) in &self.form_plans {
            self.form_type_idx.entry(fp.form_type).or_default().push(*fid);
        }

        // tasks_by_form_idx
        for (i, task) in self.tasks.iter().enumerate() {
            self.tasks_by_form_idx.entry(task.form_id).or_default().push(i);
        }

        // score_gears_by_form_idx
        for (i, gear) in self.score_gears.iter().enumerate() {
            self.score_gears_by_form_idx.entry(gear.form_id).or_default().push(i);
        }
    }
}
