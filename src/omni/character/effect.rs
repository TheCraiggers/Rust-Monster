use time::Duration;
use crate::omni::character::Character;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Effect {
    name: String,
    duration: Duration,
    owner: Character,
    target: Character,
}