use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleSide {
    Attacker,
    Defender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleOutcome {
    AttackerWin,
    DefenderWin,
    Draw,
}

impl BattleOutcome {
    pub fn winner(self) -> Option<BattleSide> {
        match self {
            Self::AttackerWin => Some(BattleSide::Attacker),
            Self::DefenderWin => Some(BattleSide::Defender),
            Self::Draw => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleInput {
    pub battle_id: u64,
    pub attacker: Fighter,
    pub defender: Fighter,
    pub max_rounds: Option<u32>,
}

impl BattleInput {
    pub fn new(battle_id: u64, attacker: Fighter, defender: Fighter) -> Self {
        Self {
            battle_id,
            attacker,
            defender,
            max_rounds: None,
        }
    }

    pub fn with_max_rounds(mut self, max_rounds: u32) -> Self {
        self.max_rounds = Some(max_rounds);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fighter {
    pub fighter_id: u64,
    pub name: Option<String>,
    pub units: Vec<Unit>,
}

impl Fighter {
    pub fn new(fighter_id: u64, units: Vec<Unit>) -> Self {
        Self {
            fighter_id,
            name: None,
            units,
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn total_units(&self) -> u64 {
        self.units.iter().map(|unit| unit.count).sum()
    }

    pub fn total_power(&self) -> u64 {
        self.units
            .iter()
            .map(Unit::total_power)
            .fold(0_u64, u64::saturating_add)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub unit_id: u64,
    pub config_id: u32,
    pub level: u32,
    pub count: u64,
    pub attack: u32,
    pub defense: u32,
    pub hp: u32,
}

impl Unit {
    pub fn new(
        unit_id: u64,
        config_id: u32,
        count: u64,
        attack: u32,
        defense: u32,
        hp: u32,
    ) -> Self {
        Self {
            unit_id,
            config_id,
            level: 1,
            count,
            attack,
            defense,
            hp,
        }
    }

    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    pub fn power_per_unit(&self) -> u64 {
        unit_power(self.attack, self.defense, self.hp)
    }

    pub fn total_power(&self) -> u64 {
        self.count.saturating_mul(self.power_per_unit())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleRules {
    pub max_rounds: u32,
    pub minimum_damage: u64,
}

impl Default for BattleRules {
    fn default() -> Self {
        Self {
            max_rounds: 20,
            minimum_damage: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleResult {
    pub battle_id: u64,
    pub outcome: BattleOutcome,
    pub rounds: u32,
    pub attacker: FighterBattleStats,
    pub defender: FighterBattleStats,
    pub report: BattleReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FighterBattleStats {
    pub fighter_id: u64,
    pub side: BattleSide,
    pub initial_units: u64,
    pub remaining_units: u64,
    pub units_lost: u64,
    pub initial_power: u64,
    pub remaining_power: u64,
    pub power_lost: u64,
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub units: Vec<UnitBattleStats>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitBattleStats {
    pub unit_id: u64,
    pub config_id: u32,
    pub initial_count: u64,
    pub remaining_count: u64,
    pub units_lost: u64,
    pub initial_power: u64,
    pub remaining_power: u64,
    pub power_lost: u64,
    pub damage_dealt: u64,
    pub damage_taken: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleSummary {
    pub battle_id: u64,
    pub outcome: BattleOutcome,
    pub winner: Option<BattleSide>,
    pub rounds: u32,
    pub total_events: usize,
    pub attacker: FighterBattleSummary,
    pub defender: FighterBattleSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FighterBattleSummary {
    pub fighter_id: u64,
    pub side: BattleSide,
    pub initial_units: u64,
    pub remaining_units: u64,
    pub units_lost: u64,
    pub loss_rate_bps: u32,
    pub initial_power: u64,
    pub remaining_power: u64,
    pub power_lost: u64,
    pub damage_dealt: u64,
    pub damage_taken: u64,
}

impl FighterBattleStats {
    pub fn loss_rate_bps(&self) -> u32 {
        if self.initial_units == 0 {
            return 0;
        }

        ((self.units_lost.saturating_mul(10_000)) / self.initial_units).min(10_000) as u32
    }

    pub fn summary(&self) -> FighterBattleSummary {
        FighterBattleSummary {
            fighter_id: self.fighter_id,
            side: self.side,
            initial_units: self.initial_units,
            remaining_units: self.remaining_units,
            units_lost: self.units_lost,
            loss_rate_bps: self.loss_rate_bps(),
            initial_power: self.initial_power,
            remaining_power: self.remaining_power,
            power_lost: self.power_lost,
            damage_dealt: self.damage_dealt,
            damage_taken: self.damage_taken,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleReport {
    pub battle_id: u64,
    pub outcome: BattleOutcome,
    pub rounds: Vec<BattleRoundReport>,
    pub attacker_initial: Vec<UnitSnapshot>,
    pub defender_initial: Vec<UnitSnapshot>,
    pub attacker_remaining: Vec<UnitSnapshot>,
    pub defender_remaining: Vec<UnitSnapshot>,
}

impl BattleReport {
    pub fn total_events(&self) -> usize {
        self.rounds.iter().map(|round| round.events.len()).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleRoundReport {
    pub round: u32,
    pub events: Vec<BattleEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleEvent {
    pub round: u32,
    pub actor_side: BattleSide,
    pub actor_unit_id: u64,
    pub target_side: BattleSide,
    pub target_unit_id: u64,
    pub damage: u64,
    pub units_lost: u64,
    pub target_remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitSnapshot {
    pub unit_id: u64,
    pub config_id: u32,
    pub count: u64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BattleError {
    #[error("{side:?} fighter has no units")]
    EmptyFighter { side: BattleSide },
    #[error("{side:?} fighter has duplicate unit id {unit_id}")]
    DuplicateUnitId { side: BattleSide, unit_id: u64 },
    #[error("{side:?} unit {unit_id} has zero hp")]
    ZeroHp { side: BattleSide, unit_id: u64 },
    #[error("{side:?} unit {unit_id} has zero count")]
    ZeroCount { side: BattleSide, unit_id: u64 },
    #[error("battle has zero max rounds")]
    ZeroMaxRounds,
}

#[derive(Debug, Clone, Default)]
pub struct BattleEngine {
    rules: BattleRules,
}

impl BattleEngine {
    pub fn new(rules: BattleRules) -> Self {
        Self { rules }
    }

    pub fn resolve(&self, input: &BattleInput) -> Result<BattleResult, BattleError> {
        validate_input(input)?;

        let max_rounds = match input.max_rounds {
            Some(0) => return Err(BattleError::ZeroMaxRounds),
            Some(max_rounds) => max_rounds,
            None => self.rules.max_rounds,
        };
        if max_rounds == 0 {
            return Err(BattleError::ZeroMaxRounds);
        }

        let mut attacker = FighterState::from_fighter(BattleSide::Attacker, &input.attacker);
        let mut defender = FighterState::from_fighter(BattleSide::Defender, &input.defender);
        let attacker_initial = attacker.snapshots();
        let defender_initial = defender.snapshots();
        let mut rounds = Vec::new();
        let mut elapsed_rounds = 0;

        for round in 1..=max_rounds {
            if attacker.is_defeated() || defender.is_defeated() {
                break;
            }

            elapsed_rounds = round;
            let mut events = Vec::new();
            run_side_turn(
                round,
                &mut attacker,
                &mut defender,
                self.rules.minimum_damage,
                &mut events,
            );
            if !defender.is_defeated() {
                run_side_turn(
                    round,
                    &mut defender,
                    &mut attacker,
                    self.rules.minimum_damage,
                    &mut events,
                );
            }
            rounds.push(BattleRoundReport { round, events });
        }

        let outcome = match (attacker.is_defeated(), defender.is_defeated()) {
            (true, true) => BattleOutcome::Draw,
            (false, true) => BattleOutcome::AttackerWin,
            (true, false) => BattleOutcome::DefenderWin,
            (false, false) => BattleOutcome::Draw,
        };

        let attacker_remaining = attacker.snapshots();
        let defender_remaining = defender.snapshots();
        let attacker_stats = attacker.into_stats();
        let defender_stats = defender.into_stats();

        Ok(BattleResult {
            battle_id: input.battle_id,
            outcome,
            rounds: elapsed_rounds,
            attacker: attacker_stats,
            defender: defender_stats,
            report: BattleReport {
                battle_id: input.battle_id,
                outcome,
                rounds,
                attacker_initial,
                defender_initial,
                attacker_remaining,
                defender_remaining,
            },
        })
    }
}

impl BattleResult {
    pub fn winner_side(&self) -> Option<BattleSide> {
        self.outcome.winner()
    }

    pub fn total_events(&self) -> usize {
        self.report.total_events()
    }

    pub fn summary(&self) -> BattleSummary {
        BattleSummary {
            battle_id: self.battle_id,
            outcome: self.outcome,
            winner: self.winner_side(),
            rounds: self.rounds,
            total_events: self.total_events(),
            attacker: self.attacker.summary(),
            defender: self.defender.summary(),
        }
    }
}

pub fn resolve_battle(input: &BattleInput) -> Result<BattleResult, BattleError> {
    BattleEngine::default().resolve(input)
}

fn validate_input(input: &BattleInput) -> Result<(), BattleError> {
    validate_fighter(BattleSide::Attacker, &input.attacker)?;
    validate_fighter(BattleSide::Defender, &input.defender)?;
    Ok(())
}

fn validate_fighter(side: BattleSide, fighter: &Fighter) -> Result<(), BattleError> {
    if fighter.units.is_empty() {
        return Err(BattleError::EmptyFighter { side });
    }

    let mut unit_ids = HashSet::new();
    for unit in &fighter.units {
        if !unit_ids.insert(unit.unit_id) {
            return Err(BattleError::DuplicateUnitId {
                side,
                unit_id: unit.unit_id,
            });
        }
        if unit.count == 0 {
            return Err(BattleError::ZeroCount {
                side,
                unit_id: unit.unit_id,
            });
        }
        if unit.hp == 0 {
            return Err(BattleError::ZeroHp {
                side,
                unit_id: unit.unit_id,
            });
        }
    }

    Ok(())
}

fn run_side_turn(
    round: u32,
    actor: &mut FighterState,
    target: &mut FighterState,
    minimum_damage: u64,
    events: &mut Vec<BattleEvent>,
) {
    let actor_unit_count = actor.units.len();
    for actor_idx in 0..actor_unit_count {
        if target.is_defeated() {
            return;
        }
        if actor.units[actor_idx].remaining_count == 0 {
            continue;
        }

        let Some(target_idx) = target.first_alive_index() else {
            return;
        };

        let attack = actor.units[actor_idx].attack as u64;
        let defense = target.units[target_idx].defense as u64;
        let per_unit_damage = attack.saturating_sub(defense / 2).max(minimum_damage);
        let damage = per_unit_damage.saturating_mul(actor.units[actor_idx].remaining_count);
        let outcome = target.units[target_idx].apply_damage(damage);

        actor.units[actor_idx].damage_dealt = actor.units[actor_idx]
            .damage_dealt
            .saturating_add(outcome.damage_taken);
        target.units[target_idx].damage_taken = target.units[target_idx]
            .damage_taken
            .saturating_add(outcome.damage_taken);

        events.push(BattleEvent {
            round,
            actor_side: actor.side,
            actor_unit_id: actor.units[actor_idx].unit_id,
            target_side: target.side,
            target_unit_id: target.units[target_idx].unit_id,
            damage: outcome.damage_taken,
            units_lost: outcome.units_lost,
            target_remaining: target.units[target_idx].remaining_count,
        });
    }
}

#[derive(Debug, Clone)]
struct FighterState {
    fighter_id: u64,
    side: BattleSide,
    units: Vec<UnitState>,
}

impl FighterState {
    fn from_fighter(side: BattleSide, fighter: &Fighter) -> Self {
        Self {
            fighter_id: fighter.fighter_id,
            side,
            units: fighter.units.iter().map(UnitState::from_unit).collect(),
        }
    }

    fn first_alive_index(&self) -> Option<usize> {
        self.units.iter().position(|unit| unit.remaining_count > 0)
    }

    fn is_defeated(&self) -> bool {
        self.first_alive_index().is_none()
    }

    fn snapshots(&self) -> Vec<UnitSnapshot> {
        self.units
            .iter()
            .map(|unit| UnitSnapshot {
                unit_id: unit.unit_id,
                config_id: unit.config_id,
                count: unit.remaining_count,
            })
            .collect()
    }

    fn into_stats(self) -> FighterBattleStats {
        let units: Vec<UnitBattleStats> =
            self.units.into_iter().map(UnitState::into_stats).collect();
        let initial_units = units.iter().map(|unit| unit.initial_count).sum();
        let remaining_units = units.iter().map(|unit| unit.remaining_count).sum();
        let initial_power = units.iter().map(|unit| unit.initial_power).sum();
        let remaining_power = units.iter().map(|unit| unit.remaining_power).sum();
        let damage_dealt = units.iter().map(|unit| unit.damage_dealt).sum();
        let damage_taken = units.iter().map(|unit| unit.damage_taken).sum();

        FighterBattleStats {
            fighter_id: self.fighter_id,
            side: self.side,
            initial_units,
            remaining_units,
            units_lost: initial_units.saturating_sub(remaining_units),
            initial_power,
            remaining_power,
            power_lost: initial_power.saturating_sub(remaining_power),
            damage_dealt,
            damage_taken,
            units,
        }
    }
}

#[derive(Debug, Clone)]
struct UnitState {
    unit_id: u64,
    config_id: u32,
    initial_count: u64,
    remaining_count: u64,
    attack: u32,
    defense: u32,
    hp: u32,
    front_hp: u32,
    damage_dealt: u64,
    damage_taken: u64,
}

impl UnitState {
    fn from_unit(unit: &Unit) -> Self {
        Self {
            unit_id: unit.unit_id,
            config_id: unit.config_id,
            initial_count: unit.count,
            remaining_count: unit.count,
            attack: unit.attack,
            defense: unit.defense,
            hp: unit.hp,
            front_hp: unit.hp,
            damage_dealt: 0,
            damage_taken: 0,
        }
    }

    fn apply_damage(&mut self, damage: u64) -> DamageOutcome {
        if self.remaining_count == 0 || damage == 0 {
            return DamageOutcome {
                damage_taken: 0,
                units_lost: 0,
            };
        }

        let before = self.remaining_count;
        let remaining_damage = damage;
        let front_hp = self.front_hp as u64;
        let hp = self.hp as u64;
        let total_remaining_hp =
            front_hp.saturating_add(self.remaining_count.saturating_sub(1).saturating_mul(hp));
        let damage_taken = remaining_damage.min(total_remaining_hp);

        if damage_taken >= total_remaining_hp {
            self.remaining_count = 0;
            self.front_hp = self.hp;
        } else if damage_taken < front_hp {
            self.front_hp = (front_hp - damage_taken) as u32;
        } else {
            let after_front = damage_taken - front_hp;
            let full_units_lost = after_front / hp;
            let partial_damage = after_front % hp;
            self.remaining_count = self
                .remaining_count
                .saturating_sub(1)
                .saturating_sub(full_units_lost);
            self.front_hp = if partial_damage == 0 {
                self.hp
            } else {
                (hp - partial_damage) as u32
            };
        }

        DamageOutcome {
            damage_taken,
            units_lost: before.saturating_sub(self.remaining_count),
        }
    }

    fn into_stats(self) -> UnitBattleStats {
        let power_per_unit = unit_power(self.attack, self.defense, self.hp);
        let initial_power = self.initial_count.saturating_mul(power_per_unit);
        let remaining_power = self.remaining_count.saturating_mul(power_per_unit);

        UnitBattleStats {
            unit_id: self.unit_id,
            config_id: self.config_id,
            initial_count: self.initial_count,
            remaining_count: self.remaining_count,
            units_lost: self.initial_count.saturating_sub(self.remaining_count),
            initial_power,
            remaining_power,
            power_lost: initial_power.saturating_sub(remaining_power),
            damage_dealt: self.damage_dealt,
            damage_taken: self.damage_taken,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DamageOutcome {
    damage_taken: u64,
    units_lost: u64,
}

fn unit_power(attack: u32, defense: u32, hp: u32) -> u64 {
    u64::from(attack)
        .saturating_add(u64::from(defense))
        .saturating_add(u64::from(hp))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fighter(id: u64, units: Vec<Unit>) -> Fighter {
        Fighter::new(id, units)
    }

    #[test]
    fn fighter_and_unit_helpers_report_power() {
        let unit = Unit::new(10, 100, 3, 20, 5, 10).with_level(4);
        assert_eq!(unit.level, 4);
        assert_eq!(unit.power_per_unit(), 35);
        assert_eq!(unit.total_power(), 105);

        let fighter = fighter(1, vec![unit, Unit::new(11, 101, 2, 10, 3, 7)]).named("red");

        assert_eq!(fighter.name.as_deref(), Some("red"));
        assert_eq!(fighter.total_units(), 5);
        assert_eq!(fighter.total_power(), 145);
    }

    #[test]
    fn attacker_wins_and_report_tracks_losses() {
        let input = BattleInput::new(
            1001,
            fighter(1, vec![Unit::new(10, 100, 8, 20, 4, 10)]),
            fighter(2, vec![Unit::new(20, 200, 5, 8, 2, 10)]),
        );

        let result = resolve_battle(&input).expect("battle should resolve");

        assert_eq!(result.outcome, BattleOutcome::AttackerWin);
        assert_eq!(result.attacker.initial_units, 8);
        assert_eq!(result.defender.initial_units, 5);
        assert_eq!(result.defender.remaining_units, 0);
        assert!(result.attacker.damage_dealt >= result.defender.damage_taken);
        assert_eq!(result.report.outcome, BattleOutcome::AttackerWin);
        assert_eq!(result.winner_side(), Some(BattleSide::Attacker));
        assert_eq!(result.report.attacker_initial[0].count, 8);
        assert_eq!(result.report.defender_remaining[0].count, 0);
        assert!(!result.report.rounds.is_empty());

        let summary = result.summary();
        assert_eq!(summary.winner, Some(BattleSide::Attacker));
        assert_eq!(summary.total_events, result.report.total_events());
        assert_eq!(summary.defender.loss_rate_bps, 10_000);
        assert_eq!(summary.defender.remaining_power, 0);
        assert_eq!(
            summary.defender.power_lost,
            result
                .defender
                .initial_power
                .saturating_sub(result.defender.remaining_power)
        );
    }

    #[test]
    fn defender_wins_when_attacker_is_weaker() {
        let input = BattleInput::new(
            1002,
            fighter(1, vec![Unit::new(10, 100, 2, 6, 1, 8)]),
            fighter(2, vec![Unit::new(20, 200, 8, 16, 4, 10)]),
        );

        let result = resolve_battle(&input).expect("battle should resolve");

        assert_eq!(result.outcome, BattleOutcome::DefenderWin);
        assert_eq!(result.attacker.remaining_units, 0);
        assert!(result.defender.remaining_units > 0);
        assert_eq!(result.attacker.damage_taken, result.defender.damage_dealt);
    }

    #[test]
    fn unresolved_battle_draws_after_max_rounds() {
        let mut input = BattleInput::new(
            1003,
            fighter(1, vec![Unit::new(10, 100, 1, 1, 100, 1_000)]),
            fighter(2, vec![Unit::new(20, 200, 1, 1, 100, 1_000)]),
        );
        input.max_rounds = Some(1);

        let result = resolve_battle(&input).expect("battle should resolve");

        assert_eq!(result.outcome, BattleOutcome::Draw);
        assert_eq!(result.rounds, 1);
        assert_eq!(result.report.rounds.len(), 1);
        assert_eq!(result.attacker.remaining_units, 1);
        assert_eq!(result.defender.remaining_units, 1);
    }

    #[test]
    fn engine_rules_supply_default_max_rounds_when_input_does_not_override() {
        let input = BattleInput::new(
            1004,
            fighter(1, vec![Unit::new(10, 100, 1, 1, 100, 1_000)]),
            fighter(2, vec![Unit::new(20, 200, 1, 1, 100, 1_000)]),
        );
        let engine = BattleEngine::new(BattleRules {
            max_rounds: 1,
            minimum_damage: 1,
        });

        let result = engine.resolve(&input).expect("battle should resolve");

        assert_eq!(result.outcome, BattleOutcome::Draw);
        assert_eq!(result.rounds, 1);
    }

    #[test]
    fn input_max_rounds_override_engine_rules() {
        let input = BattleInput::new(
            1005,
            fighter(1, vec![Unit::new(10, 100, 1, 1, 100, 1_000)]),
            fighter(2, vec![Unit::new(20, 200, 1, 1, 100, 1_000)]),
        )
        .with_max_rounds(2);
        let engine = BattleEngine::new(BattleRules {
            max_rounds: 10,
            minimum_damage: 1,
        });

        let result = engine.resolve(&input).expect("battle should resolve");

        assert_eq!(result.outcome, BattleOutcome::Draw);
        assert_eq!(result.rounds, 2);
    }

    #[test]
    fn rejects_zero_input_max_rounds() {
        let input = BattleInput::new(
            1006,
            fighter(1, vec![Unit::new(10, 100, 1, 1, 1, 1)]),
            fighter(2, vec![Unit::new(20, 200, 1, 1, 1, 1)]),
        )
        .with_max_rounds(0);

        assert_eq!(resolve_battle(&input), Err(BattleError::ZeroMaxRounds));
    }

    #[test]
    fn rejects_invalid_unit_hp() {
        let input = BattleInput::new(
            1007,
            fighter(1, vec![Unit::new(10, 100, 1, 1, 1, 0)]),
            fighter(2, vec![Unit::new(20, 200, 1, 1, 1, 1)]),
        );

        assert_eq!(
            resolve_battle(&input),
            Err(BattleError::ZeroHp {
                side: BattleSide::Attacker,
                unit_id: 10,
            })
        );
    }

    #[test]
    fn rejects_zero_unit_count() {
        let input = BattleInput::new(
            1008,
            fighter(1, vec![Unit::new(10, 100, 0, 1, 1, 1)]),
            fighter(2, vec![Unit::new(20, 200, 1, 1, 1, 1)]),
        );

        assert_eq!(
            resolve_battle(&input),
            Err(BattleError::ZeroCount {
                side: BattleSide::Attacker,
                unit_id: 10,
            })
        );
    }

    #[test]
    fn rejects_duplicate_unit_ids_per_fighter() {
        let input = BattleInput::new(
            1009,
            fighter(
                1,
                vec![
                    Unit::new(10, 100, 1, 1, 1, 1),
                    Unit::new(10, 101, 1, 1, 1, 1),
                ],
            ),
            fighter(2, vec![Unit::new(20, 200, 1, 1, 1, 1)]),
        );

        assert_eq!(
            resolve_battle(&input),
            Err(BattleError::DuplicateUnitId {
                side: BattleSide::Attacker,
                unit_id: 10,
            })
        );
    }
}
