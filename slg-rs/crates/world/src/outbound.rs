use anyhow::Result;
use prost::Message;
use proto::slg::{
    AwardPb, BaseTroop, WorldBattleFighterSummaryPayload, WorldBattleResultPayload,
    WorldCollectReturnedPayload, WorldCollectStartedPayload, WorldGarrisonChangedPayload,
    WorldOutboundRq, WorldScoutReportRequestedPayload, WorldTroopReturnedPayload,
};
use shared::battle::{BattleOutcome, BattleResult, BattleSide, FighterBattleSummary};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::arrival::{ArrivalEffect, ArrivalResolution};
use crate::march::{ArrivalAction, MarchArrival};

pub const WORLD_OUTBOUND_EVENT_SCOUT_REPORT_REQUESTED: i32 = 1;
pub const WORLD_OUTBOUND_EVENT_COLLECT_STARTED: i32 = 2;
pub const WORLD_OUTBOUND_EVENT_TROOP_RETURNED: i32 = 3;
pub const WORLD_OUTBOUND_EVENT_GARRISON_CHANGED: i32 = 4;
pub const WORLD_OUTBOUND_EVENT_COLLECT_RETURNED: i32 = 5;
pub const WORLD_OUTBOUND_EVENT_BATTLE_RESULT: i32 = 6;

/// World entity types mirroring WorldEntityTypeDefine
pub const ENTITY_TYPE_PLAYER: i32 = 1;
pub const ENTITY_TYPE_BANDIT: i32 = 2;
pub const ENTITY_TYPE_MINE: i32 = 3;
pub const ENTITY_TYPE_CITY: i32 = 4;

/// Scout report data collected from the World side.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoutReportData {
    pub target_entity_type: Option<i32>,
    pub target_owner_id: Option<i64>,
    pub target_camp: Option<i32>,
    pub target_conf_id: Option<i32>,
    pub target_is_battle: bool,
    pub target_protect_time: Option<i32>,
    pub scout_time_ms: i64,
    pub target_resources: Vec<AwardPb>,
    pub garrison_troops: Vec<proto::slg::GarrisonTroop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldOutboundTarget {
    Home,
    Battle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorldOutboundEvent {
    BattleStartRequested {
        troop_key: i32,
        march_type: Option<i32>,
        origin: Option<i32>,
        target_pos: i32,
        camp: Option<i32>,
    },
    ScoutReportRequested {
        troop_key: i32,
        origin: Option<i32>,
        target_pos: i32,
        camp: Option<i32>,
        report: Option<ScoutReportData>,
    },
    CollectStarted {
        troop_key: i32,
        target_pos: i32,
        march_type: Option<i32>,
        start_time_ms: i64,
    },
    CollectReturned {
        troop_key: i32,
        target_pos: i32,
        home_pos: i32,
        march_type: Option<i32>,
        formation_id: Option<i32>,
        awards: Vec<AwardPb>,
        collect_start_time_ms: i64,
        collect_end_time_ms: i64,
    },
    TroopReturned {
        troop_key: i32,
        home_pos: i32,
        march_type: Option<i32>,
    },
    GarrisonChanged {
        troop_key: i32,
        target_pos: i32,
        camp: Option<i32>,
        is_arrival: bool,
    },
    BattleResult {
        troop_key: i32,
        march_type: Option<i32>,
        origin: Option<i32>,
        target_pos: i32,
        camp: Option<i32>,
        result: BattleResult,
        target_owner_id: Option<i64>,
        target_entity_type: Option<i32>,
    },
}

impl WorldOutboundEvent {
    pub fn troop_key(&self) -> i32 {
        match self {
            Self::BattleStartRequested { troop_key, .. }
            | Self::ScoutReportRequested { troop_key, .. }
            | Self::CollectStarted { troop_key, .. }
            | Self::CollectReturned { troop_key, .. }
            | Self::TroopReturned { troop_key, .. }
            | Self::GarrisonChanged { troop_key, .. }
            | Self::BattleResult { troop_key, .. } => *troop_key,
        }
    }

    pub fn target(&self) -> WorldOutboundTarget {
        match self {
            Self::BattleStartRequested { .. } => WorldOutboundTarget::Battle,
            Self::ScoutReportRequested { .. }
            | Self::CollectStarted { .. }
            | Self::CollectReturned { .. }
            | Self::TroopReturned { .. }
            | Self::GarrisonChanged { .. }
            | Self::BattleResult { .. } => WorldOutboundTarget::Home,
        }
    }

    pub fn event_key_for_role(&self, role_id: i64) -> String {
        match self {
            Self::BattleStartRequested {
                troop_key,
                march_type,
                origin,
                target_pos,
                camp,
            } => format!(
                "world:battle_start_requested:role={}:troop={}:target={}:march_type={}:origin={}:camp={}",
                role_id,
                troop_key,
                target_pos,
                optional_i32(*march_type),
                optional_i32(*origin),
                optional_i32(*camp)
            ),
            Self::ScoutReportRequested {
                troop_key,
                origin,
                target_pos,
                camp,
                ..
            } => format!(
                "world:scout_report_requested:role={}:troop={}:target={}:origin={}:camp={}",
                role_id,
                troop_key,
                target_pos,
                optional_i32(*origin),
                optional_i32(*camp)
            ),
            Self::CollectStarted {
                troop_key,
                target_pos,
                march_type,
                start_time_ms,
            } => format!(
                "world:collect_started:role={}:troop={}:target={}:march_type={}:start={}",
                role_id,
                troop_key,
                target_pos,
                optional_i32(*march_type),
                start_time_ms
            ),
            Self::CollectReturned {
                troop_key,
                target_pos,
                home_pos,
                march_type,
                formation_id,
                collect_start_time_ms,
                collect_end_time_ms,
                ..
            } => format!(
                "world:collect_returned:role={}:troop={}:target={}:home={}:march_type={}:formation={}:start={}:end={}",
                role_id,
                troop_key,
                target_pos,
                home_pos,
                optional_i32(*march_type),
                optional_i32(*formation_id),
                collect_start_time_ms,
                collect_end_time_ms
            ),
            Self::TroopReturned {
                troop_key,
                home_pos,
                march_type,
            } => format!(
                "world:troop_returned:role={}:troop={}:home={}:march_type={}",
                role_id,
                troop_key,
                home_pos,
                optional_i32(*march_type)
            ),
            Self::GarrisonChanged {
                troop_key,
                target_pos,
                camp,
                is_arrival,
            } => format!(
                "world:garrison_changed:role={}:troop={}:target={}:camp={}:arrival={}",
                role_id,
                troop_key,
                target_pos,
                optional_i32(*camp),
                is_arrival
            ),
            Self::BattleResult {
                troop_key,
                march_type,
                origin,
                target_pos,
                camp,
                result,
                ..
            } => format!(
                "world:battle_result:role={}:troop={}:target={}:battle={}:march_type={}:origin={}:camp={}",
                role_id,
                troop_key,
                target_pos,
                result.battle_id,
                optional_i32(*march_type),
                optional_i32(*origin),
                optional_i32(*camp)
            ),
        }
    }

    pub fn to_home_request(&self, role_id: i64) -> Result<WorldOutboundRq> {
        if role_id <= 0 {
            return Err(anyhow::anyhow!(
                "role_id is required for Home outbound event: {}",
                role_id
            ));
        }

        let (event_type, world_entity_id, troop_key, payload, context) = match self {
            Self::ScoutReportRequested {
                troop_key,
                origin,
                target_pos,
                camp,
                report,
            } => {
                let (
                    target_entity_type,
                    target_owner_id,
                    target_camp,
                    target_conf_id,
                    target_is_battle,
                    target_protect_time,
                    scout_time_ms,
                    target_resources,
                    garrison_troops,
                ) = if let Some(r) = report {
                    (
                        r.target_entity_type,
                        r.target_owner_id,
                        r.target_camp,
                        r.target_conf_id,
                        Some(r.target_is_battle),
                        r.target_protect_time,
                        Some(r.scout_time_ms),
                        r.target_resources.clone(),
                        r.garrison_troops.clone(),
                    )
                } else {
                    (
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Vec::new(),
                        Vec::new(),
                    )
                };
                (
                    WORLD_OUTBOUND_EVENT_SCOUT_REPORT_REQUESTED,
                    *target_pos,
                    *troop_key,
                    WorldScoutReportRequestedPayload {
                        origin: *origin,
                        target_pos: *target_pos,
                        camp: *camp,
                        target_entity_type,
                        target_owner_id,
                        target_camp,
                        target_conf_id,
                        target_is_battle,
                        target_protect_time,
                        scout_time_ms,
                        target_resources,
                        garrison_troops,
                    }
                    .encode_to_vec(),
                    format!(
                        "scout_report_requested origin={} camp={}",
                        optional_i32(*origin),
                        optional_i32(*camp)
                    ),
                )
            }
            Self::CollectStarted {
                troop_key,
                target_pos,
                march_type,
                start_time_ms,
            } => (
                WORLD_OUTBOUND_EVENT_COLLECT_STARTED,
                *target_pos,
                *troop_key,
                WorldCollectStartedPayload {
                    target_pos: *target_pos,
                    march_type: *march_type,
                    start_time_ms: *start_time_ms,
                }
                .encode_to_vec(),
                format!(
                    "collect_started march_type={} start_time_ms={}",
                    optional_i32(*march_type),
                    start_time_ms
                ),
            ),
            Self::CollectReturned {
                troop_key,
                target_pos,
                home_pos,
                march_type,
                formation_id,
                awards,
                collect_start_time_ms,
                collect_end_time_ms,
            } => (
                WORLD_OUTBOUND_EVENT_COLLECT_RETURNED,
                *target_pos,
                *troop_key,
                WorldCollectReturnedPayload {
                    target_pos: *target_pos,
                    home_pos: *home_pos,
                    march_type: *march_type,
                    formation_id: *formation_id,
                    awards: awards.clone(),
                    collect_start_time_ms: *collect_start_time_ms,
                    collect_end_time_ms: *collect_end_time_ms,
                }
                .encode_to_vec(),
                format!(
                    "collect_returned target_pos={} home_pos={} awards={}",
                    target_pos,
                    home_pos,
                    awards.len()
                ),
            ),
            Self::TroopReturned {
                troop_key,
                home_pos,
                march_type,
            } => (
                WORLD_OUTBOUND_EVENT_TROOP_RETURNED,
                *home_pos,
                *troop_key,
                WorldTroopReturnedPayload {
                    home_pos: *home_pos,
                    march_type: *march_type,
                }
                .encode_to_vec(),
                format!("troop_returned march_type={}", optional_i32(*march_type)),
            ),
            Self::GarrisonChanged {
                troop_key,
                target_pos,
                camp,
                is_arrival,
            } => (
                WORLD_OUTBOUND_EVENT_GARRISON_CHANGED,
                *target_pos,
                *troop_key,
                WorldGarrisonChangedPayload {
                    target_pos: *target_pos,
                    camp: *camp,
                    is_arrival: *is_arrival,
                }
                .encode_to_vec(),
                format!(
                    "garrison_changed camp={} is_arrival={}",
                    optional_i32(*camp),
                    is_arrival
                ),
            ),
            Self::BattleResult {
                troop_key,
                march_type,
                origin,
                target_pos,
                camp,
                result,
                target_owner_id,
                target_entity_type,
            } => {
                let summary = result.summary();
                (
                    WORLD_OUTBOUND_EVENT_BATTLE_RESULT,
                    *target_pos,
                    *troop_key,
                    WorldBattleResultPayload {
                        battle_id: u64_to_i64_saturating(summary.battle_id),
                        target_pos: *target_pos,
                        origin: *origin,
                        march_type: *march_type,
                        camp: *camp,
                        outcome: battle_outcome_name(summary.outcome).to_string(),
                        winner_side: summary
                            .winner
                            .map(battle_side_name)
                            .unwrap_or("Draw")
                            .to_string(),
                        rounds: u32_to_i32_saturating(summary.rounds),
                        total_events: usize_to_i32_saturating(summary.total_events),
                        attacker: Some(fighter_summary_payload(&summary.attacker)),
                        defender: Some(fighter_summary_payload(&summary.defender)),
                        target_owner_id: *target_owner_id,
                        target_entity_type: *target_entity_type,
                    }
                    .encode_to_vec(),
                    format!(
                        "battle_result battle_id={} outcome={} rounds={} attacker_lost={} defender_lost={}",
                        summary.battle_id,
                        battle_outcome_name(summary.outcome),
                        summary.rounds,
                        summary.attacker.units_lost,
                        summary.defender.units_lost
                    ),
                )
            }
            Self::BattleStartRequested { .. } => {
                return Err(anyhow::anyhow!(
                    "battle outbound event cannot be sent to Home"
                ));
            }
        };
        let event_key = self.event_key_for_role(role_id);
        let event_id = stable_event_id(&event_key);

        Ok(WorldOutboundRq {
            role_id,
            event_type,
            world_entity_id,
            troop_key,
            payload,
            context,
            event_id,
            event_key,
        })
    }
}

fn stable_event_id(event_key: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in event_key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:016x}", hash)
}

fn optional_i32(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn battle_outcome_name(outcome: BattleOutcome) -> &'static str {
    match outcome {
        BattleOutcome::AttackerWin => "AttackerWin",
        BattleOutcome::DefenderWin => "DefenderWin",
        BattleOutcome::Draw => "Draw",
    }
}

fn battle_side_name(side: BattleSide) -> &'static str {
    match side {
        BattleSide::Attacker => "Attacker",
        BattleSide::Defender => "Defender",
    }
}

fn fighter_summary_payload(summary: &FighterBattleSummary) -> WorldBattleFighterSummaryPayload {
    WorldBattleFighterSummaryPayload {
        fighter_id: u64_to_i64_saturating(summary.fighter_id),
        initial_units: u64_to_i64_saturating(summary.initial_units),
        remaining_units: u64_to_i64_saturating(summary.remaining_units),
        units_lost: u64_to_i64_saturating(summary.units_lost),
        initial_power: u64_to_i64_saturating(summary.initial_power),
        remaining_power: u64_to_i64_saturating(summary.remaining_power),
        power_lost: u64_to_i64_saturating(summary.power_lost),
        damage_dealt: u64_to_i64_saturating(summary.damage_dealt),
        damage_taken: u64_to_i64_saturating(summary.damage_taken),
        loss_rate_bps: u32_to_i32_saturating(summary.loss_rate_bps),
    }
}

fn u64_to_i64_saturating(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

fn u32_to_i32_saturating(value: u32) -> i32 {
    value.min(i32::MAX as u32) as i32
}

fn usize_to_i32_saturating(value: usize) -> i32 {
    value.min(i32::MAX as usize) as i32
}

pub trait WorldOutboundSink: Send + Sync {
    fn publish(&self, event: WorldOutboundEvent) -> Result<()>;

    fn publish_all<I>(&self, events: I) -> Result<()>
    where
        Self: Sized,
        I: IntoIterator<Item = WorldOutboundEvent>,
    {
        for event in events {
            self.publish(event)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryOutboundSink {
    events: Arc<Mutex<Vec<WorldOutboundEvent>>>,
}

impl InMemoryOutboundSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&self) -> Vec<WorldOutboundEvent> {
        self.events
            .lock()
            .expect("outbound events mutex poisoned")
            .clone()
    }

    pub fn clear(&self) {
        self.events
            .lock()
            .expect("outbound events mutex poisoned")
            .clear();
    }
}

impl WorldOutboundSink for InMemoryOutboundSink {
    fn publish(&self, event: WorldOutboundEvent) -> Result<()> {
        self.events
            .lock()
            .expect("outbound events mutex poisoned")
            .push(event);
        Ok(())
    }
}

#[derive(Clone)]
pub struct ChannelOutboundSink {
    tx: mpsc::UnboundedSender<WorldOutboundEvent>,
}

impl ChannelOutboundSink {
    pub fn new(tx: mpsc::UnboundedSender<WorldOutboundEvent>) -> Self {
        Self { tx }
    }
}

impl WorldOutboundSink for ChannelOutboundSink {
    fn publish(&self, event: WorldOutboundEvent) -> Result<()> {
        self.tx
            .send(event)
            .map_err(|err| anyhow::anyhow!("world outbound receiver dropped: {}", err))
    }
}

pub struct HomeOutboundChannelSink<R>
where
    R: Fn(&WorldOutboundEvent) -> Option<i64> + Send + Sync,
{
    tx: mpsc::UnboundedSender<WorldOutboundRq>,
    role_resolver: R,
}

impl<R> HomeOutboundChannelSink<R>
where
    R: Fn(&WorldOutboundEvent) -> Option<i64> + Send + Sync,
{
    pub fn new(tx: mpsc::UnboundedSender<WorldOutboundRq>, role_resolver: R) -> Self {
        Self { tx, role_resolver }
    }
}

impl<R> WorldOutboundSink for HomeOutboundChannelSink<R>
where
    R: Fn(&WorldOutboundEvent) -> Option<i64> + Send + Sync,
{
    fn publish(&self, event: WorldOutboundEvent) -> Result<()> {
        let role_id = (self.role_resolver)(&event)
            .ok_or_else(|| anyhow::anyhow!("role_id resolver returned none for {:?}", event))?;
        let request = event.to_home_request(role_id)?;
        self.tx
            .send(request)
            .map_err(|err| anyhow::anyhow!("home outbound request receiver dropped: {}", err))
    }
}

#[derive(Clone)]
pub struct WorldOutboundDispatcher {
    home: Arc<dyn WorldOutboundSink>,
    battle_placeholder: Arc<dyn WorldOutboundSink>,
}

impl WorldOutboundDispatcher {
    pub fn new(
        home: Arc<dyn WorldOutboundSink>,
        battle_placeholder: Arc<dyn WorldOutboundSink>,
    ) -> Self {
        Self {
            home,
            battle_placeholder,
        }
    }
}

impl WorldOutboundSink for WorldOutboundDispatcher {
    fn publish(&self, event: WorldOutboundEvent) -> Result<()> {
        match event.target() {
            WorldOutboundTarget::Home => self.home.publish(event),
            WorldOutboundTarget::Battle => self.battle_placeholder.publish(event),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrivalOutboundInput {
    pub troop: BaseTroop,
    pub action: ArrivalAction,
    pub pos: i32,
}

impl From<MarchArrival> for ArrivalOutboundInput {
    fn from(arrival: MarchArrival) -> Self {
        Self {
            troop: arrival.troop,
            action: arrival.action,
            pos: arrival.pos,
        }
    }
}

impl From<&MarchArrival> for ArrivalOutboundInput {
    fn from(arrival: &MarchArrival) -> Self {
        Self {
            troop: arrival.troop.clone(),
            action: arrival.action,
            pos: arrival.pos,
        }
    }
}

pub fn outbound_events_for_arrival(
    arrival: impl Into<ArrivalOutboundInput>,
) -> Vec<WorldOutboundEvent> {
    let arrival = arrival.into();
    outbound_events_for_action(&arrival.troop, arrival.action, arrival.pos)
}

pub fn outbound_events_for_resolution(
    troop: &BaseTroop,
    resolution: &ArrivalResolution,
) -> Vec<WorldOutboundEvent> {
    match resolution.effect {
        ArrivalEffect::Noop => Vec::new(),
        ArrivalEffect::BattleRequested => vec![WorldOutboundEvent::BattleStartRequested {
            troop_key: troop.key,
            march_type: troop.r#type,
            origin: troop.origin,
            target_pos: resolution.pos,
            camp: troop.camp,
        }],
        ArrivalEffect::CollectStarted => vec![WorldOutboundEvent::CollectStarted {
            troop_key: troop.key,
            target_pos: resolution.pos,
            march_type: troop.r#type,
            start_time_ms: troop.end_time.unwrap_or_default(),
        }],
        ArrivalEffect::ScoutReportRequested => vec![WorldOutboundEvent::ScoutReportRequested {
            troop_key: troop.key,
            origin: troop.origin,
            target_pos: resolution.pos,
            camp: troop.camp,
            report: None,
        }],
        ArrivalEffect::GarrisonPlaced => vec![WorldOutboundEvent::GarrisonChanged {
            troop_key: troop.key,
            target_pos: resolution.pos,
            camp: troop.camp,
            is_arrival: true,
        }],
        ArrivalEffect::ReturnedHome => vec![WorldOutboundEvent::TroopReturned {
            troop_key: troop.key,
            home_pos: resolution.pos,
            march_type: troop.r#type,
        }],
    }
}

pub fn outbound_events_for_action(
    troop: &BaseTroop,
    action: ArrivalAction,
    pos: i32,
) -> Vec<WorldOutboundEvent> {
    match action {
        ArrivalAction::None => Vec::new(),
        ArrivalAction::Battle => vec![WorldOutboundEvent::BattleStartRequested {
            troop_key: troop.key,
            march_type: troop.r#type,
            origin: troop.origin,
            target_pos: pos,
            camp: troop.camp,
        }],
        ArrivalAction::Collect => vec![WorldOutboundEvent::CollectStarted {
            troop_key: troop.key,
            target_pos: pos,
            march_type: troop.r#type,
            start_time_ms: troop.end_time.unwrap_or_default(),
        }],
        ArrivalAction::Scout => vec![WorldOutboundEvent::ScoutReportRequested {
            troop_key: troop.key,
            origin: troop.origin,
            target_pos: pos,
            camp: troop.camp,
            report: None,
        }],
        ArrivalAction::Garrison => vec![WorldOutboundEvent::GarrisonChanged {
            troop_key: troop.key,
            target_pos: pos,
            camp: troop.camp,
            is_arrival: true,
        }],
        ArrivalAction::Return => vec![WorldOutboundEvent::TroopReturned {
            troop_key: troop.key,
            home_pos: pos,
            march_type: troop.r#type,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::march::{
        MARCH_TYPE_ATK_PLAYER, MARCH_TYPE_GARRISON_CITY, MARCH_TYPE_MINE_COLLECT, MARCH_TYPE_SCOUT,
    };
    use shared::battle::{resolve_battle, BattleInput, Fighter, Unit};

    fn troop(key: i32, troop_type: i32, origin: i32, goal: i32) -> BaseTroop {
        BaseTroop {
            key,
            r#type: Some(troop_type),
            origin: Some(origin),
            goal: Some(goal),
            camp: Some(7),
            ..Default::default()
        }
    }

    #[test]
    fn records_events_in_publish_order() {
        let sink = InMemoryOutboundSink::new();

        sink.publish(WorldOutboundEvent::ScoutReportRequested {
            troop_key: 10,
            origin: Some(1),
            target_pos: 2,
            camp: Some(7),
            report: None,
        })
        .unwrap();
        sink.publish(WorldOutboundEvent::TroopReturned {
            troop_key: 11,
            home_pos: 1,
            march_type: Some(MARCH_TYPE_SCOUT),
        })
        .unwrap();

        assert_eq!(
            sink.records(),
            vec![
                WorldOutboundEvent::ScoutReportRequested {
                    troop_key: 10,
                    origin: Some(1),
                    target_pos: 2,
                    camp: Some(7),
                    report: None,
                },
                WorldOutboundEvent::TroopReturned {
                    troop_key: 11,
                    home_pos: 1,
                    march_type: Some(MARCH_TYPE_SCOUT),
                },
            ]
        );
    }

    #[test]
    fn publish_all_preserves_helper_event_order() {
        let sink = InMemoryOutboundSink::new();
        let arrivals = vec![
            ArrivalOutboundInput {
                troop: troop(1, MARCH_TYPE_ATK_PLAYER, 10, 20),
                action: ArrivalAction::Battle,
                pos: 20,
            },
            ArrivalOutboundInput {
                troop: troop(2, MARCH_TYPE_MINE_COLLECT, 10, 21),
                action: ArrivalAction::Collect,
                pos: 21,
            },
            ArrivalOutboundInput {
                troop: troop(3, MARCH_TYPE_SCOUT, 10, 22),
                action: ArrivalAction::Scout,
                pos: 22,
            },
        ];

        let events = arrivals
            .into_iter()
            .flat_map(outbound_events_for_arrival)
            .collect::<Vec<_>>();
        sink.publish_all(events).unwrap();

        assert_eq!(
            sink.records(),
            vec![
                WorldOutboundEvent::BattleStartRequested {
                    troop_key: 1,
                    march_type: Some(MARCH_TYPE_ATK_PLAYER),
                    origin: Some(10),
                    target_pos: 20,
                    camp: Some(7),
                },
                WorldOutboundEvent::CollectStarted {
                    troop_key: 2,
                    target_pos: 21,
                    march_type: Some(MARCH_TYPE_MINE_COLLECT),
                    start_time_ms: 0,
                },
                WorldOutboundEvent::ScoutReportRequested {
                    troop_key: 3,
                    origin: Some(10),
                    target_pos: 22,
                    camp: Some(7),
                    report: None,
                },
            ]
        );
    }

    #[test]
    fn maps_return_and_garrison_arrivals() {
        let return_events = outbound_events_for_action(
            &troop(4, MARCH_TYPE_ATK_PLAYER, 30, 10),
            ArrivalAction::Return,
            10,
        );
        let garrison_events = outbound_events_for_action(
            &troop(5, MARCH_TYPE_GARRISON_CITY, 30, 40),
            ArrivalAction::Garrison,
            40,
        );

        assert_eq!(
            return_events,
            vec![WorldOutboundEvent::TroopReturned {
                troop_key: 4,
                home_pos: 10,
                march_type: Some(MARCH_TYPE_ATK_PLAYER),
            }]
        );
        assert_eq!(
            garrison_events,
            vec![WorldOutboundEvent::GarrisonChanged {
                troop_key: 5,
                target_pos: 40,
                camp: Some(7),
                is_arrival: true,
            }]
        );
    }

    #[test]
    fn none_action_emits_no_event() {
        assert!(outbound_events_for_action(
            &troop(6, MARCH_TYPE_ATK_PLAYER, 10, 20),
            ArrivalAction::None,
            20,
        )
        .is_empty());
    }

    #[test]
    fn event_target_marks_battle_vs_home_boundary() {
        assert_eq!(
            WorldOutboundEvent::BattleStartRequested {
                troop_key: 1,
                march_type: Some(MARCH_TYPE_ATK_PLAYER),
                origin: Some(10),
                target_pos: 20,
                camp: Some(7),
            }
            .target(),
            WorldOutboundTarget::Battle
        );
        assert_eq!(
            WorldOutboundEvent::CollectStarted {
                troop_key: 2,
                target_pos: 21,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
                start_time_ms: 0,
            }
            .target(),
            WorldOutboundTarget::Home
        );
        assert_eq!(
            WorldOutboundEvent::CollectReturned {
                troop_key: 3,
                target_pos: 21,
                home_pos: 10,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
                formation_id: Some(7),
                awards: Vec::new(),
                collect_start_time_ms: 0,
                collect_end_time_ms: 500,
            }
            .target(),
            WorldOutboundTarget::Home
        );
        let battle_result = resolve_battle(&BattleInput::new(
            99,
            Fighter::new(1, vec![Unit::new(1, 1, 10, 12, 5, 10)]),
            Fighter::new(2, vec![Unit::new(2, 2, 8, 10, 4, 10)]),
        ))
        .unwrap();
        assert_eq!(
            WorldOutboundEvent::BattleResult {
                troop_key: 4,
                march_type: Some(MARCH_TYPE_ATK_PLAYER),
                origin: Some(10),
                target_pos: 20,
                camp: Some(7),
                result: battle_result,
                target_owner_id: Some(900_002),
                target_entity_type: Some(ENTITY_TYPE_PLAYER),
            }
            .target(),
            WorldOutboundTarget::Home
        );
    }

    #[tokio::test]
    async fn dispatcher_routes_home_events_to_channel_and_keeps_battle_placeholder() {
        let (home_tx, mut home_rx) = mpsc::unbounded_channel();
        let battle = Arc::new(InMemoryOutboundSink::new());
        let dispatcher = WorldOutboundDispatcher::new(
            Arc::new(ChannelOutboundSink::new(home_tx)),
            battle.clone(),
        );

        let home_event = WorldOutboundEvent::CollectStarted {
            troop_key: 2,
            target_pos: 21,
            march_type: Some(MARCH_TYPE_MINE_COLLECT),
            start_time_ms: 0,
        };
        let battle_event = WorldOutboundEvent::BattleStartRequested {
            troop_key: 1,
            march_type: Some(MARCH_TYPE_ATK_PLAYER),
            origin: Some(10),
            target_pos: 20,
            camp: Some(7),
        };

        dispatcher.publish(home_event.clone()).unwrap();
        dispatcher.publish(battle_event.clone()).unwrap();

        assert_eq!(home_rx.recv().await, Some(home_event));
        assert_eq!(battle.records(), vec![battle_event]);
    }

    #[tokio::test]
    async fn home_channel_sink_builds_world_outbound_request() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sink = HomeOutboundChannelSink::new(tx, |_event| Some(900_001));

        sink.publish(WorldOutboundEvent::TroopReturned {
            troop_key: 44,
            home_pos: 101,
            march_type: Some(MARCH_TYPE_ATK_PLAYER),
        })
        .unwrap();

        let request = rx.recv().await.unwrap();
        assert_eq!(request.role_id, 900_001);
        assert_eq!(request.event_type, WORLD_OUTBOUND_EVENT_TROOP_RETURNED);
        assert_eq!(request.world_entity_id, 101);
        assert_eq!(request.troop_key, 44);
        assert!(!request.event_id.is_empty());
        assert!(request.event_key.contains("world:troop_returned"));
        assert!(request.event_key.contains("role=900001"));
        let payload = WorldTroopReturnedPayload::decode(request.payload.as_slice()).unwrap();
        assert_eq!(payload.home_pos, 101);
        assert_eq!(payload.march_type, Some(MARCH_TYPE_ATK_PLAYER));
        assert!(request.context.contains("troop_returned"));
    }

    #[test]
    fn home_requests_encode_typed_non_combat_payloads() {
        let scout = WorldOutboundEvent::ScoutReportRequested {
            troop_key: 10,
            origin: Some(1),
            target_pos: 2,
            camp: Some(7),
            report: None,
        }
        .to_home_request(900_001)
        .unwrap();
        let scout_payload =
            WorldScoutReportRequestedPayload::decode(scout.payload.as_slice()).unwrap();
        assert_eq!(scout_payload.origin, Some(1));
        assert_eq!(scout_payload.target_pos, 2);
        assert_eq!(scout_payload.camp, Some(7));

        let collect = WorldOutboundEvent::CollectStarted {
            troop_key: 11,
            target_pos: 3,
            march_type: Some(MARCH_TYPE_MINE_COLLECT),
            start_time_ms: 12_000,
        }
        .to_home_request(900_001)
        .unwrap();
        let collect_payload =
            WorldCollectStartedPayload::decode(collect.payload.as_slice()).unwrap();
        assert_eq!(collect_payload.target_pos, 3);
        assert_eq!(collect_payload.march_type, Some(MARCH_TYPE_MINE_COLLECT));
        assert_eq!(collect_payload.start_time_ms, 12_000);

        let garrison = WorldOutboundEvent::GarrisonChanged {
            troop_key: 12,
            target_pos: 4,
            camp: Some(2),
            is_arrival: true,
        }
        .to_home_request(900_001)
        .unwrap();
        let garrison_payload =
            WorldGarrisonChangedPayload::decode(garrison.payload.as_slice()).unwrap();
        assert_eq!(garrison_payload.target_pos, 4);
        assert_eq!(garrison_payload.camp, Some(2));
        assert!(garrison_payload.is_arrival);

        let returned = WorldOutboundEvent::CollectReturned {
            troop_key: 13,
            target_pos: 5,
            home_pos: 1,
            march_type: Some(MARCH_TYPE_MINE_COLLECT),
            formation_id: Some(7),
            awards: vec![AwardPb {
                r#type: crate::collect::AWARD_TYPE_LORD_RESOURCE,
                id: crate::collect::RESOURCE_ID_MEAT,
                count: 100,
                safe: Some(true),
                ..Default::default()
            }],
            collect_start_time_ms: 12_000,
            collect_end_time_ms: 12_500,
        }
        .to_home_request(900_001)
        .unwrap();
        assert_eq!(returned.event_type, WORLD_OUTBOUND_EVENT_COLLECT_RETURNED);
        assert_eq!(
            returned.event_key,
            "world:collect_returned:role=900001:troop=13:target=5:home=1:march_type=3:formation=7:start=12000:end=12500"
        );
        assert_eq!(
            returned.event_id,
            stable_event_id("world:collect_returned:role=900001:troop=13:target=5:home=1:march_type=3:formation=7:start=12000:end=12500")
        );
        let returned_payload =
            WorldCollectReturnedPayload::decode(returned.payload.as_slice()).unwrap();
        assert_eq!(returned_payload.target_pos, 5);
        assert_eq!(returned_payload.home_pos, 1);
        assert_eq!(returned_payload.formation_id, Some(7));
        assert_eq!(returned_payload.awards.len(), 1);
        assert_eq!(returned_payload.awards[0].count, 100);

        let result = resolve_battle(&BattleInput::new(
            99,
            Fighter::new(1, vec![Unit::new(1, 1, 10, 12, 5, 10)]),
            Fighter::new(2, vec![Unit::new(2, 2, 8, 10, 4, 10)]),
        ))
        .unwrap();
        let battle = WorldOutboundEvent::BattleResult {
            troop_key: 14,
            march_type: Some(MARCH_TYPE_ATK_PLAYER),
            origin: Some(1),
            target_pos: 6,
            camp: Some(7),
            result,
            target_owner_id: Some(900_002),
            target_entity_type: Some(ENTITY_TYPE_PLAYER),
        }
        .to_home_request(900_001)
        .unwrap();
        assert_eq!(battle.event_type, WORLD_OUTBOUND_EVENT_BATTLE_RESULT);
        assert_eq!(
            battle.event_key,
            "world:battle_result:role=900001:troop=14:target=6:battle=99:march_type=2:origin=1:camp=7"
        );
        let battle_payload = WorldBattleResultPayload::decode(battle.payload.as_slice()).unwrap();
        assert_eq!(battle_payload.battle_id, 99);
        assert_eq!(battle_payload.target_pos, 6);
        assert_eq!(battle_payload.target_owner_id, Some(900_002));
        assert_eq!(battle_payload.target_entity_type, Some(ENTITY_TYPE_PLAYER));
        assert!(battle_payload.attacker.unwrap().initial_units > 0);
        assert!(battle_payload.defender.unwrap().initial_units > 0);
    }

    #[test]
    fn home_request_rejects_missing_role_or_battle_target() {
        let home_event = WorldOutboundEvent::CollectStarted {
            troop_key: 2,
            target_pos: 21,
            march_type: Some(MARCH_TYPE_MINE_COLLECT),
            start_time_ms: 0,
        };
        assert!(home_event.to_home_request(0).is_err());

        let battle_event = WorldOutboundEvent::BattleStartRequested {
            troop_key: 1,
            march_type: Some(MARCH_TYPE_ATK_PLAYER),
            origin: Some(10),
            target_pos: 20,
            camp: Some(7),
        };
        assert!(battle_event.to_home_request(900_001).is_err());
    }

    #[test]
    fn maps_arrival_resolution_to_collect_started() {
        let mut collect_troop = troop(7, MARCH_TYPE_MINE_COLLECT, 10, 21);
        collect_troop.end_time = Some(12_000);
        let resolution = crate::arrival::resolve_arrival(&collect_troop, None);

        assert_eq!(
            outbound_events_for_resolution(&collect_troop, &resolution),
            vec![WorldOutboundEvent::CollectStarted {
                troop_key: 7,
                target_pos: 21,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
                start_time_ms: 12_000,
            }]
        );
    }
}
