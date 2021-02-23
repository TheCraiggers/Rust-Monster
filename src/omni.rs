mod character;
use crate::{discord, omni::character::Character};
use serde::{Deserialize, Serialize};
use crate::discord::{DiscordReferences, *};
use anyhow::{Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct Omnidata {
    pub version: u16,
    pub is_dirty: bool,
    pub characters:  Vec<Character>,
}

impl Omnidata {
    async fn save(self, discord_references: &DiscordReferences<'_>) -> Result<()> {
        discord::omni_data_save(&discord_references, self).await
    }
}

pub fn create_empty_omnidata() -> Omnidata {
    return Omnidata {version: 0, characters: Vec::new(), is_dirty: false};
}

pub async fn handle_command(discord_refs: &DiscordReferences<'_>) -> Result<()> {
    let mut omnidata = construct_tracker(&discord_refs).await?;
    println!("Before: {:#?}", omnidata);
    omnidata.characters.push(character::Character{kind: character::CharacterKind::Player, name: "foo".to_string(), owner: "test".to_string(), effects: Vec::new()});
    println!("After: {:#?}", omnidata);
    omnidata.save(&discord_refs).await?;

    return Ok(());
}