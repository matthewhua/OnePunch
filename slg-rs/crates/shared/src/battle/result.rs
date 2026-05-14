use super::BattleReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleWinner {
    Attacker,
    Defender,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleEndReason {
    Eliminated,
    MaxRounds,
    NoUnits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleSideLoss {
    pub initial: i64,
    pub remaining: i64,
    pub lost: i64,
    pub dead: i64,
    pub wounded: i64,
}

impl BattleSideLoss {
    pub fn new(initial: i64, remaining: i64, death_permille: i32) -> Self {
        let lost = (initial - remaining).max(0);
        let dead = lost * death_permille.clamp(0, 1000) as i64 / 1000;
        Self {
            initial,
            remaining,
            lost,
            dead,
            wounded: lost - dead,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleLossSummary {
    pub attacker: BattleSideLoss,
    pub defender: BattleSideLoss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOutcome {
    pub winner: BattleWinner,
    pub reason: BattleEndReason,
    pub rounds: u32,
    pub losses: BattleLossSummary,
    pub report: BattleReport,
}
