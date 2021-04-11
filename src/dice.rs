use std::sync::Arc;

use roll_lib;
use futures::lock::Mutex;
use crate::discord::{DiscordReferences};
use anyhow::{Result, anyhow};
use crate::omni::Omnidata;

pub async fn handle_command(
    discord_refs: &DiscordReferences<'_>, 
    omnidata_cache: Arc<Mutex<Option<Omnidata>>>,
    arguments: &str,
) -> Result<()> {
    let roll = roll_lib::roll_inline(arguments, true).unwrap();
    discord_refs.send_message_reply(&format!("{}", &roll.string_result)).await?;
    Ok(())
}