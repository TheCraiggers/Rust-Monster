mod character;
use crate::{discord, omni::character::Character};
use serde::{Deserialize, Serialize};
use crate::discord::{DiscordReferences};
use anyhow::{Result};
use std::{sync::Arc, u16};
use futures::{lock::Mutex};

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
    omnidata_cache: Arc<Mutex<Omnidata>>,
    arguments: &str,
) -> Result<()> {
    //let mut omnidata = construct_tracker(&discord_refs).await?;
    let mut omnidata = omnidata_cache.lock().await;
    //println!("Before: {:#?}", omnidata);
    omnidata.add_character("me");
    //println!("After: {:#?}", omnidata);
    let reply_msg = discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).content(format!("This is your reply for {}", arguments))?;
    let save = discord::omni_data_save(&discord_refs, &omnidata);
    futures::join!(reply_msg, save);
    println!("Actually done saving.");
    return Ok(());
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