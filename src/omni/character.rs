mod effect;
use crate::omni::character::effect::Effect;
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
    pub owner: String,  //TODO: May need to find a better type for this
    pub effects: Vec<Effect>,
}

impl Character {
    
}