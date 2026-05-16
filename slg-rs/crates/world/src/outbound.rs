use anyhow::Result;
use proto::slg::BaseTroop;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::arrival::{ArrivalEffect, ArrivalResolution};
use crate::march::{ArrivalAction, MarchArrival};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldOutboundTarget {
    Home,
    Battle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    },
    CollectStarted {
        troop_key: i32,
        target_pos: i32,
        march_type: Option<i32>,
        start_time_ms: i64,
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
}

impl WorldOutboundEvent {
    pub fn target(&self) -> WorldOutboundTarget {
        match self {
            Self::BattleStartRequested { .. } => WorldOutboundTarget::Battle,
            Self::ScoutReportRequested { .. }
            | Self::CollectStarted { .. }
            | Self::TroopReturned { .. }
            | Self::GarrisonChanged { .. } => WorldOutboundTarget::Home,
        }
    }
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
