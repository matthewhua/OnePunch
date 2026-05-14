use super::{BattleUnit, TroopAdvantage};

#[derive(Debug, Clone, Copy)]
pub struct DamageInput<'a> {
    pub attacker: &'a BattleUnit,
    pub defender: &'a BattleUnit,
    pub skill_bonus_permille: i32,
    pub variance_permille: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageOutput {
    pub amount: i64,
    pub base_amount: i64,
    pub advantage: TroopAdvantage,
    pub advantage_permille: i32,
    pub skill_bonus_permille: i32,
    pub variance_permille: i32,
}

pub struct DamageCalculator;

impl DamageCalculator {
    pub fn calculate(input: &DamageInput<'_>) -> DamageOutput {
        let advantage = input
            .attacker
            .troop_kind
            .advantage_against(input.defender.troop_kind);
        let advantage_permille = advantage.damage_permille();
        let skill_permille = (1000 + input.skill_bonus_permille).max(0);
        let variance_permille = input.variance_permille.clamp(950, 1050);
        let base_amount = base_damage(input.attacker, input.defender);
        if base_amount == 0 {
            return DamageOutput {
                amount: 0,
                base_amount,
                advantage,
                advantage_permille,
                skill_bonus_permille: input.skill_bonus_permille,
                variance_permille,
            };
        }

        let amount = ((base_amount as i128
            * advantage_permille as i128
            * skill_permille as i128
            * variance_permille as i128)
            / 1_000_000_000)
            .max(1) as i64;

        DamageOutput {
            amount,
            base_amount,
            advantage,
            advantage_permille,
            skill_bonus_permille: input.skill_bonus_permille,
            variance_permille,
        }
    }
}

fn base_damage(attacker: &BattleUnit, defender: &BattleUnit) -> i64 {
    if !attacker.is_alive() {
        return 0;
    }

    let soldier_coeff = (attacker.soldiers.max(1) as f64 / 100.0).max(1.0);
    let attack_value = attacker.attack.max(0) as f64 * soldier_coeff;
    let defense_value = defender.defense.max(0) as f64 * 0.5;
    (attack_value - defense_value).round().max(1.0) as i64
}
