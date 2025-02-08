use rand::Rng;

pub fn roll_dice<R: Rng>(rng: &mut R, sides: i32, modifier: i32) -> i32 {
    rng.random_range(1..=sides) + modifier
}
