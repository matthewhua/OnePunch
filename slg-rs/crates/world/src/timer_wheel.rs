use std::collections::BTreeMap;

pub struct TimerEntry<T> {
    pub deadline_ms: i64,
    pub data: T,
}

/// 分层时间轮：O(1) 插入，O(1) 均摊 tick 推进
/// 适合大量定时事件（行军到达、采集完成、buff 过期等）
pub struct TimerWheel<T> {
    /// 细粒度轮（100ms 一格，10 格 = 1 秒）
    ticks: [Vec<T>; 10],
    /// 粗粒度轮（1 秒一格，60 格 = 1 分钟）
    seconds: [Vec<TimerEntry<T>>; 60],
    /// 溢出桶（超过 1 分钟的事件）
    overflow: BTreeMap<i64, Vec<TimerEntry<T>>>,
    /// 当前刻度 (单位: 100ms)
    current_tick: u64,
    /// 基准时间 (ms)
    base_time_ms: i64,
}

impl<T> TimerWheel<T> {
    pub fn new(base_time_ms: i64) -> Self {
        Self {
            ticks: Default::default(),
            seconds: std::array::from_fn(|_| Vec::new()),
            overflow: BTreeMap::new(),
            current_tick: 0,
            base_time_ms,
        }
    }

    /// 插入定时事件
    pub fn schedule(&mut self, deadline_ms: i64, data: T) {
        let delay_ms = deadline_ms - (self.base_time_ms + (self.current_tick * 100) as i64);
        
        if delay_ms <= 0 {
            // 已过期的直接丢到当前 tick 桶里，下次 advance 就会弹出
            self.ticks[(self.current_tick % 10) as usize].push(data);
            return;
        }

        if delay_ms < 1000 {
            // 落在当前秒内的 ticks 轮
            let target_tick = (self.current_tick + (delay_ms / 100) as u64) % 10;
            self.ticks[target_tick as usize].push(data);
        } else if delay_ms < 60_000 {
            // 落在当前分钟内的 seconds 轮
            let target_sec = ((self.current_tick / 10) + (delay_ms / 1000) as u64) % 60;
            self.seconds[target_sec as usize].push(TimerEntry { deadline_ms, data });
        } else {
            // 超过一分钟，进入溢出桶
            self.overflow.entry(deadline_ms).or_default().push(TimerEntry { deadline_ms, data });
        }
    }

    /// tick 推进，返回到期事件
    /// 假设每次调用 advance 都是推进了 100ms
    pub fn advance(&mut self) -> Vec<T> {
        self.current_tick += 1;
        let tick_idx = (self.current_tick % 10) as usize;
        
        // 1. 获取当前 tick 的所有事件
        let mut expired = std::mem::take(&mut self.ticks[tick_idx]);

        // 2. 如果当前刻度是一秒的开始，从 seconds 轮下沉数据
        if tick_idx == 0 {
            let sec_idx = ((self.current_tick / 10) % 60) as usize;
            let entries = std::mem::take(&mut self.seconds[sec_idx]);
            for entry in entries {
                // 重新 schedule 进细粒度轮
                self.schedule_entry(entry);
            }

            // 3. 如果当前刻度是一分钟的开始，从 overflow 下沉数据
            if (self.current_tick / 10) % 60 == 0 {
                let now_ms = self.base_time_ms + (self.current_tick * 100) as i64;
                let next_min_ms = now_ms + 60_000;
                
                // 找出接下来一分钟内要到期的
                let mut to_move = Vec::new();
                while let Some(entry) = self.overflow.first_entry() {
                    if *entry.key() < next_min_ms {
                        to_move.push(entry.remove());
                    } else {
                        break;
                    }
                }
                
                for entries in to_move {
                    for entry in entries {
                        self.schedule_entry(entry);
                    }
                }
            }
        }

        expired
    }

    fn schedule_entry(&mut self, entry: TimerEntry<T>) {
        self.schedule(entry.deadline_ms, entry.data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_wheel() {
        let base_time = 1000000;
        let mut wheel = TimerWheel::new(base_time);

        // 1. 测试短延时 (300ms)
        wheel.schedule(base_time + 300, 1);
        
        // Advance 100ms
        assert!(wheel.advance().is_empty()); // 100ms
        assert!(wheel.advance().is_empty()); // 200ms
        let results = wheel.advance();       // 300ms
        assert_eq!(results, vec![1]);

        // 2. 测试跨秒延时 (1500ms)
        wheel.schedule(base_time + 1500, 2);
        for _ in 0..11 { wheel.advance(); } // Up to 1400ms
        let results = wheel.advance();      // 1500ms
        assert_eq!(results, vec![2]);

        // 3. 测试跨分钟延时 (65s)
        wheel.schedule(base_time + 65000, 3);
        for _ in 0..649 { wheel.advance(); } 
        let results = wheel.advance();
        assert_eq!(results, vec![3]);
    }
}
