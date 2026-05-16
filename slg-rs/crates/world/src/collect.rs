use crate::arrival::{ArrivalEffect, ArrivalResolution, TargetSummary};
use proto::slg::BaseTroop;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectPhase {
    Arrived,
    Collecting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectState {
    pub troop_key: i32,
    pub target_pos: i32,
    pub march_type: Option<i32>,
    pub start_time_ms: i64,
    pub target: Option<TargetSummary>,
    pub phase: CollectPhase,
}

impl CollectState {
    pub fn arrived(troop: &BaseTroop, resolution: &ArrivalResolution) -> Option<Self> {
        if resolution.effect != ArrivalEffect::CollectStarted {
            return None;
        }

        Some(Self {
            troop_key: troop.key,
            target_pos: resolution.pos,
            march_type: troop.r#type,
            start_time_ms: troop.end_time.unwrap_or_default(),
            target: resolution.target.clone(),
            phase: CollectPhase::Arrived,
        })
    }

    pub fn start_collecting(mut self) -> Self {
        self.phase = CollectPhase::Collecting;
        self
    }
}

#[derive(Debug, Default)]
pub struct CollectStateMachine {
    by_troop: HashMap<i32, CollectState>,
}

impl CollectStateMachine {
    pub fn on_arrival(
        &mut self,
        troop: &BaseTroop,
        resolution: &ArrivalResolution,
    ) -> Option<&CollectState> {
        let state = CollectState::arrived(troop, resolution)?.start_collecting();
        let troop_key = state.troop_key;
        self.by_troop.insert(troop_key, state);
        self.by_troop.get(&troop_key)
    }

    pub fn remove(&mut self, troop_key: i32) -> Option<CollectState> {
        self.by_troop.remove(&troop_key)
    }

    pub fn get(&self, troop_key: i32) -> Option<&CollectState> {
        self.by_troop.get(&troop_key)
    }

    pub fn len(&self) -> usize {
        self.by_troop.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arrival::resolve_arrival;
    use crate::march::MARCH_TYPE_MINE_COLLECT;

    #[test]
    fn collect_arrival_enters_collecting_phase() {
        let pos = 202;
        let troop = BaseTroop {
            key: 5,
            r#type: Some(MARCH_TYPE_MINE_COLLECT),
            goal: Some(pos),
            end_time: Some(12_000),
            ..Default::default()
        };
        let resolution = resolve_arrival(&troop, None);
        let mut machine = CollectStateMachine::default();

        let state = machine.on_arrival(&troop, &resolution).unwrap();

        assert_eq!(state.troop_key, 5);
        assert_eq!(state.target_pos, pos);
        assert_eq!(state.start_time_ms, 12_000);
        assert_eq!(state.phase, CollectPhase::Collecting);
    }
}
