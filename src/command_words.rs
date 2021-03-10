//! This contains all the key command words the bot understands.
//! Adding words to this file automatically adds them to the help system and event listener.

enum WordType {
    Verb,
    Noun,
    Target
}

struct Word<'a> {
    kind: WordType,
    word: &'a str,
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
const Verbs: [Word; 1] = [
    Word{
        word: "help",
        kind: WordType::Verb,
        short_help: "Get help on any bot command or term",
        long_help: "Use the help command to get detailed help about any command word the bot recognizes. Which you probably already knew, since you just typed `!help help`. Clever girl.",
    },
];