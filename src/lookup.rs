//use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{payload::MessageCreate};
use twilight_model::channel::embed::{Embed, EmbedField};
use anyhow::{Result};
use reqwest;

pub async fn lookup(http: &HttpClient, msg: &Box<MessageCreate>, keyword: String) -> Result<(), Box<dyn std::error::Error>> {
    //Use embed to build the results of the lookup.
    let search_results = search_for_term(&keyword).await?;
    if &search_results.len() == &0 {
        //Can't find any results. Alert user and get out of this function.
        http.create_message(msg.channel_id).reply(msg.id).content(format!("Sorry, couldn't find anything when searching for {}", &keyword))?.await?;
        return Ok(());
    } else if &search_results.len() == &1 {
        //Exact match! Start building the embed and send a response
        println!("Exact match! {:?}", &search_results);
    } else {
        //Disambiguous. Ask user which of the short list they mean.
        println!("A lot of matches {:?}", &search_results);
    }
    let embed = Embed {
        author: None,
        color: Some(12009742),
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
        timestamp: None,
        title: Some("Title".to_owned()),
        url: Some("https://pf2.easytool.es/".to_owned()),
        video: None        
    };
    http.create_message(msg.channel_id).reply(msg.id).embed(embed)?.await?;
    Ok(())
}

/**
 * This method searches for a term, then adds the top 3 results to a vector in the form of
 * ["Prescient Planner - GENERAL FEAT 3 > 8462", "Prescient Consumable - GENERAL FEAT 7 > 8461"]
 **/
pub async fn search_for_term(term: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    //This function will return a vector of search results
    let req_body:String = format!("name={}", term);
    let client = reqwest::Client::new();
    let mut search_results: Vec<String> = Vec::new();
    //Send the request
    let response = client.post("https://pf2.easytool.es/php/search.php")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(req_body)
        .send()
        .await?;
    //Check the response for a success
    if response.status().is_success() {
        //Split the response
        let response_string: String = response.text().await?;
        let split_response = response_string.split("<button");
        //Add split items to search_results vector
        let mut i: i8 = 0;
        for r in split_response {
            //Set start and end position of each string split
            let start_bytes_title = r.find("<strong>").unwrap_or(0)+8;
            let end_bytes_title = r.find("</strong>").unwrap_or(r.len());
            let start_bytes_extra = r.find("<small>").unwrap_or(0)+7;
            let end_bytes_extra = r.find("</small>").unwrap_or(r.len());
            let start_bytes_id = r.find("value='").unwrap_or(0)+7;
            let end_bytes_id = r.find("' />").unwrap_or(r.len());
            //Some dumb result comes back in the split like "/n/t/t". This if statement handles that.
            if &r.len() > &8 {
                let one_result_title = &r[start_bytes_title..end_bytes_title];
                let one_result_extra = &r[start_bytes_extra..end_bytes_extra];
                let one_result_id = &r[start_bytes_id..end_bytes_id];
                let one_result = format!("{} - {} > {}", one_result_title, one_result_extra, one_result_id);
                //Add the result to the vector
                search_results.push(one_result);
            }
            i+=1;
            if i > 3 {
                println!("to Cancel.");
                break;
            }
        }
        return Ok(search_results);
    }
    return Ok(search_results);
}