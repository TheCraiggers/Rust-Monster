mod effect;
use crate::{discord::DiscordReferences, omni::character::effect::Effect};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::{pin::Pin, sync::Arc, u16, u64};
use anyhow::{Result, anyhow};
use pest::Parser;
use super::Omnidata;

#[derive(Parser)]
#[grammar = "character_commands.pest"]
pub struct CharacterCommandParser;

#[derive(Serialize, Deserialize, Debug)]
pub enum CharacterKind {
    Player,
    Npc
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub kind: CharacterKind,
    pub name: String,
    pub owner: u64,  //TODO: May need to find a better type for this
    pub effects: Vec<Effect>,
}

impl Character {
    
}

/// Adds a new player or NPC to the server's data. Basically, anything that can have stats like HP is a character.
/// Returns a box containing a future to await.
pub fn add_character<'a, 'message:'a>(discord_refs: &'a DiscordReferences<'message>, omnidata: &mut Omnidata, arguments: &str) -> Pin<Box<dyn Future<Output=Result<()>> + Send + 'a>> {
    let parsed = CharacterCommandParser::parse(Rule::add_character, arguments);
    if parsed.is_err() {
        return Box::pin(discord_refs.send_message_reply("Failed to parse command. Remember the add command should follow the verb-noun-target syntax. For more help, consult `!help add`."));
    }
    let mut pairs = parsed.expect("Parse failed and wasn't caught!").next().unwrap().into_inner(); //Go into the command object
    
    // First pair should be the noun.
    let noun_pair = pairs.next().unwrap();
    if noun_pair.as_rule() != Rule::noun {
        return Box::pin(discord_refs.send_message_reply(format!("Failed to parse command. Expected a Noun, got something unknown")));
    }
    let kind = match noun_pair.as_str() {
        "player" => CharacterKind::Player,
        "enemy" => CharacterKind::Npc,
        unknown => return Box::pin(discord_refs.send_message_reply(format!("Failed to parse command. Unknown noun of '{}'", unknown))),
    };
    
    // Second pair should be the character name.
    let name_pair = pairs.next().unwrap();
    if name_pair.as_rule() != Rule::target {
        return Box::pin(discord_refs.send_message_reply(format!("Failed to parse command. Expected a Target, got something unknown")));
    }
    let name = name_pair.as_str();

    // Everything else should be stats to add to the character. This will be done after the character is added.
    // TODO: Parse and add props to new char

    omnidata.characters.push(Character {
        name: name.to_string(),
        kind: kind,
        owner: discord_refs.msg.author.id.0,
        effects: Vec::new(),
    });
    omnidata.dirty();

    Box::pin(discord_refs.send_message_reply(format!("Added new charcter named {}", name)))
}