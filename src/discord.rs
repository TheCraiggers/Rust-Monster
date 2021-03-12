//! At the time of writing this, both Discord libs for Rust are quite new and haven't reached 1.0 status yet.
//! Also, as the communities are still quite small, there are risks about maintainability as well.
//! Because of this, it was decided to create a layer between the "bot code" and the discord library.
//! This way, if the library ever needed to be switched, or if a breaking change was introduced, we could simply
//! update the code here and all of the calling functions would be ignorant.

use omni::{Omnidata};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::{ChannelType::GuildCategory, GuildChannel}, gateway::{payload::MessageCreate}, guild::{Emoji}, id::MessageId};
use anyhow::{Context, Result, anyhow};
use crate::{command_words::Word, omni};
use reqwest;
use futures;
use core::mem::size_of_val;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};

pub const BOT_DATA_CHANNEL_CATEGORY_NAME: &str = "rust-monster-bot-data";
pub const BOT_DATA_CHANNEL_NAME: &str = "omni-bot-data";

//Vec Mutex to hold all the Boxed Mutexes holding the trackers. Look for a dict so I can reference by guild id.


/// The standard amount of info that all discord functions take.
pub struct DiscordReferences<'a> {
    pub http: &'a HttpClient,
    pub msg: &'a Box<MessageCreate>,
}

impl DiscordReferences<'_> {
    /// Sends a text message to the same guild/channel
    pub async fn send_message(&self, text: &str) -> Result<()>{
        match self.http.create_message(self.msg.channel_id).content(text)?.await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e.to_string()))
        }
    }

    /// Sends a text message to the same guild/channel, but also makes it a reply to the original sender
    pub async fn send_message_reply(&self, text: &str) -> Result<()>{
        match self.http.create_message(self.msg.channel_id).reply(self.msg.id).content(text)?.await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e.to_string()))
        }
    }

    /// Sends a private DM to the user containing help about a bot command or keyword
    pub async fn dm_help_message(&self, help_word: &Word<'_>) -> Result<()> {
        let embed = EmbedBuilder::new()
            .description(help_word.embed_title())?
            .field(EmbedFieldBuilder::new("Description", help_word.long_help)?)
            .field(EmbedFieldBuilder::new("Usage examples", "examples would go here\nand here")?.inline())
            .build()?;
        
        let private_channel = self.http.create_private_channel(self.msg.author.id).await?;
            
        match self.http.create_message(private_channel.id).embed(embed)?.await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e.to_string()))
        }
    }
    
}

/// This is an idempotent function that will create the channels to house all bot data and a category to contain them.
pub async fn create_omni_data_channel(DiscordReferences { http, msg }: &DiscordReferences<'_>, guild_channels: &Vec<GuildChannel>) -> Result<GuildChannel> {
    //Usually we want to make the channel in a category to make things easier for the server owner to manage, so find/make that first.
    let channel_category;
    match guild_channels.iter().find(|&channel| channel.name() == BOT_DATA_CHANNEL_CATEGORY_NAME) {
        Some(category) => {
            channel_category = category.clone();
        }
        None => {
            channel_category = http.create_guild_channel(msg.guild_id.expect("Could not find guild ID when creating bot category!"), BOT_DATA_CHANNEL_CATEGORY_NAME)?
                .kind(GuildCategory)
                .position(999)
                .await
                .context("Could not create category for bot data channel. Does the bot have the correct permissions?")?;
        }
    }

    //Now do it again for the actual channel
    let bot_data_channel: GuildChannel;
    match guild_channels.iter().find(|&channel| channel.name() == BOT_DATA_CHANNEL_NAME) {
        Some(channel) => {
            bot_data_channel = channel.clone();
        }
        None => {
            //Let the user know that we are getting Discord set up for the bot.
            http.create_message(msg.channel_id).reply(msg.id).content(format!("Getting Discord set up."))?.await?;
            bot_data_channel = http.create_guild_channel(msg.guild_id.expect("Could not find guild ID when creating bot category!"), BOT_DATA_CHANNEL_NAME)?
                .parent_id(channel_category.id())
                .await
                .context("Could not create channel for bot data. Does the bot have the correct permissions?")?;
        }
    }
    return Ok(bot_data_channel.clone());
}

pub async fn get_omni_data_channel(discord_references: &DiscordReferences<'_>) -> Result<GuildChannel> {
    let guild_channels = discord_references.http.guild_channels(discord_references.msg.guild_id.expect("Could not get guild ID!")).await?;

    match guild_channels.iter().find(|&channel| channel.name() == BOT_DATA_CHANNEL_NAME) {
        Some(channel) => {
            return Ok(channel.to_owned());
        }
        None => {
            //Do setup
            let new_channel = create_omni_data_channel(&discord_references, &guild_channels).await?;
            discord_references.http.create_message(discord_references.msg.channel_id).reply(discord_references.msg.id).content(format!("Bot setup complete."))?.await?;
            Ok(new_channel)
        }
    }
}

/// create_custom_emojis will check to see if the necessary emojis exist on the guild. If they do not, this method creates them.
pub async fn create_custom_emojis(discord_references: &DiscordReferences<'_>) -> Result<()>{
    let guild_id = discord_references.msg.guild_id.expect("Could not get guild ID!");
    let emojis = discord_references.http.emojis(guild_id).await?;
    let mut create_1 = true;
    let mut create_2 = true;
    let mut create_3 = true;
    let mut create_free = true;
    let mut create_react = true;
    for emoji in 0..emojis.len() {
        let name = &emojis[emoji].name;
        if name == "1_action" {
            create_1 = false;
        } else if name == "2_actions" {
            create_2 = false;
        } else if name == "3_actions" {
            create_3 = false;
        } else if name == "free_action" {
            create_free = false;
        } else if name == "reaction" {
            create_react = false;
        }
    }
    if create_1 {
        discord_references.http.create_emoji(guild_id, "1_action", "data:image/png;base64,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").await?;
    }
    if create_2 {
        discord_references.http.create_emoji(guild_id, "2_actions", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAABRCAYAAAAaXK5BAAAN40lEQVR4nO1de4xcVRk/d2bb3VbFwD+83wVfQeMzaDRrjFFJjIhmIxoUfBKEBomKpgQ3gAgKRcpDKI+WlkDrpmokpkKiqVh8UIpoH1ErUmRp90F3Z6Y7d+49r/vzjznfzre3O4/dnZl7Z7q/f9rs3HvO932/3/nOueeec48QKQMAD4AnhPByudz6KIqgtb7O/ZZxvy2iG8HIzxw6dGgtyjDWWqu1vtZd09Ei6GTbWwpO/uTk5MMAEEWRjaIoAhABgNZ6VezajgLZ3Im2txQAMu6/HiPflLkHIgcAsNZSd9BRIoiTPzg4mKl9x1ECRmR2YmJinUv7msgncBFora9396a+O+BC1VpfA+D3Sql3u9+yyVqXMKgV9Pf390xMTGwg8lEFnSYCR35GCCG01t+21pIfLyql3uOuOTpFQKT19/f3TE5OPkppvxr5MRFYJ4JBKittIoiR/90oiuDEq1xX9pLv++911x5dImB9fqZQKGyslvZriQCABQAp5Q1ClLNJWkTAyZdSfp/Ip+xFQrfWvqyUOrpEQIHp7+/vyeVyj86V/NlEoLW+0ZWdeCaIkb8qTj6z3wCAMeYVpdT73L3dLQJURsBLc7ncY0T+nJivIgIp5U1UR1IiwMy0v4psjJPP7Lfu3/8ppc53ZXSnCDj5k5OTmxZKPhcBGxP8kOpqtwgws+VfV498Zv90JgDwfldWd4mARvuDg4M9+Xx+M3e8GYiJ4EdCtFcEvC4p5fXOLFuP/LgIrLWvaq0/4MrsDhHAtYqBgYGlhUJhs/N5zn1+A0HkIrjF1d1yEWAm+T9w5jRMPrPfuH8PdI0IqOWvWbOmN5/PDzlfF5z2awRxWgTW2luEEB5aKALMnOQZdGbMmXxmP40JDgRB8CFXR2eKAK7lr1+/vi+fz29xPraMfBZEngl+DEcSmiwCXmYQBDeyuheU2tiY4GAQBP2urs4SAbX8rVu39h4+fPgXAGCtbVqf30AQuQhuQ5NFAMAjH7XWN7k6593yZ7GfsthoEAQfdnV2hgjgWv7KlSt7C4XCdMtvdp/fQBB5d3C7s216pL4A//ho/2YirFnkM/spE4x3jAjgWtjq1auXUctHG9J+jSBOi8AYcwfNFmKemQAzB3y3uDqaTj7BupcH1trxIAg+4mxIpwgoOLt27To2n8//0vmQGPmEuAgAZDEPEWDmgK/l5DP7SQRjYRh+1NmSPhEA6BFCiNHR0W8522UrAzMXxMYEd8IFEA2KgF3nlUqln7gyW04+s58EPJ5aEcC1ECnlm40xf3eGt23gVw+xTHAXKgs06o4JAHg7d+5cEobhra6stpHP7KdMcCjVIhBCiMOHD59rrd3tDE+rCO5GJRNUFQH9tnv37rPDMATKz/mJ+MQmiybCMPyYs29BImjqsiTP8wAge8wxx+zTWl9kjNnteV4WgG1mPfOF53meKE8ORdls9ipr7Z0AejzPi1ClO6DftmzZsr9UKn1PlGPmAUBbjS/bkgUQeZ533JIlSx4Pw/DjnufZhYpgzgB7Dp4NQ0NDWSGECMNwhdY6rd0Bzb/fs3PnziXOr1qZwBNCiLGxsWtdGW3vBgjs6eCQ7/sXOPuyaPJkV81AxP8/y3UZIYQoFAorjDF/c0FLmwioO7hv27ZtPQ34lBVCiLGxse+4YhITAZs2ngjD8AJuX8vAg7Njx44TZvv7bAELguCstItAa30/3JMMqmQClAe6WSGEeO21165xxZgUiCBnjGmtCCgo+/bt61VK3VcsFie01jewwNQTwRlpF4G19v49e/Ys5f7O4s+0CNgjr7XWJi2CyTAMP8lj3nTy9+/f36eUWs8NaOT9O90fBMEZ1tqdzuDUiACYMfX6wL59+3qFqL12n7qM0dHRq6lLTjoTWGtzYRh+qqkigCPVtfxHKFhuEMTfv9d84UIG5fP5M7XWz1E5SQSsGpgIHgSwlPtfy6eRkZGrjCmPKVMggoKU8kJm3/wHhpiZ9jfyILn/z3j1SgFDHREAON1auyNeXhpA/iil1gFo5OmARPDNFGWCKd/3P83tmzf5APqklLR694gBDxeBlPK2RkVQKpVOS2MmiD0drAPQW0sEYGOCsbGxK5wIohSMCQ5LKS+alwio7ztw4MBypVTdTRuxTHBbvbduqIjgFGPMX+uVnwRYd7AeQB+RXS1mcE8QY2Njl0spo3IR5ZgkYDuJoGCMmZsIUGn5y5RSj/Fg1KmUi+B2uC1bqDMwLJVKp2qt/wK0d9FIIyC/lVIbUBFB3e5gfHz8q1prg5SIQEr5GbIPtSaLwNK+1noTD0KDlfL0eQcrr54ITtFa/3mu9bUDLBNsRJ3ugItgZGTkyySCFHQHU1wENcl3aX966fZcBzRcBEqpO1HnrRsZ5Pv+yWkUQWza+NHh4eFldfyZHhOMjIxc5roDpCATFI0xn51VBOTM+Pj46xdCPquUdwdrgNrv31ERwYla6z9R/U3wvymIiwDA8loiEOVH4h4ngku11hrp6A6mpJSfmyECMPK11kPuwgUHP9Yd3IXKNGvN7sD3/ZO01n9slh3NBOsOHhsdHX1dHRHw7uBLWmuFdHQHRSnl5+MiWK6U2sKdbFKlEQva3WhcBCdqrZ9utj3NAPNnE4Ca3YH7rUcIIQ4ePHiJEwHtaUzCdhJBqVQqXUwGvsEY07SWX6tiY8y9aHBgWCwWT7DW/sHdn0jAqoGJ4OcsE1QdXdO0sRNBCNc2ErKdROBLKS8RUspf0w+tMirWHdwLYDkamCcoFovHG2N+10rb5oNYZtsE4DjUWWiKyjzBlVprm6RPvF5a2WKEED3VjG8GAERCCHielxVCnO953rMAMp7nRdUC5nmeKRaL5y1btuzpTCZzLAC4VT2Jw9liRTluF3qe9wSArPtb/NqM53mR7/sn9fX1Pex53idEORZJbWO3nudlgyDYnlFKPSGE6AHKn2JrUYUQorykyVr7MyHEbpRby6z1uUCaYrF4fF9f3089z3tj2sgXQkRCiB5r7WYhxDPOnyPE7HyJisXiCb29vZszmUyi5AshtOd5Wd/3n9y+ffsFi2OAeaDRMQAqj7cnGWPS8GSjAcD3/Se3bdvWxw1dfApoEI0+BaDiy8lpIr9UKv32qaeeqjzCYnEeoGE0Og8ARr7Weju/NyFQy986g/y4wYszgbNjLjOBYFPbxpjEfaEXbFNTU78ZGhqqPm+BxXcBs2Iu7wJQec09/YYzDS1fKfXMmjVr6r7IWnwbOAtY2q/5NhAVX04zxiTuC+0diKLo1Tl9lRQV0hbXA1TIfwQ11gPQ34IgOD2KosR9IR6stcOlUumD1eyuinauCNJap31F0DrUWBEEtsTNWvssvzchu4n8V0ul0vw/NoUWrwkEcKrWOnULQ2Nifhi10/70noc0LHdnY5XhUqm08A9PktPNXhUcBMHpxpjEW8tsYC3oIdRYFYxK2j/TGJP44laqW2v9SlPIZ442dV9AEARnpHE1MFCxJ4qiB9gOoSP8oc2vbstb4r6wlv9yU8ln5DVtZ5AxJvFUGUdU/qAztaC1cJtCZtsZhMoGl7OMMc+7+9PQ57/c0o9O48i9gYe6bW+g1rrm3kBUyD/bWvuCuz8VLd/3/dYfQIEu3h2slKq5O5iyAYAVKSP/Jd/323cEDbrw+wBKqZ+hxnQ1KrOZ56SJ/CiKXmwr+Twg3fKFEGPMPajR8uHIHx4ePk4p9ZwrIslvH5LdLyql3uVsTM/HosgYR/4ubnQaEJuurvuKmsS8d+/ec8IwVPz+BGyntP9f3/ffyeOdClAQu+krYdwva+1Ka61x5bT7M3FE/n+klG93dqWLfHTpdwLJPyGECMPwamutbacIWNr/t1LqHc6e9JAvRGXVazd+KZT5mBFCCKXUStrg0erugLX8f6Wy5RPAvhVcKBR+5ezvmm8FMx9JBFfRG79W7fbhLb9YLJ7nbEgf+QQK6urVq5elQQRx8hf6tXDyEY4EpdSVURRpoPkiYI96/5yamnqbqzu95BPAzgvgn4xv85hpBvnNPC8gXo5S6goSQbPGBCzt75VSdg75BH5WUDeeGEJARQSXsxU4CxoTsLS/R0r5VldP55BPoOB065lBzkeeCb5hraVMMC8RsJa/R0r5FldH55FPSPLUMK31raLFp4YJcYQIvh5FkXQkzuvYOGvt3jAM3+TK7lzyCRScNp8beKuruy2HR2KmCL5mrZXOpoYyASN/F5FPM5BdAX5y6NTU1OPc6RaQ3/aTQwmoPB18JYoiOia+oaNjrbX/6KqWHweRsXbt2iWFQmHBB0dXIf9mqqvd5LN6SQSXUXdQLRPEyD/XldF95BOIGCeC6aPjF0I+KkfIJ356OKs/K4QQxphLoygKZhNBjPwV7t7uJZ8At19gYGAgm8vlNpII5jomiJF/oys7UfI5iEwp5aVRFJW4CBj5L4RheDa//qgAKpMxXqFQ2EBBaXQihZMvpbxBiPI4Iy3kE5gIvkiZAO4diTHmeSK/qwZ8jYJa68DAQHZiYmIDZYJGyGd9/iAvK2mfZgMTwRcoE7iVxGe535t6blNHgZ4OnAjW1xMBfwWrtb5eiEqXkqwn1QH2iAjgYgAbgyAg8o++lh8Ha72ZfD7/EIkg3ht0Ivkc8ZbeKXa3BZzIfD7/oCPckAg4+dba69w9qU371UB+dprdbQELjDc5OfmQI54+pUYtf1Xs2kV0E3h3MDk5eZ/L/sZaa7XW17prOibtL2Ie4Jkgl8uti6IIWmtK+4vkNxn/B+ZemUGEmWaGAAAAAElFTkSuQmCC").await?;
    }
    if create_3 {
        discord_references.http.create_emoji(guild_id, "3_actions", "data:image/png;base64,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").await?;
    }
    if create_free {
        discord_references.http.create_emoji(guild_id, "free_action", "data:image/png;base64,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").await?;
    }
    if create_react {
        discord_references.http.create_emoji(guild_id, "reaction", "data:image/png;base64,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").await?;
    }

    return Ok(());
}

pub async fn construct_emoji(discord_references: &DiscordReferences<'_>, emoji_name: String) -> Result<String> {
    let mut result_string:String = "".to_string();
    let guild_id = discord_references.msg.guild_id.expect("Could not get guild ID!");
    let emojis = discord_references.http.emojis(guild_id).await?;

    for emoji in 0..emojis.len() {
        if emojis[emoji].name == emoji_name {
            result_string = format!("<:{}:{}>", emoji_name, emojis[emoji].id)
        } 
    }

    return Ok(result_string.to_string());
}

/// Save the omni data to the discord guild to preserve state between bot commands.
/// This also takes care of pinning the new message and unpinning all others.
/// Will only do anything if the omnidata object is dirty.
pub async fn omni_data_save(discord_references: &DiscordReferences<'_>, omnidata: &omni::Omnidata) -> Result<()> {
    if omnidata.is_dirty {
        let serialized = serde_json::to_vec(&omnidata)?;
        println!("Size of vec is: {:?}", size_of_val(&*serialized));
        let data_channel = get_omni_data_channel(&discord_references).await?;
        match discord_references.http.create_message(data_channel.id()).attachment("state", serialized).content(format!("'{}'", &discord_references.msg.content))?.await {
            Err(error) => {
                println!("Error when saving bot data. {:?}", error);
                discord_references.http.create_message(discord_references.msg.channel_id).content("Something went wrong saving the bot data. Rolling back the previous command!")?.await?;
                return Err(anyhow!(error.to_string()));
            },
            Ok(new_message) => {
                // The bot relies on a message being pinned in the data channel to know which one is the 'active' one. Unpin the old one, then pin the new one.
                // TODO: Pinning API is STUPID SLOW. Find a better way, like using the newest message.
                let mut pin_jobs = Vec::new();
                let old_pins = discord_references.http.pins(new_message.channel_id).await?;
                for old_pin in old_pins.iter() {
                    pin_jobs.push(discord_references.http.delete_pin(old_pin.channel_id, old_pin.id));
                }
                let delete_jobs = futures::future::join_all(pin_jobs);
                let foo = discord_references.http.create_pin(new_message.channel_id, new_message.id);
                futures::join!(delete_jobs, foo);
                return Ok(());
             },
        };
    } else {
        return Ok(());
    }
}

/// Given a discord ref struct, find the current omni tracker data, deserialize it, and return a usable object
pub async fn get_tracker(discord_refs: &DiscordReferences<'_>) -> Result<Omnidata> {
    let data_channel = get_omni_data_channel(discord_refs).await?;
    let pins = discord_refs.http.pins(data_channel.id()).await?;

    match pins.len() {
        0 => return Ok(omni::Omnidata::new()),
        _ => {
            let data = reqwest::get(&pins[0].attachments[0].url).await?.text().await?;
            let omnidata: Omnidata = serde_json::from_str(&data)?;
            return Ok(omnidata);
        },
    }
}