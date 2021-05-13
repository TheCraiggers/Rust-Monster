use crate::omni::character::Character;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Stat {
    pub(crate) display_name: String,
    pub(crate) display_on_tracker: bool,
    pub(crate) value: String,
    pub(crate) maximum_value: Option<String>,
}

impl Stat {
    pub fn name(&self) -> String {
        self.display_name.to_lowercase()
    }
}