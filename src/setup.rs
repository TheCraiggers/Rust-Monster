use twilight_http::Client as HttpClient;
use twilight_model::guild as Guild;

pub fn setup(client: HttpClient){
    let channel_names: [String; 2] = ["omni-bot-data".to_string(), "Reference".to_string()];
    let role_names: [String; 2] = ["GM".to_string(), "Players".to_string()];
    create_channels(client, channel_names);
    //create_roles(role_names);
    //Send message to let user know that the setup is complete
}

async fn create_channels (client: HttpClient, channel_arr: [String; 2]) {
    for channel_name in channel_arr.iter() {
        if check_channel(channel_name.to_string()) {
            println!("{} channel already exists", channel_name);
        } else {
            //Create the channel. 
            //HOW DO I GET A GUILD ID?
            let guild_id = Guild::GuildInfo.id;
            let new_channel = client.create_guild_channel(guild_id, channel_name.to_string());
            println!("{} channel created.", channel_name);
        }
    }
    Ok(());
}

fn check_channel (channel_name_string: String) -> bool {
    let guild_channels = Guild::Guild.channels;
    match guild_channels.iter().find(|&channel| channel.name() == channel_name_string) {
        Some(channel) => {
            return true;
        }
        None => {
            return false;
        }
    }
}