#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleSkill {
    pub id: i32,
    pub trigger: SkillTrigger,
    pub effect: SkillEffect,
}

impl BattleSkill {
    pub fn is_active_on_round(&self, round: u32) -> bool {
        match self.trigger {
            SkillTrigger::ActiveEvery { rounds } => rounds > 0 && round % rounds == 0,
            SkillTrigger::PassiveAlways => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillTrigger {
    ActiveEvery { rounds: u32 },
    PassiveAlways,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillEffect {
    DamageBonus { bonus_permille: i32 },
    Heal { heal_permille: i32 },
}

pub fn active_skill_for_round(skills: &[BattleSkill], round: u32) -> Option<&BattleSkill> {
    skills.iter().find(|skill| skill.is_active_on_round(round))
}

pub fn passive_damage_bonus_permille(skills: &[BattleSkill]) -> i32 {
    skills
        .iter()
        .filter_map(|skill| match (skill.trigger, skill.effect) {
            (SkillTrigger::PassiveAlways, SkillEffect::DamageBonus { bonus_permille }) => {
                Some(bonus_permille)
            }
            _ => None,
        })
        .sum()
}
