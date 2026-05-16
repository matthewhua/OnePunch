use anyhow::Result;
use proto::slg::BaseTroop;
use std::sync::{Arc, Mutex};

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
    CollectCompleted {
        troop_key: i32,
        target_pos: i32,
        march_type: Option<i32>,
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
            | Self::CollectCompleted { .. }
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
        ArrivalAction::Collect => vec![WorldOutboundEvent::CollectCompleted {
            troop_key: troop.key,
            target_pos: pos,
            march_type: troop.r#type,
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
                WorldOutboundEvent::CollectCompleted {
                    troop_key: 2,
                    target_pos: 21,
                    march_type: Some(MARCH_TYPE_MINE_COLLECT),
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
            WorldOutboundEvent::CollectCompleted {
                troop_key: 2,
                target_pos: 21,
                march_type: Some(MARCH_TYPE_MINE_COLLECT),
            }
            .target(),
            WorldOutboundTarget::Home
        );
    }
}
