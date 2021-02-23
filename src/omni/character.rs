mod effect;
use crate::omni::character::effect::Effect;
use twilight_model::id::UserId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CharacterKind {
    Player,
    Npc
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub kind: CharacterKind,
    pub name: String,
    pub owner: String,
    pub effects: Vec<Effect>,
}