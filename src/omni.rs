mod character;
use crate::omni::character::Character;

struct Tracker {
    version: u16,
    characters:  Vec<Character>,
}

impl Tracker {
    pub fn constructTrackerFromMessage(message: String) -> Tracker {
        return Tracker {version: 0, characters: Vec::new()};

    }
}