mod character;
use crate::{discord, omni::character::Character};
use serde::{Deserialize, Serialize};
use crate::discord::{DiscordReferences};
use anyhow::{Result, anyhow};
use std::{pin::Pin, sync::Arc, u16};
use futures::{Future, TryFutureExt, lock::Mutex};
use roll_rs::roll_inline;
use pest::Parser;
use crate::omni::character::add_character;

#[derive(Parser)]
#[grammar = "omni_commands.pest"]
pub struct OmniCommandParser;

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
}

/// Entry point for all bot commands that deal with the tracker data and characters
pub async fn handle_command(
    discord_refs: &DiscordReferences<'_>, 
    omnidata_cache: Arc<Mutex<Option<Omnidata>>>,
    command: &str,
    arguments: &str,
) -> Result<()> {

    // Lock the cached botdata. This should prevent any other commands from being run on this guild
    // If it doesn't exist, get the data from the guild and cache it
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
    let omnidata: &mut Omnidata = omnidata_guard.as_mut().unwrap();

    // Do whatever the user requested us to do. First we'll match by verb and let the following function handle the rest.
    let response = match command {
        "add" => Some(handle_add_command(discord_refs, omnidata, arguments)),
        "roll" => Some(handle_roll_command(discord_refs, omnidata, arguments)),
        _ => None
    };
    
    //Save the data and send the reply returned from the function that handled the command. These both happen at the same time to make things snappier.
    let reply_msg = response.unwrap().map_err(|e| anyhow!("Problem creating reply! {:?}", e.to_string()));
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

/// Given a string of arguments, this will parse and return the noun aka the first word.
/// Word, in this case, is the first thing surrounded by spaces, or a quoted string with
/// zero or more words and spaces inside. This will automatically strip any quotes.
fn get_noun(arguments: &str) -> Result<String> {
    let parsed = OmniCommandParser::parse(Rule::generic_command, arguments);
    println!("Parsing '{}'", arguments);
    match parsed {
        Ok(pairs) => Ok(pairs.peek().unwrap().into_inner().peek().unwrap().as_str().replace("\"", "")),
        Err(e) => Err(anyhow!("Couldn't parse command."))
    }
}

/// Handle all ADD commands, although mostly that just involves figuring out what should be added and calling the correct function.
fn handle_add_command<'a, 'message:'a>(discord_refs: &'a DiscordReferences<'message>, omnidata: &mut Omnidata, arguments: &str) -> Pin<Box<dyn Future<Output=Result<()>> + Send + 'a>> {
    let noun = get_noun(arguments);
    if noun.is_err() {
        return Box::pin(discord_refs.send_message_reply("Failed to parse command. Remember the add command should follow the verb-noun-target syntax. For more help, consult `!help add`."));
    }
    match get_noun(arguments).unwrap().as_str() {
        "player" | "enemy" => add_character(discord_refs, omnidata, arguments),
        unknown => return Box::pin(discord_refs.send_message_reply(format!("Sorry, I don't know how to add a '{}'. For more help, consult `!help add`.", unknown))),
    }
}

/// Handle simple roll commands. Arguments parameter should contain what to roll.
/// Return is a Future containing the message back to the user with the results.
fn handle_roll_command<'a, 'message:'a>(discord_refs: &'a DiscordReferences<'message>, omnidata: &Omnidata, arguments: &str) -> Pin<Box<dyn Future<Output=Result<()>> + Send + 'a>> {
    // TODO: Call a function that will resolve any named stats in the dice notation for the character being rolled, for example !roll reflex
    match roll_inline(arguments, false) {
        Ok(roll) => Box::pin(discord_refs.send_message_reply(format!("```\n{}```", roll.string_result))),
        Err(err) => Box::pin(discord_refs.send_message_reply(format!("```\n{}```", err)))
    }
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

    #[test]
    fn dice_string() {
        let roll = roll_inline("1d4", false).unwrap();
        assert!(roll.string_result.contains("="));
    }

    #[test]
    fn noun_parser() {
        assert_eq!(get_noun("player Plunk HP:30").unwrap(), "player");
        assert_eq!(get_noun("enemy \"War Boss\"").unwrap(), "War Boss");
    }
}