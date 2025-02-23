#![allow(dead_code)]
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};
use uuid::Uuid;

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

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    Npc,
    Monster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    #[serde(default)]
    pub id: i32, // for multiple Monsters of the same type
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,
    pub name: String,
    pub entity_type: EntityType,
    pub initiative: Option<i32>,
    pub initiative_modifier: i32,
    pub ac: i32,
    pub max_hp: i32,
    #[serde(default)]
    pub current_hp: i32,
    pub conditions: HashSet<Condition>,
}

impl Entity {
    pub fn new(
        name: &str,
        entity_type: EntityType,
        initiative_modifier: i32,
        ac: i32,
        max_hp: i32,
    ) -> Self {
        Self {
            id: 0,
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            entity_type,
            initiative: None,
            initiative_modifier,
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

    pub fn ref_array_string(&self) -> Vec<String> {
        // TODO this funciton should not be the responsibility of the Entity
        let display_name = if self.id != 0 {
            format!("{} ({})", self.name, self.id)
        } else {
            self.name.to_string()
        };
        vec![
            self.initiative
                .expect("Initiative needs to be rolled.")
                .to_string(),
            display_name,
            format!(
                "{}/{}",
                self.current_hp.to_string(),
                self.max_hp.to_string()
            ),
            self.ac.to_string(),
            self.conditions.iter().join(", "),
        ]
    }
}
