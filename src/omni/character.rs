mod effect;
use crate::omni::character::effect::Effect;
use twilight_model::id::UserId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum CharacterKind {
    Player,
    Npc
}

#[derive(Serialize, Deserialize)]
pub struct Character {
    kind: CharacterKind,
    name: String,
    owner: UserId,
    effects: Vec<Effect>,
}