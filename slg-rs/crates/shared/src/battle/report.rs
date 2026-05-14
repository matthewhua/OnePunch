use prost::Message;
use proto::slg::{
    BattleReport as ProtoBattleReport, BattleReportAction as ProtoBattleReportAction, TwoInt,
};

use super::{BattleSide, TroopAdvantage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleActionType {
    Attack,
    Skill,
    Heal,
}

impl BattleActionType {
    fn proto_code(self) -> i32 {
        match self {
            Self::Attack => 1,
            Self::Skill => 2,
            Self::Heal => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleReportAction {
    pub round: u32,
    pub action_index: u32,
    pub actor_side: BattleSide,
    pub actor_id: i32,
    pub target_side: BattleSide,
    pub target_id: i32,
    pub action_type: BattleActionType,
    pub skill_id: Option<i32>,
    pub damage: i64,
    pub healing: i64,
    pub remaining_soldiers: i64,
    pub advantage: TroopAdvantage,
    pub time_offset_ms: i32,
}

impl BattleReportAction {
    pub fn attack(
        round: u32,
        action_index: u32,
        actor_side: BattleSide,
        actor_id: i32,
        target_side: BattleSide,
        target_id: i32,
        damage: i64,
        remaining_soldiers: i64,
        advantage: TroopAdvantage,
    ) -> Self {
        Self {
            round,
            action_index,
            actor_side,
            actor_id,
            target_side,
            target_id,
            action_type: BattleActionType::Attack,
            skill_id: None,
            damage,
            healing: 0,
            remaining_soldiers,
            advantage,
            time_offset_ms: action_index.saturating_mul(500) as i32,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BattleReport {
    pub actions: Vec<BattleReportAction>,
}

impl BattleReport {
    pub fn push(&mut self, action: BattleReportAction) {
        self.actions.push(action);
    }

    pub fn to_proto(&self, key_id: impl Into<String>) -> ProtoBattleReport {
        ProtoBattleReport {
            key_id: key_id.into(),
            actions: self
                .actions
                .iter()
                .map(|action| ProtoBattleReportAction {
                    action_type: Some(action.action_type.proto_code()),
                    atk_hero_id: Some(action.actor_id),
                    dest_hero_ids: vec![action.target_id],
                    skill_id: action.skill_id,
                    buff_id: None,
                    effect: effect_pairs(action),
                    random_index: None,
                    time_offset: Some(action.time_offset_ms),
                })
                .collect(),
        }
    }

    pub fn encode_proto(&self, key_id: impl Into<String>) -> Result<Vec<u8>, prost::EncodeError> {
        let proto = self.to_proto(key_id);
        let mut bytes = Vec::with_capacity(proto.encoded_len());
        proto.encode(&mut bytes)?;
        Ok(bytes)
    }
}

fn effect_pairs(action: &BattleReportAction) -> Vec<TwoInt> {
    let mut pairs = Vec::new();
    if action.damage > 0 {
        pairs.push(TwoInt {
            v1: 1,
            v2: action.damage.min(i32::MAX as i64) as i32,
        });
    }
    if action.healing > 0 {
        pairs.push(TwoInt {
            v1: 2,
            v2: action.healing.min(i32::MAX as i64) as i32,
        });
    }
    pairs
}
