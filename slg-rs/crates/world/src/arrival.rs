use crate::march::{arrival_action_for_troop, ArrivalAction};
use proto::slg::{BaseEntity, BaseTroop};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetSummary {
    pub pos: i32,
    pub entity_type: Option<i32>,
    pub key_id: Option<i32>,
    pub camp: Option<i32>,
    pub conf_id: Option<i32>,
    pub is_battle: Option<bool>,
}

impl From<&BaseEntity> for TargetSummary {
    fn from(entity: &BaseEntity) -> Self {
        Self {
            pos: entity.pos,
            entity_type: entity.entity_type,
            key_id: entity.key_id,
            camp: entity.camp,
            conf_id: entity.conf_id,
            is_battle: entity.is_battle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrivalEffect {
    BattleRequested,
    CollectStarted,
    ScoutReportRequested,
    GarrisonPlaced,
    ReturnedHome,
    Noop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrivalResolution {
    pub troop_key: i32,
    pub troop_type: Option<i32>,
    pub pos: i32,
    pub target: Option<TargetSummary>,
    pub effect: ArrivalEffect,
}

pub fn resolve_arrival(troop: &BaseTroop, target: Option<&BaseEntity>) -> ArrivalResolution {
    let effect = match arrival_action_for_troop(troop) {
        ArrivalAction::Battle => ArrivalEffect::BattleRequested,
        ArrivalAction::Collect => ArrivalEffect::CollectStarted,
        ArrivalAction::Scout => ArrivalEffect::ScoutReportRequested,
        ArrivalAction::Garrison => ArrivalEffect::GarrisonPlaced,
        ArrivalAction::Return => ArrivalEffect::ReturnedHome,
        ArrivalAction::None => ArrivalEffect::Noop,
    };

    ArrivalResolution {
        troop_key: troop.key,
        troop_type: troop.r#type,
        pos: troop.goal.unwrap_or(0),
        target: target.map(TargetSummary::from),
        effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::march::{
        MARCH_STATUS_RETREAT, MARCH_TYPE_ATK_PLAYER, MARCH_TYPE_GARRISON_CITY,
        MARCH_TYPE_MINE_COLLECT, MARCH_TYPE_SCOUT,
    };
    use proto::slg::WorldEntityTypeDefine;

    fn troop(key: i32, troop_type: i32, goal: i32) -> BaseTroop {
        BaseTroop {
            key,
            r#type: Some(troop_type),
            goal: Some(goal),
            ..Default::default()
        }
    }

    fn target(pos: i32) -> BaseEntity {
        BaseEntity {
            pos,
            entity_type: Some(WorldEntityTypeDefine::EntityTypePlayer as i32),
            key_id: Some(9001),
            camp: Some(2),
            conf_id: Some(7),
            is_battle: Some(true),
            ..Default::default()
        }
    }

    #[test]
    fn resolves_attack_to_battle_request_with_target_summary() {
        let pos = 101;
        let target = target(pos);

        let resolution = resolve_arrival(&troop(1, MARCH_TYPE_ATK_PLAYER, pos), Some(&target));

        assert_eq!(resolution.troop_key, 1);
        assert_eq!(resolution.troop_type, Some(MARCH_TYPE_ATK_PLAYER));
        assert_eq!(resolution.pos, pos);
        assert_eq!(resolution.effect, ArrivalEffect::BattleRequested);
        assert_eq!(
            resolution.target,
            Some(TargetSummary {
                pos,
                entity_type: Some(WorldEntityTypeDefine::EntityTypePlayer as i32),
                key_id: Some(9001),
                camp: Some(2),
                conf_id: Some(7),
                is_battle: Some(true),
            })
        );
    }

    #[test]
    fn resolves_collect_to_collect_started() {
        let resolution = resolve_arrival(&troop(2, MARCH_TYPE_MINE_COLLECT, 202), None);

        assert_eq!(resolution.effect, ArrivalEffect::CollectStarted);
        assert_eq!(resolution.pos, 202);
        assert!(resolution.target.is_none());
    }

    #[test]
    fn resolves_scout_to_report_request() {
        let resolution = resolve_arrival(&troop(3, MARCH_TYPE_SCOUT, 303), None);

        assert_eq!(resolution.effect, ArrivalEffect::ScoutReportRequested);
    }

    #[test]
    fn resolves_garrison_to_garrison_placed() {
        let resolution = resolve_arrival(&troop(4, MARCH_TYPE_GARRISON_CITY, 404), None);

        assert_eq!(resolution.effect, ArrivalEffect::GarrisonPlaced);
    }

    #[test]
    fn resolves_retreating_troop_to_returned_home() {
        let mut troop = troop(5, MARCH_TYPE_ATK_PLAYER, 505);
        troop.status = Some(MARCH_STATUS_RETREAT);

        let resolution = resolve_arrival(&troop, None);

        assert_eq!(resolution.effect, ArrivalEffect::ReturnedHome);
    }

    #[test]
    fn resolves_unknown_type_to_noop() {
        let resolution = resolve_arrival(&troop(6, 9999, 606), None);

        assert_eq!(resolution.effect, ArrivalEffect::Noop);
    }
}
