use super::skill::{active_skill_for_round, passive_damage_bonus_permille};
use super::{
    BattleActionType, BattleArmy, BattleEndReason, BattleKind, BattleLossSummary, BattleOptions,
    BattleOutcome, BattleReport, BattleReportAction, BattleRequest, BattleSide, BattleSideLoss,
    BattleUnit, BattleWinner, DamageCalculator, DamageInput, SkillEffect, TroopAdvantage,
};

pub struct BattleEngine;

impl BattleEngine {
    pub fn resolve(request: BattleRequest) -> BattleOutcome {
        let options = request.options.normalized();
        let initial_attacker = request.attacker.initial_soldiers();
        let initial_defender = request.defender.initial_soldiers();
        let mut state = BattleState {
            attacker: request.attacker,
            defender: request.defender,
            options,
            report: BattleReport::default(),
            action_index: 0,
        };

        if let Some(winner) = winner_by_elimination(&state.attacker, &state.defender) {
            return state.finish(
                winner,
                BattleEndReason::NoUnits,
                0,
                initial_attacker,
                initial_defender,
            );
        }

        let max_rounds = state.options.max_rounds;
        for round in 1..=max_rounds {
            state.execute_round(round);

            if let Some(winner) = winner_by_elimination(&state.attacker, &state.defender) {
                return state.finish(
                    winner,
                    BattleEndReason::Eliminated,
                    round,
                    initial_attacker,
                    initial_defender,
                );
            }
        }

        let winner = winner_by_timeout(
            state.options.battle_kind,
            state.attacker.alive_soldiers(),
            state.defender.alive_soldiers(),
        );
        state.finish(
            winner,
            BattleEndReason::MaxRounds,
            max_rounds,
            initial_attacker,
            initial_defender,
        )
    }
}

struct BattleState {
    attacker: BattleArmy,
    defender: BattleArmy,
    options: BattleOptions,
    report: BattleReport,
    action_index: u32,
}

impl BattleState {
    fn execute_round(&mut self, round: u32) {
        let order = self.action_order();
        for actor in order {
            if !self.unit(actor.side, actor.index).is_alive() {
                continue;
            }

            let target_side = actor.side.opponent();
            let Some(target_index) = self.select_target(target_side) else {
                return;
            };

            self.action_index = self.action_index.saturating_add(1);
            self.execute_action(round, actor, target_side, target_index);

            if !self.army(target_side).has_alive_units() {
                return;
            }
        }
    }

    fn execute_action(
        &mut self,
        round: u32,
        actor_ref: UnitRef,
        target_side: BattleSide,
        target_index: usize,
    ) {
        let actor = self.unit(actor_ref.side, actor_ref.index).clone();
        let target = self.unit(target_side, target_index).clone();
        let active_skill = active_skill_for_round(&actor.skills, round);

        if let Some(skill) = active_skill {
            if let SkillEffect::Heal { heal_permille } = skill.effect {
                self.apply_heal(round, actor_ref, skill.id, heal_permille);
                return;
            }
        }

        let active_bonus = active_skill
            .and_then(|skill| match skill.effect {
                SkillEffect::DamageBonus { bonus_permille } => Some(bonus_permille),
                SkillEffect::Heal { .. } => None,
            })
            .unwrap_or(0);
        let skill_bonus = passive_damage_bonus_permille(&actor.skills) + active_bonus;
        let variance =
            self.options
                .variance
                .permille(round, self.action_index, actor.id, target.id);
        let damage = DamageCalculator::calculate(&DamageInput {
            attacker: &actor,
            defender: &target,
            skill_bonus_permille: skill_bonus,
            variance_permille: variance,
        });
        let applied = damage.amount.min(target.soldiers.max(0));
        let remaining = {
            let target = self.unit_mut(target_side, target_index);
            target.soldiers = (target.soldiers - applied).max(0);
            target.soldiers
        };

        let mut action = BattleReportAction::attack(
            round,
            self.action_index,
            actor.side,
            actor.id,
            target.side,
            target.id,
            applied,
            remaining,
            damage.advantage,
        );
        if let Some(skill) = active_skill {
            action.action_type = BattleActionType::Skill;
            action.skill_id = Some(skill.id);
        }
        self.report.push(action);
    }

    fn apply_heal(&mut self, round: u32, actor_ref: UnitRef, skill_id: i32, heal_permille: i32) {
        let actor = self.unit(actor_ref.side, actor_ref.index).clone();
        let healing = (actor.attack.max(0) * heal_permille.max(0) as i64 / 1000).max(1);
        let remaining = {
            let unit = self.unit_mut(actor_ref.side, actor_ref.index);
            unit.soldiers = unit.soldiers.saturating_add(healing);
            unit.soldiers
        };
        self.report.push(BattleReportAction {
            round,
            action_index: self.action_index,
            actor_side: actor.side,
            actor_id: actor.id,
            target_side: actor.side,
            target_id: actor.id,
            action_type: BattleActionType::Heal,
            skill_id: Some(skill_id),
            damage: 0,
            healing,
            remaining_soldiers: remaining,
            advantage: TroopAdvantage::Neutral,
            time_offset_ms: self.action_index.saturating_mul(500) as i32,
        });
    }

    fn action_order(&self) -> Vec<UnitRef> {
        let mut order: Vec<UnitRef> = self
            .attacker
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.is_alive())
            .map(|(index, unit)| UnitRef {
                side: BattleSide::Attacker,
                index,
                speed: unit.speed,
                id: unit.id,
            })
            .chain(
                self.defender
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, unit)| unit.is_alive())
                    .map(|(index, unit)| UnitRef {
                        side: BattleSide::Defender,
                        index,
                        speed: unit.speed,
                        id: unit.id,
                    }),
            )
            .collect();

        order.sort_by(|left, right| {
            right
                .speed
                .cmp(&left.speed)
                .then_with(|| side_order(left.side).cmp(&side_order(right.side)))
                .then_with(|| left.id.cmp(&right.id))
        });
        order
    }

    fn select_target(&self, side: BattleSide) -> Option<usize> {
        self.army(side)
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.is_alive())
            .min_by_key(|(_, unit)| (unit.soldiers, unit.id))
            .map(|(index, _)| index)
    }

    fn unit(&self, side: BattleSide, index: usize) -> &BattleUnit {
        &self.army(side).units[index]
    }

    fn unit_mut(&mut self, side: BattleSide, index: usize) -> &mut BattleUnit {
        &mut self.army_mut(side).units[index]
    }

    fn army(&self, side: BattleSide) -> &BattleArmy {
        match side {
            BattleSide::Attacker => &self.attacker,
            BattleSide::Defender => &self.defender,
        }
    }

    fn army_mut(&mut self, side: BattleSide) -> &mut BattleArmy {
        match side {
            BattleSide::Attacker => &mut self.attacker,
            BattleSide::Defender => &mut self.defender,
        }
    }

    fn finish(
        self,
        winner: BattleWinner,
        reason: BattleEndReason,
        rounds: u32,
        initial_attacker: i64,
        initial_defender: i64,
    ) -> BattleOutcome {
        BattleOutcome {
            winner,
            reason,
            rounds,
            losses: BattleLossSummary {
                attacker: BattleSideLoss::new(
                    initial_attacker,
                    self.attacker.alive_soldiers(),
                    self.options.death_permille,
                ),
                defender: BattleSideLoss::new(
                    initial_defender,
                    self.defender.alive_soldiers(),
                    self.options.death_permille,
                ),
            },
            report: self.report,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct UnitRef {
    side: BattleSide,
    index: usize,
    speed: i32,
    id: i32,
}

fn winner_by_elimination(attacker: &BattleArmy, defender: &BattleArmy) -> Option<BattleWinner> {
    match (attacker.has_alive_units(), defender.has_alive_units()) {
        (true, false) => Some(BattleWinner::Attacker),
        (false, true) => Some(BattleWinner::Defender),
        (false, false) => Some(BattleWinner::Draw),
        (true, true) => None,
    }
}

fn winner_by_timeout(
    kind: BattleKind,
    attacker_soldiers: i64,
    defender_soldiers: i64,
) -> BattleWinner {
    match kind {
        BattleKind::Siege => BattleWinner::Defender,
        BattleKind::Field => match attacker_soldiers.cmp(&defender_soldiers) {
            std::cmp::Ordering::Greater => BattleWinner::Attacker,
            std::cmp::Ordering::Less => BattleWinner::Defender,
            std::cmp::Ordering::Equal => BattleWinner::Draw,
        },
    }
}

fn side_order(side: BattleSide) -> u8 {
    match side {
        BattleSide::Attacker => 0,
        BattleSide::Defender => 1,
    }
}
