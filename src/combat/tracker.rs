use crate::combat::{
    dice::roll_dice,
    entity::{Entity, EntityType},
};
use rand::{rngs::StdRng, SeedableRng};

#[derive(Debug)]
pub struct Initiative {
    pub initiative: i32,
    pub entity: Entity,
}

impl Initiative {
    pub fn new(initiative: i32, entity: Entity) -> Self {
        Self { initiative, entity }
    }
}

#[derive(Debug)]
pub struct CombatTracker {
    entities: Vec<Initiative>,
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
        self.entities.iter_mut().for_each(|initiative| {
            let rolled_initiative = if group_by_name {
                *initiative_map
                    .entry(initiative.entity.name.clone())
                    .or_insert_with(|| {
                        roll_dice(&mut self.rng, 20, initiative.entity.initiative_modifier)
                    })
            } else {
                roll_dice(&mut self.rng, 20, initiative.entity.initiative_modifier)
            };
            initiative.initiative = rolled_initiative;
        });
        self.sort_by_initiative();
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(Initiative::new(
            roll_dice(&mut self.rng, 20, entity.initiative_modifier),
            entity,
        ));
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
            .map(|initiative| initiative.entity.clone())
    }

    fn sort_by_initiative(&mut self) {
        self.entities.sort_by(|a, b| {
            let initiative_cmp = b.initiative.cmp(&a.initiative);
            if initiative_cmp == std::cmp::Ordering::Equal {
                matches!(b.entity.entity_type, EntityType::Player)
                    .cmp(&matches!(a.entity.entity_type, EntityType::Player))
            } else {
                initiative_cmp
            }
        });
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(&ct.entities[0].entity.name, &ct.entities[1].entity.name);
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
}
