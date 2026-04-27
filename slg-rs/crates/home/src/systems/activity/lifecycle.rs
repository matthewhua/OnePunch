use std::time::{SystemTime, UNIX_EPOCH};
use super::types::ActivityStage;
use super::model::GlobalActivityData;

/// 获取当前 Unix 时间戳 (秒)
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

impl GlobalActivityData {
    /// 根据当前时间更新活动阶段
    pub fn update_stage(&mut self, now: i64) -> bool {
        let old_stage = self.stage;
        
        self.stage = if now < self.begin_time {
            ActivityStage::PreDisplay
        } else if now < self.end_time {
            ActivityStage::Open
        } else if now < self.display_end_time {
            ActivityStage::EndDisplay
        } else {
            ActivityStage::Closed
        };

        old_stage != self.stage
    }

    /// 检查并推进活动天数 (day_num)
    /// 假设每天凌晨 0 点切换，或者根据 begin_time 偏移量计算
    pub fn check_day_increment(&mut self, _now: i64) -> bool {
        // TODO: 实现更精确的跨天逻辑
        false
    }
}
