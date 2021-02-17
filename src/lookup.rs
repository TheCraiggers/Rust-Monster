//use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{payload::MessageCreate};
use twilight_model::channel::embed::{Embed, EmbedField};
use anyhow::{Result};

pub async fn lookup(http: &HttpClient, msg: &Box<MessageCreate>, keyword: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("In lookup function");
    let embed = Embed {
        author: None,
        color: Some(123),
        description: Some("Here is a description".to_owned()),
        fields: vec![EmbedField {
            inline: true,
            name: keyword,
            value: "You looked up a keyword!".to_owned()
            },
            EmbedField {
            inline:true,
            name: "Static Name".to_owned(),
            value: "Static Value".to_owned()
            }
        ],
        footer: None,
        image: None,
        kind: "rich".to_owned(),
        provider: None,
        thumbnail: None,
        timestamp: Some("timestamp".to_owned()),
        title: Some("Title".to_owned()),
        url: Some("https://pf2.easytool.es/".to_owned()),
        video: None        
    };
    println!("Created Embed");
    http.create_message(msg.channel_id).reply(msg.id).embed(embed)?.await?;
    Ok(())
}
