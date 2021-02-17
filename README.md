![rust v1.50.0+](https://img.shields.io/badge/rust-v1.50.0+-orange)

# Pathfinder Discord Bot
*A bot to help Pathfinder 2e players and GMs while using Discord*

# Developer Information
*Information for developers and those looking to host this bot locally or in debug mode.*
## Pre-requisites
- Rust v1.50.0+ (https://blog.rust-lang.org/2021/02/11/Rust-1.50.0.html)
- Discord Developer Portal access (https://discord.com/developers/applications)
- Bot added to discord and a unique token created (https://www.saintlad.com/add-bots-to-discord-server/)

## Running bot locally
1. Install the pre-requisites listed above
2. Clone the repository
    `git clone https://github.com/TheCraiggers/Rust-Monster.git`
3. Build the project
    `cargo build`
4. Run the bot in debug mode
    `RUST_LOG="debug" DISCORD_TOKEN="<INSERT_DISCORD_TOKEN_HERE>" ./target/debug/rust-monster`