use std::{fs, path::PathBuf};

use crate::combat::{
    dice::roll_dice,
    entity::{Entity, EntityType},
};
use rand::{rngs::StdRng, SeedableRng};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct CombatYaml {
    players: Vec<Entity>,
    monsters: Vec<MonsterEntry>,
}

#[derive(Debug, Deserialize)]
struct MonsterEntry {
    count: Option<usize>,
    stats: Entity,
}

#[derive(Debug)]
pub struct CombatTracker {
    entities: Vec<Entity>,
    current_turn: usize,
    round: usize,
    rng: StdRng,
}

impl CombatTracker {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            current_turn: 0,
            round: 0,
            rng: StdRng::from_rng(&mut rand::rng()),
        }
    }

    pub fn roll_initiative(&mut self, group_by_name: bool) {
        let mut initiative_map = std::collections::HashMap::new();
        self.entities.iter_mut().for_each(|entity| {
            let rolled_initiative = if group_by_name {
                *initiative_map
                    .entry(entity.name.clone())
                    .or_insert_with(|| roll_dice(&mut self.rng, 20, entity.initiative_modifier))
            } else {
                roll_dice(&mut self.rng, 20, entity.initiative_modifier)
            };
            entity.initiative = rolled_initiative;
        });
        self.sort_by_initiative();
    }

    pub fn add_entity(&mut self, mut new_entity: Entity) {
        let existing_count = self
            .entities
            .iter()
            .filter(|entity| &entity.name == &new_entity.name)
            .count();

        if existing_count > 0 {
            for entity in &mut self.entities {
                if entity.name == new_entity.name && entity.id == 0 {
                    entity.id = 1;
                }
            }
            new_entity.id = existing_count as i32 + 1;
        }

        new_entity.initiative = roll_dice(&mut self.rng, 20, new_entity.initiative_modifier);
        self.entities.push(new_entity);
    }

    pub fn remove_entity_by_uuid(&mut self, entity_id: Uuid) {
        self.entities.retain(|entity| entity.uuid != entity_id);

        // Ensure we don't go out of bounds in case the last entity was removed
        if self.current_turn >= self.entities.len() {
            self.current_turn = 0;
        }
    }

    pub fn next_turn(&mut self) {
        if !self.entities.is_empty() {
            self.current_turn = (self.current_turn + 1) % self.entities.len();
        }
        if self.current_turn == 0 {
            self.round += 1;
        }
    }

    pub fn get_current_entity(&self) -> Option<Entity> {
        self.entities
            .get(self.current_turn)
            .map(|entity| entity.clone())
    }

    fn sort_by_initiative(&mut self) {
        self.entities.sort_by(|a, b| {
            let initiative_cmp = b.initiative.cmp(&a.initiative);
            // prefer EntityType::Player in the case of a tie
            if initiative_cmp == std::cmp::Ordering::Equal {
                matches!(b.entity_type, EntityType::Player)
                    .cmp(&matches!(a.entity_type, EntityType::Player))
            } else {
                initiative_cmp
            }
        });
    }

    pub fn from_yaml(path: PathBuf) -> Self {
        let yaml_str = fs::read_to_string(path).expect("Failed to read YAML file");
        let combat_data: CombatYaml = serde_yml::from_str(&yaml_str).expect("Failed to parse YAML");

        let mut tracker = CombatTracker::new();
        for player in combat_data.players {
            tracker.add_entity(player);
        }
        for mut monster_entry in combat_data.monsters {
            let count = monster_entry.count.unwrap_or(1);
            for _ in 0..count {
                if monster_entry.stats.current_hp == 0 {
                    // set current_hp to max_hp if current_hp not set
                    monster_entry.stats = Entity {
                        current_hp: monster_entry.stats.max_hp,
                        ..monster_entry.stats
                    };
                }
                tracker.add_entity(monster_entry.stats.clone());
            }
        }

        tracker
    }
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, io::Write};

    use fs::File;
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_roll_initiative() {
        let mut ct = CombatTracker::new();
        let entity1 = Entity::new("monster1.1", EntityType::Monster, 0, 10, 20);
        let entity2 = Entity::new("monster1.2", EntityType::Monster, 50, 10, 20);
        let entity3 = Entity::new("monster2", EntityType::Monster, 100, 10, 20);
        let entity4 = entity3.clone();
        ct.add_entity(entity2);
        ct.add_entity(entity3);
        ct.add_entity(entity1);
        ct.add_entity(entity4);

        let rng = StdRng::seed_from_u64(42);
        ct.rng = rng;
        ct.roll_initiative(true);

        // check that same monsters are rolled together
        assert_eq!(&ct.entities[0].name, &ct.entities[1].name);
        assert_eq!(&ct.entities[0].initiative, &ct.entities[1].initiative);

        // check correct initiative order
        assert_eq!(&ct.get_current_entity().unwrap().name, "monster2");
        ct.next_turn();
        assert_eq!(&ct.get_current_entity().unwrap().name, "monster2");
        ct.next_turn();
        assert_eq!(&ct.get_current_entity().unwrap().name, "monster1.2");
        ct.next_turn();
        assert_eq!(&ct.get_current_entity().unwrap().name, "monster1.1");
    }

    #[test]
    fn test_individual_initiative_rolls() {
        let mut ct = CombatTracker::new();
        let entity1 = Entity::new("monster", EntityType::Monster, 0, 10, 20);
        let entity2 = Entity::new("monster", EntityType::Monster, 50, 10, 20);
        ct.add_entity(entity1);
        ct.add_entity(entity2);

        let rng = StdRng::seed_from_u64(42);
        ct.rng = rng;
        ct.roll_initiative(false);

        // Ensure monsters with the same name have different initiatives
        assert_ne!(ct.entities[0].initiative, ct.entities[1].initiative);
    }

    #[test]
    fn test_player_priority_in_sorting() {
        let mut ct = CombatTracker::new();
        let mut player = Entity::new("player", EntityType::Player, 10, 15, 30);
        player.initiative = 1;
        let mut monster = Entity::new("monster", EntityType::Monster, 10, 10, 20);
        monster.initiative = 1;
        ct.entities = vec![monster, player]; // monster before player
        ct.sort_by_initiative();

        // Ensure that the player is sorted before the monster with the same initiative
        assert_eq!(ct.entities[0].entity_type, EntityType::Player);
    }

    #[test]
    fn test_empty_combat_tracker() {
        let mut ct = CombatTracker::new();

        assert!(ct.get_current_entity().is_none());

        ct.next_turn(); // Should not panic
        assert!(ct.get_current_entity().is_none());
    }

    #[test]
    fn test_add_entity() {
        let mut ct = CombatTracker::new();
        let entity1 = Entity::new("monster", EntityType::Monster, 0, 10, 20);
        let entity2 = Entity::new("monster", EntityType::Monster, 50, 10, 20);
        let entity3 = Entity::new("monster", EntityType::Monster, 100, 10, 20);
        ct.add_entity(entity1);
        assert_eq!(ct.entities[0].id, 0);
        ct.add_entity(entity2);
        assert_eq!(ct.entities[0].id, 1);
        assert_eq!(ct.entities[1].id, 2);
        ct.add_entity(entity3);
        assert_eq!(ct.entities[0].id, 1);
        assert_eq!(ct.entities[1].id, 2);
        assert_eq!(ct.entities[2].id, 3);
    }

    #[test]
    fn test_remove_entry_by_uuid() {
        let mut ct = CombatTracker::new();
        let entity1 = Entity::new("monster", EntityType::Monster, 0, 10, 20);
        let entity2 = Entity::new("monster", EntityType::Monster, 50, 10, 20);
        ct.add_entity(entity1);
        ct.add_entity(entity2);
        assert_eq!(ct.entities.len(), 2);
    }

    #[test]
    fn test_from_yaml() {
        let yaml_content = "
players:
  - name: Arthas
    entity_type: Player
    initiative_modifier: 2
    ac: 18
    max_hp: 45
    current_hp: 45
    conditions: []

monsters:
  - count: 3
    stats:
        name: Goblin
        entity_type: Monster
        initiative_modifier: 1
        ac: 13
        max_hp: 15
        current_hp: 15
        conditions: []
  - stats:
        name: Orc
        entity_type: Monster
        initiative_modifier: 1
        ac: 13
        max_hp: 15
        conditions: [Blinded, Grappled]
        ";
        let dir = temp_dir();
        let file_path = dir.join("combat.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", yaml_content).unwrap();

        let tracker = CombatTracker::from_yaml(file_path);

        assert_eq!(tracker.entities.len(), 5);
        assert!(tracker.entities.iter().any(|e| e.name == "Arthas"));
        assert_eq!(
            tracker
                .entities
                .iter()
                .filter(|e| e.name == "Goblin")
                .count(),
            3
        );
        assert_eq!(tracker.entities.last().unwrap().current_hp, 15);
        assert_eq!(tracker.entities.last().unwrap().conditions.len(), 2);
    }
}
