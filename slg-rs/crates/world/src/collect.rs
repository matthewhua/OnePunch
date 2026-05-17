use crate::arrival::{ArrivalEffect, ArrivalResolution, TargetSummary};
use proto::slg::{AwardPb, BaseTroop};
use shared::static_config::world::{StaticMine, StaticMineReward, WorldConfig};
use std::collections::HashMap;

pub const DEFAULT_COLLECT_DURATION_MS: i64 = 500;
pub const DEFAULT_COLLECT_RETURN_DURATION_MS: i64 = 100;
pub const DEFAULT_COLLECT_RESOURCE_AMOUNT: i64 = 100;
pub const AWARD_TYPE_LORD_RESOURCE: i32 = 1;
pub const RESOURCE_ID_GOLD: i32 = 2;
pub const RESOURCE_ID_MEAT: i32 = 3;
pub const STATIC_MINE_REWARD_TYPE_RESOURCE: i32 = 2;
pub const MINE_TYPE_GRAIN: i32 = 2;
pub const MINE_TYPE_GOLD_INGOT: i32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectProfile {
    pub duration_ms: i64,
    pub collected_amount: i64,
    pub resource_type: i32,
    pub config_issues: Vec<CollectConfigIssue>,
}

impl Default for CollectProfile {
    fn default() -> Self {
        Self {
            duration_ms: DEFAULT_COLLECT_DURATION_MS,
            collected_amount: DEFAULT_COLLECT_RESOURCE_AMOUNT,
            resource_type: RESOURCE_ID_MEAT,
            config_issues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectConfigIssue {
    MissingTarget,
    TargetNotMine {
        entity_type: Option<i32>,
    },
    MissingMineConfId,
    MissingMineConfig {
        mine_id: i32,
    },
    InvalidMineReward {
        mine_id: i32,
        reason: String,
    },
    InvalidMineSpeed {
        mine_id: i32,
        speed: i32,
    },
    UnsupportedMineResource {
        mine_id: i32,
        mine_type: i32,
        reward_type: i32,
        reward_id: i32,
    },
}

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
    pub config_issues: Vec<CollectConfigIssue>,
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
        Self::arrived_with_config(troop, resolution, formation_id, &WorldConfig::default())
    }

    pub fn arrived_with_config(
        troop: &BaseTroop,
        resolution: &ArrivalResolution,
        formation_id: Option<i32>,
        world_config: &WorldConfig,
    ) -> Option<Self> {
        if resolution.effect != ArrivalEffect::CollectStarted {
            return None;
        }

        let start_time_ms = troop.end_time.unwrap_or_default();
        let profile = collect_profile_for_target(resolution.target.as_ref(), world_config);
        Some(Self {
            troop_key: troop.key,
            origin_pos: troop.origin.unwrap_or_default(),
            target_pos: resolution.pos,
            march_type: troop.r#type,
            formation_id,
            start_time_ms,
            collect_end_time_ms: start_time_ms + profile.duration_ms,
            collected_amount: profile.collected_amount,
            resource_type: profile.resource_type,
            camp: troop.camp,
            target: resolution.target.clone(),
            config_issues: profile.config_issues,
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
        self.on_arrival_with_config(troop, resolution, formation_id, &WorldConfig::default())
    }

    pub fn on_arrival_with_config(
        &mut self,
        troop: &BaseTroop,
        resolution: &ArrivalResolution,
        formation_id: Option<i32>,
        world_config: &WorldConfig,
    ) -> Option<&CollectState> {
        let state =
            CollectState::arrived_with_config(troop, resolution, formation_id, world_config)?
                .start_collecting();
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

pub fn collect_profile_for_target(
    target: Option<&TargetSummary>,
    world_config: &WorldConfig,
) -> CollectProfile {
    let Some(target) = target else {
        return profile_with_issue(CollectConfigIssue::MissingTarget);
    };

    let mine_entity_type = proto::slg::WorldEntityTypeDefine::EntityTypeMine as i32;
    if target.entity_type != Some(mine_entity_type) {
        return profile_with_issue(CollectConfigIssue::TargetNotMine {
            entity_type: target.entity_type,
        });
    }

    let Some(mine_id) = target.conf_id.filter(|id| *id > 0) else {
        return profile_with_issue(CollectConfigIssue::MissingMineConfId);
    };

    let Some(mine) = world_config.mine(mine_id) else {
        return profile_with_issue(CollectConfigIssue::MissingMineConfig { mine_id });
    };

    profile_from_mine(mine)
}

fn profile_from_mine(mine: &StaticMine) -> CollectProfile {
    let reward = match mine.parsed_reward() {
        Ok(reward) => reward,
        Err(err) => {
            return profile_with_issue(CollectConfigIssue::InvalidMineReward {
                mine_id: mine.mine_id,
                reason: err.to_string(),
            });
        }
    };

    let mut profile = CollectProfile {
        collected_amount: reward.amount,
        ..CollectProfile::default()
    };

    match mine.collect_duration_ms_for_amount(reward.amount) {
        Some(duration_ms) => profile.duration_ms = duration_ms,
        None => profile
            .config_issues
            .push(CollectConfigIssue::InvalidMineSpeed {
                mine_id: mine.mine_id,
                speed: mine.speed,
            }),
    }

    match lord_resource_id_for_mine_reward(mine, &reward) {
        Some(resource_type) => profile.resource_type = resource_type,
        None => profile
            .config_issues
            .push(CollectConfigIssue::UnsupportedMineResource {
                mine_id: mine.mine_id,
                mine_type: mine.mine_type,
                reward_type: reward.award_type,
                reward_id: reward.resource_id,
            }),
    }

    profile
}

fn lord_resource_id_for_mine_reward(mine: &StaticMine, reward: &StaticMineReward) -> Option<i32> {
    if reward.award_type != STATIC_MINE_REWARD_TYPE_RESOURCE {
        return None;
    }

    match mine.mine_type {
        MINE_TYPE_GRAIN => Some(RESOURCE_ID_MEAT),
        MINE_TYPE_GOLD_INGOT => Some(RESOURCE_ID_GOLD),
        _ => None,
    }
}

fn profile_with_issue(issue: CollectConfigIssue) -> CollectProfile {
    CollectProfile {
        config_issues: vec![issue],
        ..CollectProfile::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arrival::resolve_arrival;
    use crate::march::MARCH_TYPE_MINE_COLLECT;
    use proto::slg::WorldEntityTypeDefine;
    use shared::static_config::world::StaticMine;
    use std::collections::HashMap;

    fn static_mine(mine_id: i32, mine_type: i32, reward: &str, speed: i32) -> StaticMine {
        StaticMine {
            mine_id,
            description: None,
            asset: None,
            mine_type,
            lv: 1,
            weight: 100,
            reward: Some(reward.to_string()),
            speed,
            banner: None,
            sound: None,
        }
    }

    fn world_config_with_mines(mines: Vec<StaticMine>) -> WorldConfig {
        WorldConfig {
            mines: mines.into_iter().map(|mine| (mine.mine_id, mine)).collect(),
            mines_by_type_idx: HashMap::new(),
            ..Default::default()
        }
    }

    fn mine_target(conf_id: i32) -> TargetSummary {
        TargetSummary {
            pos: 202,
            entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
            key_id: Some(9_001),
            camp: None,
            conf_id: Some(conf_id),
            is_battle: None,
        }
    }

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
        assert_eq!(state.config_issues, vec![CollectConfigIssue::MissingTarget]);
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

    #[test]
    fn collect_profile_uses_mine_reward_speed_and_resource_mapping() {
        let config = world_config_with_mines(vec![
            static_mine(201, MINE_TYPE_GRAIN, "[2,3,38400]", 192_000),
            static_mine(401, MINE_TYPE_GOLD_INGOT, "[2,6,1920]", 9_600),
        ]);

        let grain = collect_profile_for_target(Some(&mine_target(201)), &config);
        assert_eq!(
            grain,
            CollectProfile {
                duration_ms: 720_000,
                collected_amount: 38_400,
                resource_type: RESOURCE_ID_MEAT,
                config_issues: Vec::new(),
            }
        );

        let gold = collect_profile_for_target(Some(&mine_target(401)), &config);
        assert_eq!(
            gold,
            CollectProfile {
                duration_ms: 720_000,
                collected_amount: 1_920,
                resource_type: RESOURCE_ID_GOLD,
                config_issues: Vec::new(),
            }
        );
    }

    #[test]
    fn collect_profile_falls_back_when_mine_config_missing() {
        let profile = collect_profile_for_target(Some(&mine_target(999)), &WorldConfig::default());

        assert_eq!(
            profile,
            CollectProfile {
                config_issues: vec![CollectConfigIssue::MissingMineConfig { mine_id: 999 }],
                ..CollectProfile::default()
            }
        );
    }

    #[test]
    fn collect_profile_keeps_config_amount_with_speed_or_resource_fallbacks() {
        let invalid_speed =
            world_config_with_mines(vec![static_mine(201, MINE_TYPE_GRAIN, "[2,3,1200]", 0)]);
        let speed_fallback = collect_profile_for_target(Some(&mine_target(201)), &invalid_speed);
        assert_eq!(speed_fallback.duration_ms, DEFAULT_COLLECT_DURATION_MS);
        assert_eq!(speed_fallback.collected_amount, 1_200);
        assert_eq!(speed_fallback.resource_type, RESOURCE_ID_MEAT);
        assert_eq!(
            speed_fallback.config_issues,
            vec![CollectConfigIssue::InvalidMineSpeed {
                mine_id: 201,
                speed: 0
            }]
        );

        let unsupported_resource =
            world_config_with_mines(vec![static_mine(101, 1, "[2,2,9600]", 48_000)]);
        let resource_fallback =
            collect_profile_for_target(Some(&mine_target(101)), &unsupported_resource);
        assert_eq!(resource_fallback.duration_ms, 720_000);
        assert_eq!(resource_fallback.collected_amount, 9_600);
        assert_eq!(resource_fallback.resource_type, RESOURCE_ID_MEAT);
        assert_eq!(
            resource_fallback.config_issues,
            vec![CollectConfigIssue::UnsupportedMineResource {
                mine_id: 101,
                mine_type: 1,
                reward_type: 2,
                reward_id: 2,
            }]
        );
    }
}
