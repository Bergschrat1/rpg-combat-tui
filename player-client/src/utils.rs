use std::fmt;

#[derive(Debug, PartialEq)]
pub enum HealthState {
    Healthy,
    Wounded,
    NearDeath,
    Dead,
}

impl fmt::Display for HealthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HealthState::Healthy => "Healthy",
            HealthState::Wounded => "Wounded",
            HealthState::NearDeath => "Near Death",
            HealthState::Dead => "Dead",
        };
        write!(f, "{}", s)
    }
}

pub fn get_health_state(max_hp: i32, current_hp: i32) -> HealthState {
    let ratio = current_hp as f32 / max_hp as f32;
    match ratio {
        p if p >= 0.6666 => HealthState::Healthy,
        p if p >= 0.3333 => HealthState::Wounded,
        p if p == 0.0 => HealthState::Dead,
        _ => HealthState::NearDeath,
    }
}
