use time::Duration;
use crate::omni::character::Character;

pub struct Effect {
    name: String,
    duration: Duration,
    owner: Character,
    target: Character,
}