mod character;
use crate::omni::character::Character;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Omnidata {
    version: u16,
    is_dirty: bool,
    characters:  Vec<Character>,
}

pub fn constructTrackerFromMessage(message: String) -> Omnidata {
    return Omnidata {version: 0, characters: Vec::new(), is_dirty: false};
}

impl Omnidata {
 
}