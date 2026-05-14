pub const DEFAULT_MAX_ROUNDS: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BattleSide {
    Attacker,
    Defender,
}

impl BattleSide {
    pub fn opponent(self) -> Self {
        match self {
            Self::Attacker => Self::Defender,
            Self::Defender => Self::Attacker,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TroopKind {
    Infantry,
    Cavalry,
    Archer,
    Siege,
}

impl TroopKind {
    pub fn advantage_against(self, defender: Self) -> TroopAdvantage {
        match (self, defender) {
            (Self::Infantry, Self::Cavalry)
            | (Self::Cavalry, Self::Archer)
            | (Self::Archer, Self::Infantry) => TroopAdvantage::Advantaged,
            (Self::Cavalry, Self::Infantry)
            | (Self::Archer, Self::Cavalry)
            | (Self::Infantry, Self::Archer) => TroopAdvantage::Disadvantaged,
            _ => TroopAdvantage::Neutral,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TroopAdvantage {
    Advantaged,
    Disadvantaged,
    Neutral,
}

impl TroopAdvantage {
    pub fn damage_permille(self) -> i32 {
        match self {
            Self::Advantaged => 1300,
            Self::Disadvantaged => 700,
            Self::Neutral => 1000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BattleKind {
    Field,
    Siege,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BattleVariance {
    Fixed(i32),
    Seeded(u64),
}

impl BattleVariance {
    pub fn permille(self, round: u32, action_index: u32, actor_id: i32, target_id: i32) -> i32 {
        match self {
            Self::Fixed(value) => value.clamp(950, 1050),
            Self::Seeded(seed) => {
                let mut value = seed
                    ^ ((round as u64) << 48)
                    ^ ((action_index as u64) << 32)
                    ^ ((actor_id as u64) << 16)
                    ^ target_id as u64;
                value = splitmix64(value);
                950 + (value % 101) as i32
            }
        }
    }
}

impl Default for BattleVariance {
    fn default() -> Self {
        Self::Seeded(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleUnit {
    pub id: i32,
    pub owner_id: String,
    pub hero_id: i32,
    pub side: BattleSide,
    pub troop_kind: TroopKind,
    pub soldiers: i64,
    pub attack: i64,
    pub defense: i64,
    pub speed: i32,
    pub skills: Vec<crate::battle::BattleSkill>,
}

impl BattleUnit {
    pub fn is_alive(&self) -> bool {
        self.soldiers > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleArmy {
    pub id: String,
    pub units: Vec<BattleUnit>,
}

impl BattleArmy {
    pub fn new(id: impl Into<String>, units: Vec<BattleUnit>) -> Self {
        Self {
            id: id.into(),
            units,
        }
    }

    pub fn alive_soldiers(&self) -> i64 {
        self.units
            .iter()
            .filter(|unit| unit.is_alive())
            .map(|unit| unit.soldiers)
            .sum()
    }

    pub fn initial_soldiers(&self) -> i64 {
        self.units.iter().map(|unit| unit.soldiers.max(0)).sum()
    }

    pub fn has_alive_units(&self) -> bool {
        self.units.iter().any(BattleUnit::is_alive)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOptions {
    pub max_rounds: u32,
    pub battle_kind: BattleKind,
    pub variance: BattleVariance,
    pub death_permille: i32,
}

impl BattleOptions {
    pub fn normalized(mut self) -> Self {
        if self.max_rounds == 0 {
            self.max_rounds = DEFAULT_MAX_ROUNDS;
        }
        self.death_permille = self.death_permille.clamp(0, 1000);
        self
    }
}

impl Default for BattleOptions {
    fn default() -> Self {
        Self {
            max_rounds: DEFAULT_MAX_ROUNDS,
            battle_kind: BattleKind::Field,
            variance: BattleVariance::default(),
            death_permille: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleRequest {
    pub attacker: BattleArmy,
    pub defender: BattleArmy,
    pub options: BattleOptions,
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
