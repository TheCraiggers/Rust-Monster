//! At the time of writing this, both Discord libs for Rust are quite new and haven't reached 1.0 status yet.
//! Also, as the communities are still quite small, there are risks about maintainability as well.
//! Because of this, it was decided to create a layer between the "bot code" and the discord library.
//! This way, if the library ever needed to be switched, or if a breaking change was introduced, we could simply
//! update the code here and all of the calling functions would be ignorant.

use omni::{Omnidata};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::{ChannelType::GuildCategory, GuildChannel}, gateway::{payload::MessageCreate}};
use anyhow::{Context, Result, anyhow};
use crate::omni;
use reqwest;
use futures;
use core::mem::size_of_val;

pub const BOT_DATA_CHANNEL_CATEGORY_NAME: &str = "rust-monster-bot-data";
pub const BOT_DATA_CHANNEL_NAME: &str = "omni-bot-data";

//Vec Mutex to hold all the Boxed Mutexes holding the trackers. Look for a dict so I can reference by guild id.


/// The standard amount of info that all discord functions take.
pub struct DiscordReferences<'a> {
    pub http: &'a HttpClient,
    pub msg: &'a Box<MessageCreate>,
}

impl DiscordReferences<'_> {
    pub async fn send_message(&self, text: &str) -> Result<()>{
        match self.http.create_message(self.msg.channel_id).content(text)?.await {
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
