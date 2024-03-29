//! This contains all the key command words the bot understands.
//! Adding words to this file automatically adds them to the help system and event listener.

use anyhow::{Result, anyhow};
use crate::discord::{DiscordReferences};

/// Respond to !help commands. If an argument is supplied, return detailed help for it, otherwise generic bot help is sent
pub async fn handle_help_command(discord_refs: &DiscordReferences<'_>, help_term: &str) -> Result<()> {
    if help_term.is_empty() {
        discord_refs.send_message_reply(&generate_generic_help_message()).await?;
        Ok(())
    } else {
        for words_array in ALL_WORDS.iter() {
            match words_array.iter().find(|&word| word.term == help_term) {
                Some(word) => { return Ok(discord_refs.dm_help_message(&word).await?)}
                None => { continue; }
            };
        }
        discord_refs.send_message_reply(format!("Could not find help for '{}'. Check your spelling.", help_term).as_str()).await?;
        return Err(anyhow!("Could not find help for term supplied"));
    }
}

enum WordType {
    Verb,
    Noun,
    Target
}

pub struct Word<'a> {
    kind: WordType,
    pub term: &'a str,
    pub short_help: &'a str,
    pub long_help: &'a str,
    pub usage_examples: &'a str,
}

impl Word<'_> {
    pub fn embed_title(&self) -> String {
        format!("Help for {}", self.term)
    }
}

fn generate_generic_help_message() -> String {
    let mut response: String = "Most commands take the form of `!verb noun target`, where target is usually the name of a character. The following words/commands are known to the bot. You can use `!help <word>` for more info about any of these:\n\n".to_string();
    let verb_list: Vec<String> = VERBS.iter().map(|word| word.term.to_string()).collect();
    let noun_list: Vec<String> = NOUNS.iter().map(|word| word.term.to_string()).collect();
    let target_list: Vec<String> = TARGETS.iter().map(|word| word.term.to_string()).collect();
    response.push_str(&format!("**Verbs:** {}\n", verb_list.join(", ")));
    response.push_str(&format!("**Nouns:** {}\n", noun_list.join(", ")));
    response.push_str(&format!("**Targets:** {}\n", target_list.join(", ")));

    response
}

const ALL_WORDS: [&[Word];3] = [&VERBS,&NOUNS,&TARGETS];

///////////////////////////////////////////////////////
// VERBS 
///////////////////////////////////////////////////////
pub const VERBS: [Word; 4] = [
    Word{
        term: "add",
        kind: WordType::Verb,
        short_help: "Add a new <noun>",
        long_help: "Use the add command to add a new <noun>, such as an enemy, or something like an effect to a player. Most nouns are supported, but consult the help pages for each for specifics about adding them.",
        usage_examples: "!add player Plunk\n!add enemy Slurk",
    },
    Word{
        term: "help",
        kind: WordType::Verb,
        short_help: "Get help on any bot command or term",
        long_help: "Use the help command to get detailed help about any command word the bot recognizes. Which you probably already knew, since you just typed `!help help`. Clever girl.",
        usage_examples: "!help roll\n!help effect\n!help lookup",
    },
    Word{
        term: "lookup",
        kind: WordType::Verb,
        short_help: "Get definitions of feats, spells, rules, etc",
        long_help: "The lookup command can look up the definitions of just about any Pathfinder thing there is, using the power of the Pathfinder 2 Easy Library. Feats, skills, spells, creatures, gods, you name it. If searching terns up more than one result, a list of options will be presented to you as reaction buttons to click. Simply click the correct button to select your choice.",
        usage_examples: "!lookup mage hand\n!lookup goblin dog\n!lookup cast a spell",
    },
    Word{
        term: "roll",
        kind: WordType::Verb,
        short_help: "Roll some dice",
        long_help: "The roll command can be used to roll discrete dice and/or a stored property on a character. If a character is not supplied, it defaults to the character owned by you. If you own multiple, and you're in combaat, it defaults to the active character.",
        usage_examples: "!roll 3d6+5\n!roll perception",
    },
];

///////////////////////////////////////////////////////
// Nouns 
///////////////////////////////////////////////////////
pub const NOUNS: [Word; 2] = [
    Word{
        term: "enemy",
        kind: WordType::Noun,
        short_help: "Enemies are characters whose stats are hidden from players",
        long_help: "Enemies are typically GM controlled characters and serve as things for a <player> for fight. They behave much like player characters in that they have stats, can roll dice, take damage, etc. Where they differ is that their stats are automatically hidden or obfuscated from those without the GM role.",
        usage_examples: "!remove enemy Goblin\n!add enemy Slurk",
    },
    Word{
        term: "stat",
        kind: WordType::Noun,
        short_help: "Information about a character like HP or attacks",
        long_help: "A stat can be almost anything. Use stats to remember your HP, level, focus points, or store complex dice rolls. A stat can either be static or dynamic.\n\nStatic stats are those with a value that only changes when you tell it to change, such as your level or hero points. When creating a static stat, simply give the name and the value seperated by a colon. If dice notation or references are included, they are resolved immediately and only the final result is stored.\n\nDynamic stats are a whole different beast. Their value is stored as a dice roll and can reference other stats. When you ask the bot to roll or otherwise return the value, it will *dynamically* compute it, rolling any dice and resolving any references needed. These are often used for things like attack rolls or saves. Dynamic stats are created like static, only prefix an equal sign before the value, like an Excel formula.\n\nStats can also be ranges with a maximum value, such as HP. To give a stat a maximum value, include a forward slash after the value, followed by the maximum. The maximum is only adjusted when the bot is asked to, so future set commands will only adjust the value unless the maximum is also given.",
        usage_examples: "!add stat Bob HP:35/35\n!set stat Bob HP:20\n!add stat Frank Reflex:=1d20+DEX\n!set stat Bob Level:5\n!roll stat Frank Reflex",
    }
];

///////////////////////////////////////////////////////
// TARGETS
///////////////////////////////////////////////////////
pub const TARGETS: [Word; 0] = [];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_header_sizes() {
        //! Discord prohibts embed titles from being larger than 256 chars
        for words_array in ALL_WORDS.iter() {
            for word in words_array.iter() {
                assert!(word.embed_title().chars().count() < 256)
            }
        }
    }
    #[test]
    fn embed_description_sizes() {
        //! Discord prohibts embed descriptions from being larger than 2048 chars
        for words_array in ALL_WORDS.iter() {
            for word in words_array.iter() {
                assert!(word.long_help.chars().count() < 2048)
            }
        }
    }

    #[test]
    fn generic_help_message_length() {
        //! Discord prohibts messages from being larger than 2000 chars
        assert!(generate_generic_help_message().chars().count() < 2000)
    }
}