mod calculator;
mod model;
mod report;
mod result;
mod round;
mod skill;

pub use calculator::{DamageCalculator, DamageInput, DamageOutput};
pub use model::{
    BattleArmy, BattleKind, BattleOptions, BattleRequest, BattleSide, BattleUnit, BattleVariance,
    TroopAdvantage, TroopKind, DEFAULT_MAX_ROUNDS,
};
pub use report::{BattleActionType, BattleReport, BattleReportAction};
pub use result::{BattleEndReason, BattleLossSummary, BattleOutcome, BattleSideLoss, BattleWinner};
pub use round::BattleEngine;
pub use skill::{BattleSkill, SkillEffect, SkillTrigger};

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(
        id: i32,
        side: BattleSide,
        troop_kind: TroopKind,
        soldiers: i64,
        speed: i32,
    ) -> BattleUnit {
        BattleUnit {
            id,
            owner_id: format!("role-{id}"),
            hero_id: id,
            side,
            troop_kind,
            soldiers,
            attack: 100,
            defense: 20,
            speed,
            skills: Vec::new(),
        }
    }

    #[test]
    fn damage_calculation_applies_troop_counter() {
        let attacker = unit(1, BattleSide::Attacker, TroopKind::Infantry, 100, 10);
        let cavalry = unit(2, BattleSide::Defender, TroopKind::Cavalry, 100, 10);
        let archer = unit(3, BattleSide::Defender, TroopKind::Archer, 100, 10);

        let advantaged = DamageCalculator::calculate(&DamageInput {
            attacker: &attacker,
            defender: &cavalry,
            skill_bonus_permille: 0,
            variance_permille: 1000,
        });
        let disadvantaged = DamageCalculator::calculate(&DamageInput {
            attacker: &attacker,
            defender: &archer,
            skill_bonus_permille: 0,
            variance_permille: 1000,
        });

        assert!(advantaged.amount > disadvantaged.amount);
        assert_eq!(advantaged.advantage, TroopAdvantage::Advantaged);
        assert_eq!(disadvantaged.advantage, TroopAdvantage::Disadvantaged);
    }

    #[test]
    fn dead_unit_deals_no_damage_when_calculator_is_called_directly() {
        let attacker = unit(1, BattleSide::Attacker, TroopKind::Infantry, 0, 10);
        let defender = unit(2, BattleSide::Defender, TroopKind::Cavalry, 100, 10);

        let damage = DamageCalculator::calculate(&DamageInput {
            attacker: &attacker,
            defender: &defender,
            skill_bonus_permille: 0,
            variance_permille: 1000,
        });

        assert_eq!(damage.amount, 0);
    }

    #[test]
    fn faster_unit_acts_first_and_report_keeps_action_order() {
        let mut defender = unit(2, BattleSide::Defender, TroopKind::Cavalry, 300, 30);
        defender.attack = 1;
        let mut attacker = unit(1, BattleSide::Attacker, TroopKind::Infantry, 300, 10);
        attacker.attack = 1;

        let outcome = BattleEngine::resolve(BattleRequest {
            attacker: BattleArmy::new("atk", vec![attacker]),
            defender: BattleArmy::new("def", vec![defender]),
            options: BattleOptions {
                max_rounds: 1,
                battle_kind: BattleKind::Field,
                variance: BattleVariance::Fixed(1000),
                death_permille: 500,
            },
        });

        assert_eq!(outcome.report.actions[0].actor_side, BattleSide::Defender);
        assert_eq!(outcome.report.actions[0].actor_id, 2);
        assert_eq!(outcome.rounds, 1);
    }

    #[test]
    fn active_skill_triggers_on_configured_round() {
        let mut attacker = unit(1, BattleSide::Attacker, TroopKind::Infantry, 500, 20);
        attacker.attack = 10;
        attacker.skills.push(BattleSkill {
            id: 9001,
            trigger: SkillTrigger::ActiveEvery { rounds: 2 },
            effect: SkillEffect::DamageBonus {
                bonus_permille: 500,
            },
        });
        let mut defender = unit(2, BattleSide::Defender, TroopKind::Cavalry, 500, 10);
        defender.attack = 1;

        let outcome = BattleEngine::resolve(BattleRequest {
            attacker: BattleArmy::new("atk", vec![attacker]),
            defender: BattleArmy::new("def", vec![defender]),
            options: BattleOptions {
                max_rounds: 2,
                battle_kind: BattleKind::Field,
                variance: BattleVariance::Fixed(1000),
                death_permille: 500,
            },
        });

        assert!(outcome.report.actions.iter().any(|action| action.round == 2
            && action.action_type == BattleActionType::Skill
            && action.skill_id == Some(9001)));
    }

    #[test]
    fn max_rounds_uses_battle_kind_tie_breaker() {
        let mut attacker = unit(1, BattleSide::Attacker, TroopKind::Infantry, 300, 20);
        attacker.attack = 1;
        let mut defender = unit(2, BattleSide::Defender, TroopKind::Cavalry, 300, 10);
        defender.attack = 1;

        let outcome = BattleEngine::resolve(BattleRequest {
            attacker: BattleArmy::new("atk", vec![attacker]),
            defender: BattleArmy::new("def", vec![defender]),
            options: BattleOptions {
                max_rounds: 1,
                battle_kind: BattleKind::Siege,
                variance: BattleVariance::Fixed(1000),
                death_permille: 500,
            },
        });

        assert_eq!(outcome.winner, BattleWinner::Defender);
        assert_eq!(outcome.reason, BattleEndReason::MaxRounds);
    }

    #[test]
    fn report_can_be_serialized_to_proto() {
        let attacker = unit(1, BattleSide::Attacker, TroopKind::Infantry, 100, 20);
        let defender = unit(2, BattleSide::Defender, TroopKind::Cavalry, 100, 10);

        let outcome = BattleEngine::resolve(BattleRequest {
            attacker: BattleArmy::new("atk", vec![attacker]),
            defender: BattleArmy::new("def", vec![defender]),
            options: BattleOptions {
                max_rounds: 1,
                battle_kind: BattleKind::Field,
                variance: BattleVariance::Fixed(1000),
                death_permille: 500,
            },
        });

        let proto = outcome.report.to_proto("report-1");
        assert_eq!(proto.key_id, "report-1");
        assert_eq!(proto.actions.len(), outcome.report.actions.len());
        assert_eq!(proto.actions[0].atk_hero_id, Some(1));
        assert_eq!(proto.actions[0].dest_hero_ids, vec![2]);
        assert!(outcome.report.encode_proto("report-1").unwrap().len() > 0);
    }
}
