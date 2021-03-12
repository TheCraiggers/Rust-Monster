//! This contains all the key command words the bot understands.
//! Adding words to this file automatically adds them to the help system and event listener.

use anyhow::{Result, anyhow};
use crate::discord::{DiscordReferences};

pub async fn handle_help_command(discord_refs: &DiscordReferences<'_>, help_term: &str) -> Result<()> {
    for words_array in ALL_WORDS.iter() {
        match words_array.iter().find(|&word| word.word == help_term) {
            Some(word) => { return Ok(discord_refs.dm_help_message(&word).await?)}
            None => { continue; }
        };
    }
    discord_refs.send_message_reply(format!("Could not find help for '{}'. Check your spelling.", help_term).as_str()).await?;
    Err(anyhow!("Could not find help for '{}'. Check your spelling.", help_term))
}

enum WordType {
    Verb,
    Noun,
    Target
}

pub struct Word<'a> {
    kind: WordType,
    pub word: &'a str,
    pub short_help: &'a str,
    pub long_help: &'a str,
}

impl Word<'_> {
    pub fn embed_title(&self) -> String {
        format!("Help for {}", self.word)
    }
    pub fn help_embed(&self) {
        todo!();
    }
}

const ALL_WORDS: [&[Word];3] = [&VERBS,&NOUNS,&TARGETS];

///////////////////////////////////////////////////////
// VERBS 
///////////////////////////////////////////////////////
pub const VERBS: [Word; 2] = [
    Word{
        word: "help",
        kind: WordType::Verb,
        short_help: "Get help on any bot command or term",
        long_help: "Use the help command to get detailed help about any command word the bot recognizes. Which you probably already knew, since you just typed `!help help`. Clever girl.",
    },
    Word{
        word: "lookup",
        kind: WordType::Verb,
        short_help: "Get definitions of feats, spells, rules, etc",
        long_help: "The lookup command can look up the definitions of just about any Pathfinder thing there is, using the power of the Pathfinder 2 Easy Library. Feats, skills, spells, creatures, gods, you name it.",
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
}