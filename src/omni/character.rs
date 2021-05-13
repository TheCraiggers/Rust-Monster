mod effect;
use crate::{discord::DiscordReferences};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::{pin::Pin, sync::Arc, u16, u64};
use anyhow::{Result, anyhow};
use pest::Parser;
use self::effect::Effect;
use self::stat::Stat;

use super::Omnidata;
mod stat;

#[derive(Parser)]
#[grammar = "character_commands.pest"]
pub struct CharacterCommandParser;

#[derive(Serialize, Deserialize, Debug)]
pub enum CharacterKind {
    Player,
    Npc
}

struct parsed_noun_target_stats_command {
    noun: String,
    target: String,
    stats: Vec<Stat>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub kind: CharacterKind,
    pub name: String,
    pub owner: u64,  //TODO: May need to find a better type for this
    pub effects: Vec<Effect>,
    pub stats: Vec<Stat>,
}

impl Character {
    /// Adds a new stat to the character's data.
    /// Returns a box containing a future to await.
    pub fn add_stat<'a, 'message:'a>(&mut self, discord_refs: &'a DiscordReferences<'message>, arguments: &str) -> Pin<Box<dyn Future<Output=Result<()>> + Send + 'a>> {
        let parsed_command = match parse_noun_target_stats_command(arguments) {
            Ok(parsed_command) => parsed_command,
            Err(reason) => return Box::pin(discord_refs.send_message_reply(reason.to_string())),
        };

        println!("Found these stats to add: {:?}", parsed_command.stats);
        for stat in parsed_command.stats {
            println!("Adding {:?} to stats.", stat);
            self.stats.push(stat);
        }

        Box::pin(discord_refs.send_message_reply(format!("Added new stat")))
    }
}

/// Given some arguments, will parse the command and return the noun, target, and stats
/// Should generally only be used for the longer verb+noun+target+props syntax
fn parse_noun_target_stats_command(arguments: &str) -> Result<parsed_noun_target_stats_command> {
    let parsed = CharacterCommandParser::parse(Rule::add_character, arguments);
    if parsed.is_err() {
        return Err(anyhow!("Failed to parse command. Remember the add command should follow the verb-noun-target syntax. For more help, consult `!help add`."));
    }
    let pairs = parsed.expect("Parse failed and wasn't caught!").next().unwrap().into_inner(); //Go into the command object
    
    let mut noun_pair = None;
    let mut target_pair = None;
    let mut stats: Vec<Stat> = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::noun => noun_pair = Some(pair),
            Rule::target => target_pair = Some(pair),
            Rule::stat => {
                let mut stat_name = None;
                let mut stat_value= None;
                let mut display_on_tracker = false;
                let mut maximum_value = None;
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::stat_name => stat_name = Some(inner_pair.as_str()),
                        Rule::stat_value => stat_value = Some(inner_pair.as_str()),
                        Rule::stat_maximum_value => maximum_value = Some(inner_pair.as_str().to_string()),
                        Rule::stat_always_display => display_on_tracker = true,
                        _ => {},
                    }
                }

                stats.push(Stat{ 
                    display_name: stat_name.unwrap().to_string(),
                    display_on_tracker: display_on_tracker,
                    value: stat_value.unwrap().to_string(),
                    maximum_value: maximum_value, 
                });
            }
            _ => {},
        }
    }

    if noun_pair == None {
        return Err(anyhow!("Failed to parse command; no noun found. Remember the add command should follow the verb-noun-target syntax. For more help, consult `!help add`."));
    }

    if target_pair == None {
        return Err(anyhow!("Failed to parse command; no target found. Remember the add command should follow the verb-noun-target syntax. For more help, consult `!help add`."));
    }

    Ok(parsed_noun_target_stats_command {
        noun: noun_pair.unwrap().as_str().to_string(),
        target: target_pair.unwrap().as_str().to_string(),
        stats: stats,
    })

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
        stats: Vec::new(),
    });
    omnidata.dirty();

    Box::pin(discord_refs.send_message_reply(format!("Added new charcter named {}", name)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nts_commands() {
        let mut parsed = parse_noun_target_stats_command("Player Plunk HP:40").unwrap();
        assert_eq!(parsed.noun, "Player");
        assert_eq!(parsed.target, "Plunk");
        assert_eq!(parsed.stats.len(), 1);
        assert_eq!(parsed.stats[0].display_on_tracker, false);
        assert_eq!(parsed.stats[0].name(), "hp");
        assert_eq!(parsed.stats[0].display_name, "HP");
        assert_eq!(parsed.stats[0].value, "40");

        parsed = parse_noun_target_stats_command("enemy Boss !HP:9000").unwrap();
        assert_eq!(parsed.noun, "enemy");
        assert_eq!(parsed.target, "Boss");
        assert_eq!(parsed.stats.len(), 1);
        assert_eq!(parsed.stats[0].display_on_tracker, true);
        assert_eq!(parsed.stats[0].name(), "hp");
        assert_eq!(parsed.stats[0].display_name, "HP");
        assert_eq!(parsed.stats[0].value, "9000");

        parsed = parse_noun_target_stats_command("stat Plunk !FP:2/3").unwrap();
        assert_eq!(parsed.noun, "stat");
        assert_eq!(parsed.target, "Plunk");
        assert_eq!(parsed.stats.len(), 1);
        assert_eq!(parsed.stats[0].display_on_tracker, true);
        assert_eq!(parsed.stats[0].name(), "fp");
        assert_eq!(parsed.stats[0].display_name, "FP");
        assert_eq!(parsed.stats[0].value, "2");
        assert_eq!(parsed.stats[0].maximum_value, Some(String::from("3")));
    }

}