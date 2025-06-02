use crate::combat::entity::{Condition, Entity, EntityType};
use crate::combat::tracker::CombatTracker;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerClientEntity {
    pub name: String,
    pub id: i32,
    pub current_hp: i32,
    pub max_hp: i32,
    pub conditions: Vec<Condition>,
    pub entity_type: EntityType,
    pub initiative: i32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PlayerClientState {
    pub round: usize,
    pub current_turn: usize,
    pub entities: Vec<PlayerClientEntity>,
}

impl From<&Entity> for PlayerClientEntity {
    fn from(e: &Entity) -> Self {
        Self {
            name: e.name.clone(),
            current_hp: e.current_hp,
            max_hp: e.max_hp,
            conditions: e.conditions.iter().cloned().collect(),
            id: e.id,
            entity_type: e.entity_type.clone(),
            initiative: e.initiative.expect("Initiative missing!"),
        }
    }
}

impl From<&CombatTracker> for PlayerClientState {
    fn from(t: &CombatTracker) -> Self {
        Self {
            round: t.round,
            current_turn: t.current_turn,
            entities: t.entities.iter().map(PlayerClientEntity::from).collect(),
        }
    }
}
