use omni::{Omnidata, constructTrackerFromMessage};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::{GuildChannel, ChannelType::GuildCategory}, gateway::{payload::MessageCreate}};
use anyhow::{Context, Result};
use serde_json::json;
use serde::{Deserialize, Serialize};
use crate::omni;

pub const BOT_DATA_CHANNEL_CATEGORY_NAME: &str = "rust-monster-bot-data";
pub const BOT_DATA_CHANNEL_NAME: &str = "omni-bot-data"; //This variable and string also exist in main.rs. If an update is made, it needs to be made there too.

/// The standard amount of info that all discord functions take.
pub struct DiscordReferences<'a> {
    pub http: &'a HttpClient,
    pub msg: &'a Box<MessageCreate>,
}

/// This is an idempotent function that will create the channels to house all bot data and a category to contain them.
pub async fn create_omni_data_channel(DiscordReferences { http, msg }: &DiscordReferences<'_>, guild_channels: &Vec<GuildChannel>) -> Result<GuildChannel> {
    //Usually we want to make the channel in a category to make things easier for the server owner to manage, so find/make that first.
    let channel_category;
    match guild_channels.iter().find(|&channel| channel.name() == BOT_DATA_CHANNEL_CATEGORY_NAME) {
        Some(category) => {
            println!("Found bot data channel category!");
            channel_category = category.clone();
        }
        None => {
            println!("Creating category for bot data.");
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
            println!("Found bot data channel!");
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

/// Save the omni data to the discord guild to preserve state between bot commands.
/// Will only do anything if the omnidata object is dirty.
pub fn omni_data_save(DiscordReferences { http, msg }: &DiscordReferences<'_>, omnidata: omni::Omnidata) -> Result<()> {
    let foo = serde_json::to_string(&omnidata);
    println!("{:?}", foo);
    return Ok(());
}