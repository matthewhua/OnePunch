use crate::arrival::{ArrivalEffect, ArrivalResolution, TargetSummary};
use proto::slg::{AwardPb, BaseTroop};
use std::collections::HashMap;

pub const DEFAULT_COLLECT_DURATION_MS: i64 = 500;
pub const DEFAULT_COLLECT_RETURN_DURATION_MS: i64 = 100;
pub const DEFAULT_COLLECT_RESOURCE_AMOUNT: i64 = 100;
pub const AWARD_TYPE_LORD_RESOURCE: i32 = 1;
pub const RESOURCE_ID_MEAT: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectPhase {
    Arrived,
    Collecting,
    Returning,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectState {
    pub troop_key: i32,
    pub origin_pos: i32,
    pub target_pos: i32,
    pub march_type: Option<i32>,
    pub formation_id: Option<i32>,
    pub start_time_ms: i64,
    pub collect_end_time_ms: i64,
    pub collected_amount: i64,
    pub resource_type: i32,
    pub camp: Option<i32>,
    pub target: Option<TargetSummary>,
    pub phase: CollectPhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectCompletion {
    pub troop_key: i32,
    pub origin_pos: i32,
    pub target_pos: i32,
    pub march_type: Option<i32>,
    pub formation_id: Option<i32>,
    pub collect_start_time_ms: i64,
    pub collect_end_time_ms: i64,
    pub collected_amount: i64,
    pub resource_type: i32,
    pub camp: Option<i32>,
}

impl CollectState {
    pub fn arrived(
        troop: &BaseTroop,
        resolution: &ArrivalResolution,
        formation_id: Option<i32>,
    ) -> Option<Self> {
        if resolution.effect != ArrivalEffect::CollectStarted {
            return None;
        }

        let start_time_ms = troop.end_time.unwrap_or_default();
        Some(Self {
            troop_key: troop.key,
            origin_pos: troop.origin.unwrap_or_default(),
            target_pos: resolution.pos,
            march_type: troop.r#type,
            formation_id,
            start_time_ms,
            collect_end_time_ms: start_time_ms + DEFAULT_COLLECT_DURATION_MS,
            collected_amount: DEFAULT_COLLECT_RESOURCE_AMOUNT,
            resource_type: RESOURCE_ID_MEAT,
            camp: troop.camp,
            target: resolution.target.clone(),
            phase: CollectPhase::Arrived,
        })
    }

    pub fn start_collecting(mut self) -> Self {
        self.phase = CollectPhase::Collecting;
        self
    }

    pub fn complete(&mut self, now_ms: i64) -> Option<CollectCompletion> {
        if self.phase != CollectPhase::Collecting || now_ms < self.collect_end_time_ms {
            return None;
        }

        self.phase = CollectPhase::Returning;
        Some(CollectCompletion {
            troop_key: self.troop_key,
            origin_pos: self.origin_pos,
            target_pos: self.target_pos,
            march_type: self.march_type,
            formation_id: self.formation_id,
            collect_start_time_ms: self.start_time_ms,
            collect_end_time_ms: self.collect_end_time_ms,
            collected_amount: self.collected_amount,
            resource_type: self.resource_type,
            camp: self.camp,
        })
    }

    pub fn award(&self) -> AwardPb {
        AwardPb {
            r#type: AWARD_TYPE_LORD_RESOURCE,
            id: self.resource_type,
            count: self.collected_amount,
            safe: Some(true),
            ..Default::default()
        }
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
        formation_id: Option<i32>,
    ) -> Option<&CollectState> {
        let state = CollectState::arrived(troop, resolution, formation_id)?.start_collecting();
        let troop_key = state.troop_key;
        self.by_troop.insert(troop_key, state);
        self.by_troop.get(&troop_key)
    }

    pub fn insert(&mut self, state: CollectState) {
        self.by_troop.insert(state.troop_key, state);
    }

    pub fn complete(&mut self, troop_key: i32, now_ms: i64) -> Option<CollectCompletion> {
        self.by_troop
            .get_mut(&troop_key)
            .and_then(|state| state.complete(now_ms))
    }

    pub fn finish_return(&mut self, troop_key: i32) -> Option<CollectState> {
        let mut state = self.by_troop.remove(&troop_key)?;
        if state.phase != CollectPhase::Returning {
            self.by_troop.insert(troop_key, state);
            return None;
        }
        state.phase = CollectPhase::Completed;
        Some(state)
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

        let state = machine.on_arrival(&troop, &resolution, Some(7)).unwrap();

        assert_eq!(state.troop_key, 5);
        assert_eq!(state.origin_pos, 0);
        assert_eq!(state.target_pos, pos);
        assert_eq!(state.formation_id, Some(7));
        assert_eq!(state.start_time_ms, 12_000);
        assert_eq!(
            state.collect_end_time_ms,
            12_000 + DEFAULT_COLLECT_DURATION_MS
        );
        assert_eq!(state.collected_amount, DEFAULT_COLLECT_RESOURCE_AMOUNT);
        assert_eq!(state.resource_type, RESOURCE_ID_MEAT);
        assert_eq!(state.phase, CollectPhase::Collecting);
    }

    #[test]
    fn collect_completion_moves_state_to_returning_and_builds_award() {
        let pos = 202;
        let troop = BaseTroop {
            key: 5,
            r#type: Some(MARCH_TYPE_MINE_COLLECT),
            origin: Some(11),
            goal: Some(pos),
            end_time: Some(12_000),
            ..Default::default()
        };
        let resolution = resolve_arrival(&troop, None);
        let mut machine = CollectStateMachine::default();
        machine.on_arrival(&troop, &resolution, Some(7)).unwrap();

        assert!(machine
            .complete(5, 12_000 + DEFAULT_COLLECT_DURATION_MS - 1)
            .is_none());
        let completion = machine
            .complete(5, 12_000 + DEFAULT_COLLECT_DURATION_MS)
            .unwrap();

        assert_eq!(completion.troop_key, 5);
        assert_eq!(completion.origin_pos, 11);
        assert_eq!(completion.target_pos, pos);
        assert_eq!(completion.formation_id, Some(7));
        assert_eq!(
            machine.get(5).map(|state| state.phase),
            Some(CollectPhase::Returning)
        );
        assert_eq!(
            machine.get(5).unwrap().award(),
            AwardPb {
                r#type: AWARD_TYPE_LORD_RESOURCE,
                id: RESOURCE_ID_MEAT,
                count: DEFAULT_COLLECT_RESOURCE_AMOUNT,
                safe: Some(true),
                ..Default::default()
            }
        );
    }
}
