//! 任务系统（MissionSystem）
//!
//! 对应 Java 版 MissionFunction，管理主线任务、日常任务、成长任务。
//! 数据存储在 p_data.mission_func（protobuf MissionDataFunction）。
//!
//! # 任务生命周期
//!
//! ```text
//! 创角 → create_open=1 的任务初始化到玩家身上（BaseMission, status=UNDONE）
//! 操作 → GameEvent → MissionEvent → on_mission_event → 更新 cur_schedule
//!       → cur_schedule >= s_task.schedule → status = AVAILABLE_RECEIVE
//! 领奖 → ReceiveMissionRewardRq → 发放奖励 → status = RECEIVED
//!       → config_id 加入 passedMission → 解锁 goto_states 指向的下一个任务
//! ```

use anyhow::{anyhow, Result};
use prost::Message;
use std::sync::Arc;
use tracing::{debug, info};

use super::PlayerSystem;
use proto::slg::{
    AwardPb, BaseMission, ChangeInfo, DailyMission, GainDailyLivenessRewardRq,
    GainDailyLivenessRewardRs, MissionDataFunction, ReceiveChapterMissionRewardBatchRs,
    ReceiveChapterMissionRq, ReceiveChapterMissionRs, ReceiveMissionRewardRq,
    ReceiveMissionRewardRs,
};
use shared::event::{GameEvent, MissionEvent, PlayerContext};
use shared::persistence::col;
use shared::static_config::StaticConfig;

/// 任务状态（与 proto MissionStatus 枚举对齐）
const STATUS_UNDONE: i32 = 0;
const STATUS_AVAILABLE: i32 = 1;
const STATUS_RECEIVED: i32 = 2;

/// 任务类型（与 proto MissionDefine 枚举对齐）
const MISSION_DEFINE_MAIN: i32 = 0; // 章节任务
const MISSION_DEFINE_DAILY: i32 = 2; // 每日任务
const MISSION_DEFINE_GROW_MAIN: i32 = 3; // 成长主线
const MISSION_DEFINE_GROW_SPUR: i32 = 4; // 成长支线

/// 任务系统
pub struct MissionSystem {
    dirty: bool,
    pub data: MissionDataFunction,
}

impl MissionSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            data: MissionDataFunction::default(),
        }
    }

    // ── 初始化 ────────────────────────────────────────────────────────────────

    /// 创角时初始化任务（create_open=1 的任务写入玩家数据）
    pub fn init_for_new_player(&mut self, config: &StaticConfig) {
        for (task_id, task) in &config.task.tasks {
            if task.create_open == Some(1) {
                let mission_define = task.task_type.unwrap_or(MISSION_DEFINE_GROW_MAIN);
                let mission = BaseMission {
                    mission_define: Some(mission_define),
                    config_id: Some(*task_id),
                    cur_schedule: Some(0),
                    status: Some(STATUS_UNDONE),
                    ..Default::default()
                };
                match mission_define {
                    MISSION_DEFINE_GROW_MAIN => self.data.grow_main_mission.push(mission),
                    MISSION_DEFINE_GROW_SPUR => self.data.grow_spur_mission.push(mission),
                    _ => self.data.open_mission.push(mission),
                }
            }
        }
        self.dirty = true;
        info!(
            tasks = self.data.grow_main_mission.len() + self.data.grow_spur_mission.len(),
            "MissionSystem initialized for new player"
        );
    }

    pub fn is_uninitialized(&self) -> bool {
        self.data.curr_chapter_mission.is_none()
            && self.data.open_mission.is_empty()
            && self.data.passed_mission.is_empty()
            && self.data.daily_mission.is_none()
            && self.data.grow_main_mission.is_empty()
            && self.data.grow_spur_mission.is_empty()
            && self.data.time_limit_mission.is_empty()
    }

    // ── 事件处理 ──────────────────────────────────────────────────────────────

    /// 处理游戏事件，更新任务进度
    pub fn handle_event(&mut self, event: &GameEvent, _ctx: &mut PlayerContext) {
        if let GameEvent::Mission(mission_event) = event {
            self.on_mission_event(mission_event, None);
        }
    }

    /// 带配置的事件处理（由 PlayerActor 在有 config 时调用）
    pub fn handle_event_with_config(
        &mut self,
        event: &GameEvent,
        config: &StaticConfig,
        _ctx: &mut PlayerContext,
    ) {
        if let GameEvent::Mission(mission_event) = event {
            self.on_mission_event(mission_event, Some(config));
        }
    }

    /// 核心进度更新逻辑
    ///
    /// 通过 `tasks_by_mission_type_idx` 快速找到候选任务，
    /// 过滤 `passedMission`，更新 `cur_schedule`，检查是否完成。
    fn on_mission_event(&mut self, event: &MissionEvent, config: Option<&StaticConfig>) {
        let mission_type_val = event.mission_type.as_i32();
        let delta = event.params.first().copied().unwrap_or(1);

        // 找到该 mission_type 对应的所有 task_id
        let candidate_task_ids: Vec<i32> = if let Some(cfg) = config {
            cfg.task
                .tasks_by_mission_type_idx
                .get(&mission_type_val)
                .cloned()
                .unwrap_or_default()
        } else {
            // 没有 config 时，遍历所有活跃任务（性能较差，仅作兜底）
            self.all_active_config_ids()
        };

        if candidate_task_ids.is_empty() {
            return;
        }

        // 已完成任务集合（快速查找）
        let passed: std::collections::HashSet<i32> =
            self.data.passed_mission.iter().copied().collect();

        let mut changed = false;

        // 更新各任务列表
        for list in [
            &mut self.data.open_mission,
            &mut self.data.grow_main_mission,
            &mut self.data.grow_spur_mission,
        ] {
            for mission in list.iter_mut() {
                let config_id = match mission.config_id {
                    Some(id) => id,
                    None => continue,
                };
                // 跳过已完成或已领取
                if passed.contains(&config_id) {
                    continue;
                }
                if mission.status == Some(STATUS_RECEIVED) {
                    continue;
                }
                if !candidate_task_ids.contains(&config_id) {
                    continue;
                }

                // 获取目标进度
                let target = config
                    .and_then(|c| c.task.tasks.get(&config_id))
                    .and_then(|t| t.schedule)
                    .unwrap_or(1) as i64;

                let cur = mission.cur_schedule.unwrap_or(0);
                let new_val = (cur + delta).min(target);
                mission.cur_schedule = Some(new_val);

                if new_val >= target && mission.status != Some(STATUS_AVAILABLE) {
                    mission.status = Some(STATUS_AVAILABLE);
                    debug!(
                        config_id,
                        mission_type = mission_type_val,
                        "Mission available"
                    );
                }
                changed = true;
            }
        }

        // 日常任务
        if let Some(daily) = &mut self.data.daily_mission {
            for mission in &mut daily.mission {
                let config_id = match mission.config_id {
                    Some(id) => id,
                    None => continue,
                };
                if passed.contains(&config_id) {
                    continue;
                }
                if mission.status == Some(STATUS_RECEIVED) {
                    continue;
                }
                if !candidate_task_ids.contains(&config_id) {
                    continue;
                }

                let target = config
                    .and_then(|c| c.task.tasks.get(&config_id))
                    .and_then(|t| t.schedule)
                    .unwrap_or(1) as i64;

                let cur = mission.cur_schedule.unwrap_or(0);
                let new_val = (cur + delta).min(target);
                mission.cur_schedule = Some(new_val);

                if new_val >= target && mission.status != Some(STATUS_AVAILABLE) {
                    mission.status = Some(STATUS_AVAILABLE);
                    debug!(config_id, "Daily mission available");
                }
                changed = true;
            }
        }

        if changed {
            self.dirty = true;
        }
    }

    /// 收集所有活跃任务的 config_id（无 config 时的兜底）
    fn all_active_config_ids(&self) -> Vec<i32> {
        let mut ids = Vec::new();
        for list in [
            &self.data.open_mission,
            &self.data.grow_main_mission,
            &self.data.grow_spur_mission,
        ] {
            for m in list {
                if let Some(id) = m.config_id {
                    if m.status != Some(STATUS_RECEIVED) {
                        ids.push(id);
                    }
                }
            }
        }
        if let Some(daily) = &self.data.daily_mission {
            for m in &daily.mission {
                if let Some(id) = m.config_id {
                    if m.status != Some(STATUS_RECEIVED) {
                        ids.push(id);
                    }
                }
            }
        }
        ids
    }

    // ── 奖励发放 ──────────────────────────────────────────────────────────────

    /// 解析 award_list 字符串为 AwardPb 列表
    ///
    /// 格式：`"type,id,count;type,id,count;..."`
    fn parse_award_list(award_str: &str) -> Vec<AwardPb> {
        award_str
            .split(';')
            .filter(|s| !s.is_empty())
            .filter_map(|seg| {
                let parts: Vec<&str> = seg.split(',').collect();
                if parts.len() >= 3 {
                    Some(AwardPb {
                        r#type: parts[0].parse().unwrap_or(0),
                        id: parts[1].parse().unwrap_or(0),
                        count: parts[2].parse().unwrap_or(0),
                        ..Default::default()
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    // ── 命令处理 ──────────────────────────────────────────────────────────────

    /// 领取任务奖励（ReceiveMissionRewardRq, cmd=1179）
    fn cmd_receive_mission_reward(
        &mut self,
        payload: &[u8],
        config: &StaticConfig,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = ReceiveMissionRewardRq::decode(payload)
            .map_err(|e| anyhow!("Decode ReceiveMissionRewardRq: {}", e))?;

        let config_id = rq.config_id;
        let mission_define = rq.mission_define;

        // 找到对应任务
        let mission_list = match mission_define {
            MISSION_DEFINE_DAILY => {
                // 日常任务单独处理
                return self.cmd_receive_daily_mission_reward(config_id, config);
            }
            MISSION_DEFINE_GROW_MAIN => &mut self.data.grow_main_mission,
            MISSION_DEFINE_GROW_SPUR => &mut self.data.grow_spur_mission,
            _ => &mut self.data.open_mission,
        };

        let mission = mission_list
            .iter_mut()
            .find(|m| m.config_id == Some(config_id))
            .ok_or_else(|| anyhow!("Mission {} not found", config_id))?;

        if mission.status != Some(STATUS_AVAILABLE) {
            return Err(anyhow!(
                "Mission {} not available (status={:?})",
                config_id,
                mission.status
            ));
        }

        // 标记已领取
        mission.status = Some(STATUS_RECEIVED);
        self.data.passed_mission.push(config_id);
        self.dirty = true;

        // 获取奖励
        let awards = config
            .task
            .tasks
            .get(&config_id)
            .and_then(|t| t.award_list.as_deref())
            .map(Self::parse_award_list)
            .unwrap_or_default();

        // 解锁下一个任务
        let mut next_missions = Vec::new();
        if let Some(task) = config.task.tasks.get(&config_id) {
            if let Some(next_id) = task.goto_states {
                if next_id > 0 && !self.data.passed_mission.contains(&next_id) {
                    let next_define = config
                        .task
                        .tasks
                        .get(&next_id)
                        .and_then(|t| t.task_type)
                        .unwrap_or(mission_define);
                    let next_mission = BaseMission {
                        mission_define: Some(next_define),
                        config_id: Some(next_id),
                        cur_schedule: Some(0),
                        status: Some(STATUS_UNDONE),
                        ..Default::default()
                    };
                    next_missions.push(next_mission.clone());
                    // 插入对应列表
                    match next_define {
                        MISSION_DEFINE_GROW_MAIN => self.data.grow_main_mission.push(next_mission),
                        MISSION_DEFINE_GROW_SPUR => self.data.grow_spur_mission.push(next_mission),
                        _ => self.data.open_mission.push(next_mission),
                    }
                    debug!(next_id, "Unlocked next mission");
                }
            }
        }

        let rs = ReceiveMissionRewardRs {
            info: Some(ChangeInfo {
                show_award: awards,
                ..Default::default()
            }),
            next: next_missions,
            cur: None,
            mission_define,
            daily_mission: None,
            ..Default::default()
        };

        Ok((rs.encode_to_vec(), vec![]))
    }

    /// 领取日常任务奖励
    fn cmd_receive_daily_mission_reward(
        &mut self,
        config_id: i32,
        config: &StaticConfig,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let daily = self
            .data
            .daily_mission
            .as_mut()
            .ok_or_else(|| anyhow!("No daily mission data"))?;

        let mission = daily
            .mission
            .iter_mut()
            .find(|m| m.config_id == Some(config_id))
            .ok_or_else(|| anyhow!("Daily mission {} not found", config_id))?;

        if mission.status != Some(STATUS_AVAILABLE) {
            return Err(anyhow!("Daily mission {} not available", config_id));
        }

        mission.status = Some(STATUS_RECEIVED);
        self.dirty = true;

        let awards = config
            .task
            .tasks
            .get(&config_id)
            .and_then(|t| t.award_list.as_deref())
            .map(Self::parse_award_list)
            .unwrap_or_default();

        let rs = ReceiveMissionRewardRs {
            info: Some(ChangeInfo {
                show_award: awards,
                ..Default::default()
            }),
            mission_define: MISSION_DEFINE_DAILY,
            ..Default::default()
        };

        Ok((rs.encode_to_vec(), vec![]))
    }

    /// 领取章节任务奖励（ReceiveChapterMissionRq, cmd=1181）
    fn cmd_receive_chapter_mission(
        &mut self,
        payload: &[u8],
        config: &StaticConfig,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = ReceiveChapterMissionRq::decode(payload)
            .map_err(|e| anyhow!("Decode ReceiveChapterMissionRq: {}", e))?;

        let chapter_id = rq.chapter_id;

        let chapter = self
            .data
            .curr_chapter_mission
            .as_mut()
            .filter(|c| c.config_id == chapter_id)
            .ok_or_else(|| anyhow!("Chapter {} not current", chapter_id))?;

        // 检查章节内所有任务是否都已完成
        let all_done = chapter
            .mission
            .iter()
            .all(|m| m.status == Some(STATUS_RECEIVED));
        if !all_done {
            return Err(anyhow!("Chapter {} not all missions done", chapter_id));
        }

        chapter.status = Some(STATUS_RECEIVED);
        self.dirty = true;

        // 章节奖励
        let awards = config
            .task
            .chapters
            .get(&chapter_id)
            .and_then(|c| c.reward.as_deref())
            .map(Self::parse_award_list)
            .unwrap_or_default();

        let rs = ReceiveChapterMissionRs {
            info: Some(ChangeInfo {
                show_award: awards,
                ..Default::default()
            }),
            cur: None,
            next: None,
            ..Default::default()
        };

        Ok((rs.encode_to_vec(), vec![]))
    }

    /// 领取每日活跃度奖励（GainDailyLivenessRewardRq, cmd=1183）
    fn cmd_gain_daily_liveness_reward(
        &mut self,
        payload: &[u8],
        config: &StaticConfig,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = GainDailyLivenessRewardRq::decode(payload)
            .map_err(|e| anyhow!("Decode GainDailyLivenessRewardRq: {}", e))?;

        let reward_id = rq.id;

        let daily = self
            .data
            .daily_mission
            .as_mut()
            .ok_or_else(|| anyhow!("No daily mission data"))?;

        if daily.has_received.contains(&reward_id) {
            return Err(anyhow!(
                "Daily liveness reward {} already received",
                reward_id
            ));
        }

        // TODO: 检查当前活跃度值是否达到 reward_id 对应的门槛
        daily.has_received.push(reward_id);
        self.dirty = true;

        let rs = GainDailyLivenessRewardRs {
            info: Some(ChangeInfo::default()),
            daily_huo_yue_du_data: self.data.daily_mission.clone(),
            ..Default::default()
        };

        Ok((rs.encode_to_vec(), vec![]))
    }

    /// 一键领取章节任务奖励（ReceiveChapterMissionRewardBatchRq, cmd=1189）
    fn cmd_receive_chapter_mission_batch(
        &mut self,
        _payload: &[u8],
        config: &StaticConfig,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let mut total_awards: Vec<AwardPb> = Vec::new();

        // 批量领取所有 AVAILABLE 状态的任务
        for list in [
            &mut self.data.grow_main_mission,
            &mut self.data.grow_spur_mission,
            &mut self.data.open_mission,
        ] {
            for mission in list.iter_mut() {
                if mission.status == Some(STATUS_AVAILABLE) {
                    if let Some(config_id) = mission.config_id {
                        mission.status = Some(STATUS_RECEIVED);
                        self.data.passed_mission.push(config_id);
                        if let Some(awards) = config
                            .task
                            .tasks
                            .get(&config_id)
                            .and_then(|t| t.award_list.as_deref())
                            .map(Self::parse_award_list)
                        {
                            total_awards.extend(awards);
                        }
                    }
                }
            }
        }

        self.dirty = true;

        let rs = ReceiveChapterMissionRewardBatchRs {
            info: Some(ChangeInfo {
                show_award: total_awards,
                ..Default::default()
            }),
            cur: None,
            next: None,
            ..Default::default()
        };

        Ok((rs.encode_to_vec(), vec![]))
    }

    // ── 每日重置 ──────────────────────────────────────────────────────────────

    /// 每日重置：清空日常任务进度，按当天配置重新初始化
    pub fn do_daily_reset(&mut self, config: &StaticConfig) {
        // 找到今天对应的 s_task_daily 配置（简化：取第一条）
        // TODO: 根据开服天数选择对应的 daily 配置
        if let Some(daily_conf) = config.task.dailies.first() {
            let task_ids: Vec<i32> = daily_conf
                .task_id
                .as_deref()
                .unwrap_or("")
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            let missions: Vec<BaseMission> = task_ids
                .iter()
                .map(|&tid| BaseMission {
                    mission_define: Some(MISSION_DEFINE_DAILY),
                    config_id: Some(tid),
                    cur_schedule: Some(0),
                    status: Some(STATUS_UNDONE),
                    ..Default::default()
                })
                .collect();

            self.data.daily_mission = Some(DailyMission {
                config_id: daily_conf.id,
                current_value: Some(0),
                has_received: vec![],
                mission: missions,
            });
            self.dirty = true;
            info!(daily_id = daily_conf.id, "Daily mission reset");
        }
    }
}

impl PlayerSystem for MissionSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        self.data = MissionDataFunction::decode(data)?;
        info!(
            grow_main = self.data.grow_main_mission.len(),
            grow_spur = self.data.grow_spur_mission.len(),
            "MissionSystem loaded"
        );
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.data.encode_to_vec())
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
        col::MISSION
    }

    fn on_daily_reset(&mut self) {
        // 无 config 时仅清空，PlayerActor 会在有 config 时调用 do_daily_reset
        if let Some(daily) = &mut self.data.daily_mission {
            for m in &mut daily.mission {
                m.cur_schedule = Some(0);
                m.status = Some(STATUS_UNDONE);
            }
            daily.current_value = Some(0);
            daily.has_received.clear();
        }
        self.dirty = true;
    }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        let (resp, _) = self.handle_command_with_events(cmd, payload, config)?;
        Ok(resp)
    }

    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        match cmd {
            1179 => self.cmd_receive_mission_reward(payload, config),
            1181 => self.cmd_receive_chapter_mission(payload, config),
            1183 => self.cmd_gain_daily_liveness_reward(payload, config),
            1189 => self.cmd_receive_chapter_mission_batch(payload, config),
            _ => Err(anyhow!("Unknown mission cmd: {}", cmd)),
        }
    }
}

impl shared::msg::ToFunctionClientBaseBytes for MissionSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(
            func_type::MISSION,
            func_tag::MISSION,
            &self.data,
        )
    }
}
