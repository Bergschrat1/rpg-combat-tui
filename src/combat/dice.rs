use rand;

pub fn roll_dice(sides: i32, modifier: i32) -> i32 {
    rand::random_range(0..=sides) + modifier
}
