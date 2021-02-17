use std::{env, error::Error};
use futures::stream::StreamExt;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::{GuildChannel, Message, ChannelType::GuildCategory}, gateway::{Intents, payload::MessageCreate}, guild::Guild};
use twilight_command_parser::{Command, CommandParserConfig, Parser};
use anyhow::{Result, Context};
mod omni;
mod setup;
mod lookup;

const BOT_DATA_CHANNEL_NAME: &str = "omni-bot-data"; //This variable and string also exist in setup.rs. If an update is made, it needs to be made there too.

const BOT_DATA_CHANNEL_CATEGORY_NAME: &str = "rust-monster-bot-data";
const BOT_DATA_CHANNEL_NAME: &str = "omni-bot-data";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN")?;
    tracing_subscriber::fmt::init();

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let cluster = Cluster::builder(&token, Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await?;

    // Start up the cluster.
    let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = HttpClient::new(&token);

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    let mut events = cluster.events();

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(shard_id, event, http.clone()));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    //Create the commands the bot will listen for
    let mut config = CommandParserConfig::new();
    config.add_prefix("!");
    config.add_prefix("! ");    //For mobile users like me. Android puts a space after ! because it's punctuation
    config.add_command("omni", false);
    config.add_command("lookup", false);
    let parser = Parser::new(config);

    match event {
        Event::MessageCreate(msg) => {
            match parser.parse(&msg.content) {
                Some(Command { name: "omni", arguments, .. }) => {
                    //Get the bot data from the guild. But first, we need to get the channel, or create it.
                    let guild_channels = http.guild_channels(msg.guild_id.expect("Could not get guild ID!")).await?;
                    let bot_data_channel;
                    let bot_data_message: &Message;
                    match guild_channels.iter().find(|&channel| channel.name() == BOT_DATA_CHANNEL_NAME) {
                        Some(channel) => {
                            println!("Found the bot channel!");
                            bot_data_channel = channel;
                        }
                        None => {
                            //Do setup
                            &setup::create_omni_data_channel(&http, &msg, &guild_channels).await?;
                            http.create_message(msg.channel_id).reply(msg.id).content(format!("You are good to go!"))?.await?;
                        }
                    }
                    //Next, get the messages in that channel and look for the active one.
                    //Finally, send the command args & the current data message to the omni crate entry point
                },
                Some(Command { name: "lookup", arguments, .. }) => {
                    println!("In Lookup command");
                    &lookup::lookup(&http, &msg, arguments.as_str().to_string()).await;
                }
                //Ignore anything that doesn't match the commands above.
                Some(_) => {},
                None => {},
            }
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }        
        // Other events here...
        _ => {}
    }

    Ok(())
}

async fn create_omni_data_channel(http: &HttpClient, msg: &Box<MessageCreate>, guild_channels: &Vec<GuildChannel>) -> Result<GuildChannel> {
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
            println!("Creating channel for bot data.");
            bot_data_channel = http.create_guild_channel(msg.guild_id.expect("Could not find guild ID when creating bot category!"), BOT_DATA_CHANNEL_NAME)?
                .parent_id(channel_category.id())
                .await
                .context("Could not create channel for bot data. Does the bot have the correct permissions?")?;
        }
    }
    return Ok(bot_data_channel.clone());
}