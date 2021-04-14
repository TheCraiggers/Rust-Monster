mod character;
use crate::{discord, omni::character::Character};
use serde::{Deserialize, Serialize};
use crate::discord::{DiscordReferences};
use anyhow::{Result, anyhow};
use std::{sync::Arc, u16};
use futures::{TryFutureExt, lock::Mutex};
use roll_rs::roll_inline;

const OMNI_VERSION: u16 = 0;

#[derive(Serialize, Deserialize, Debug)]
pub struct Omnidata {
    pub version: u16,
    pub is_dirty: bool,
    pub characters: Vec<Character>,
}

impl Omnidata {
    pub fn new() -> Self {
        Omnidata {
            version: OMNI_VERSION, 
            characters: Vec::new(), 
            is_dirty: false,
        }
    }

    fn dirty(&mut self) {
        self.is_dirty = true;
    }

    fn add_character(&mut self, name: &str) {
        self.characters.push(Character {
            name: name.to_string(),
            kind: character::CharacterKind::Npc,
            owner: "foo".to_string(),
            effects: Vec::new(),
        });
        self.dirty();        
    }
}

pub async fn handle_command(
    discord_refs: &DiscordReferences<'_>, 
    omnidata_cache: Arc<Mutex<Option<Omnidata>>>,
    command: &str,
    arguments: &str,
) -> Result<()> {
    
    // Lock the cached botdata. This should prevent any othe commands from being run on this guild
    // If it doesn't exist, get the data from the guild and cache itfs.msg.guild_id);
    let mut omnidata_guard = omnidata_cache.lock().await;
    if omnidata_guard.is_none() {
        *omnidata_guard = match discord::get_tracker(&discord_refs).await {
            Ok(v) => Some(v),
            Err(e) => {
                println!("Error setting up bot. {:?}", e);
                discord_refs.send_message("Could not setup bot. Does it have Manage Channel permissions?").await?;
                return Err(anyhow!("Could not set up bot channel."));
            }
        }
    }
    let omnidata = omnidata_guard.as_mut().unwrap();

    // Do whatever the user requested us to do
    match command {
        "roll" => handle_roll_command(discord_refs, omnidata, arguments).await,
        _ => { println!("Somehow an invalid command was passed to omni::handle_command") }
    }
    
    //Save the data
    let reply_msg = discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).content(format!("This is your reply for {}", arguments))?.map_err(|e| anyhow!("Problem creating reply! {:?}", e.to_string()));
    let save = discord::omni_data_save(&discord_refs, &omnidata);
    match futures::try_join!(reply_msg, save) {
        Ok((_,_)) => {
            println!("Actually done saving.");
            return Ok(());
        },
        Err(e) => {
            println!("Save failed with error: {:?}", e.to_string());
            return Err(anyhow!("Save failed with error: {:?}", e.to_string()));
        }
    }
}

async fn handle_roll_command(discord_refs: &DiscordReferences<'_>, omnidata: &Omnidata, arguments: &str) {
    let roll = roll_inline(arguments, true).unwrap();
    discord_refs.send_message_reply(&format!("{}", &roll.string_result)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_omnidata() {
        let omnidata = Omnidata::new();
        assert_eq!(omnidata.version, OMNI_VERSION);
    }
    
    #[test]
    fn dirty_omnidata() {
        let mut omnidata = Omnidata::new();
        assert_eq!(omnidata.is_dirty, false);
        omnidata.dirty();
        assert_eq!(omnidata.is_dirty, true);
    }
}