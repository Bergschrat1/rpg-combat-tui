use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    Blinded,
    Charmed,
    Deafened,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    Npc,
    Monster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub initiative: i32,
    pub ac: i32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub conditions: HashSet<Condition>,
}

impl Entity {
    pub fn new(name: &str, entity_type: EntityType, initiative: i32, ac: i32, max_hp: i32) -> Self {
        Self {
            name: name.to_string(),
            entity_type,
            initiative,
            ac,
            max_hp,
            current_hp: max_hp,
            conditions: HashSet::new(),
        }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.current_hp = (self.current_hp - damage).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.insert(condition);
    }

    pub fn remove_condition(&mut self, condition: &Condition) {
        self.conditions.remove(condition);
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }
}
