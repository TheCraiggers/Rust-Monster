mod character;
use crate::omni::character::Character;
use serde::{Deserialize, Serialize};
use crate::discord::{*};
use anyhow::{Result};

#[derive(Serialize, Deserialize)]
pub struct Omnidata {
    pub version: u16,
    pub is_dirty: bool,
    pub characters:  Vec<Character>,
}



impl Omnidata {
 
}

pub async fn handle_command(discord_refs: &DiscordReferences<'_>) -> Result<()> {
    let omnidata = constructTracker(&discord_refs).await?;
    println!("{}", omnidata.version);
    return Ok(());
}