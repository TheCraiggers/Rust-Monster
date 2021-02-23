use std::{env, error::Error};
use discord::DiscordReferences;
use futures::stream::StreamExt;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::{GuildChannel, Message, ChannelType::GuildCategory}, gateway::{Intents, payload::MessageCreate}, guild::Guild};
use twilight_command_parser::{Command, CommandParserConfig, Parser};
mod omni;
mod lookup;
pub mod discord;

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
    // Create the commands the bot will listen for
    // TODO: Move this to the parent function. No point in recreating the parser object each time.
    let mut config = CommandParserConfig::new();
    config.add_prefix("!");
    config.add_prefix("! ");    //For mobile users like me. Android puts a space after ! because it's punctuation
    config.add_command("omni", false);
    config.add_command("lookup", false);
    let parser = Parser::new(config);

    match event {
        Event::MessageCreate(msg) => {
            let discord_refs: DiscordReferences = DiscordReferences {http: &http, msg: &msg};
            match parser.parse(&msg.content) {
                Some(Command { name: "omni", arguments, .. }) => {
                    omni::handle_command(&discord_refs).await?;
                },
                Some(Command { name: "lookup", arguments, .. }) => {
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
