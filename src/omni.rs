mod character;
use crate::omni::character::Character;

struct Tracker {
    version: i16,
    characters: Vec<Character>,
}