#![allow(dead_code)]
use super::{dice::roll_dice, entity};
use crate::combat::entity::Entity;

#[derive(Debug)]
pub struct InitiativeGroup {
    pub initiative: i32,
    pub entities: Vec<Entity>,
}

impl InitiativeGroup {
    pub fn new(initiative: i32, entity: Entity) -> Self {
        Self {
            initiative,
            entities: vec![entity],
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}

#[derive(Debug)]
pub struct CombatTracker {
    initiative_groups: Vec<InitiativeGroup>,
    current_turn: usize,
    round: usize,
}

impl CombatTracker {
    pub fn new() -> Self {
        Self {
            initiative_groups: Vec::new(),
            current_turn: 0,
            round: 0,
        }
    }

    pub fn roll_initiative(&mut self) {
        let mut new_groups: Vec<InitiativeGroup> = Vec::new();
        for entity in self
            .initiative_groups
            .iter_mut()
            .flat_map(|g| g.entities.iter_mut())
        {
            let initiative = roll_dice(20, entity.initiative_modifier);
            if let Some(group) = new_groups.iter_mut().find(|g| g.initiative == initiative) {
                group.add_entity(entity.clone());
            } else {
                new_groups.push(InitiativeGroup::new(initiative, entity.clone()));
            }
        }
        self.initiative_groups = new_groups;
        self.sort_by_initiative();
    }

    pub fn add_entity(&mut self, entity: Entity) {
        let initiative = roll_dice(20, entity.initiative_modifier);
        if let Some(group) = self
            .initiative_groups
            .iter_mut()
            .find(|g| g.initiative == initiative)
        {
            group.add_entity(entity);
        } else {
            self.initiative_groups
                .push(InitiativeGroup::new(initiative, entity));
            self.sort_by_initiative();
        }
    }

    pub fn remove_entity(&mut self, name: &str) {
        self.initiative_groups.retain(|group| {
            group.entities.retain(|e| e.name != name);
            !group.entities.is_empty()
        });
    }

    pub fn next_turn(&mut self) {
        if !self.initiative_groups.is_empty() {
            self.current_turn = (self.current_turn + 1) % self.initiative_groups.len();
        }
        if self.current_turn == 0 {
            self.round += 1;
        }
    }

    pub fn get_current_entities(&self) -> Option<&Vec<Entity>> {
        self.initiative_groups
            .get(self.current_turn)
            .map(|g| &g.entities)
    }

    fn sort_by_initiative(&mut self) {
        self.initiative_groups
            .sort_by(|a, b| b.initiative.cmp(&a.initiative));
    }
}
