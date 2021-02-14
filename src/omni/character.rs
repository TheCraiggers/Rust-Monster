mod effect;
use crate::omni::character::effect::Effect;
use twilight_model::id::UserId;

enum CharacterKind {
    Player,
    Npc
}

pub struct Character {
    kind: CharacterKind,
    name: String,
    owner: UserId,
    effects: Vec<Effect>,
}