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
pub const VERBS: [Word; 2] = [
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
];

///////////////////////////////////////////////////////
// Nouns 
///////////////////////////////////////////////////////
pub const NOUNS: [Word; 0] = [];

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