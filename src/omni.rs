mod character;
use crate::{discord, omni::character::Character};
use serde::{Deserialize, Serialize};
use crate::discord::{DiscordReferences, *};
use anyhow::{Result};
use std::sync::Arc;
use futures::{executor::block_on, lock::Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub struct Omnidata {
    pub version: u16,
    pub is_dirty: bool,
    pub characters: Vec<Character>,
}

impl Omnidata {
    
}

pub fn create_empty_omnidata() -> Omnidata {
    return Omnidata {version: 0, characters: Vec::new(), is_dirty: false};
}

pub async fn handle_command(discord_refs: &DiscordReferences<'_>, omnidata_cache: Arc<Mutex<Omnidata>>) -> Result<()> {
    //let mut omnidata = construct_tracker(&discord_refs).await?;
    let mut omnidata = omnidata_cache.lock().await;
    //println!("Before: {:#?}", omnidata);
    omnidata.characters.push(character::Character{kind: character::CharacterKind::Player, name: "foo".to_string(), owner: "test".to_string(), effects: Vec::new()});
    //println!("After: {:#?}", omnidata);
    let response1 = discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).content(format!("Saving."))?;
    let save = discord::omni_data_save(&discord_refs, &omnidata);
    let response2 = discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).content(format!("Done."))?;
    futures::join!(response1, save, response2);
    return Ok(());
}