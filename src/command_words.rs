//! This contains all the key command words the bot understands.
//! Adding words to this file automatically adds them to the help system and event listener.

enum WordType {
    Verb,
    Noun,
    Target
}

pub struct Word<'a> {
    kind: WordType,
    pub word: &'a str,
    short_help: &'a str,
    long_help: &'a str,
}

impl Word<'_> {
    pub fn help_embed(&self) {
        todo!();
    }
}

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