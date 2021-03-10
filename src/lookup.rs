use twilight_http:: {request::channel::reaction::RequestReactionType};
use twilight_model::channel::{embed::{Embed, EmbedField}};
use anyhow::{Result};
use convert_case::{Case, Casing};
use tokio::time::{sleep, Duration};
use reqwest;
use crate::discord::{DiscordReferences, create_custom_emojis, construct_emoji};

const MAX_RESULTS: i8 = 5; //Number of ambiguous results to show: up to 9
const REACTIONS: [&str; 5] = ["\u{0031}\u{20E3}", "\u{0032}\u{20E3}", "\u{0033}\u{20E3}", "\u{0034}\u{20E3}", "\u{0035}\u{20E3}"]; //This should be the same length as MAX_RESULTS, all unicode numeric reactions
const CANCEL: &str = "\u{274C}"; //Unicode for the red X

//TODO: Abstract the discord api methods. Like "build_embed_from_struct" and "send_text_message" and "send_embed_message"
///Lookup accepts an HttpClient, MessageCreate, and keyword String then outputs a boolean. A "true" output means that the lookup has returned it's result. A "false" output means that it has returned too many results and needs user interaction.
pub async fn lookup(discord_refs: &DiscordReferences<'_>, keyword: String) -> Result<(), Box<dyn std::error::Error>> {
    let _typing = discord_refs.http.create_typing_trigger(discord_refs.msg.channel_id).await;
    create_custom_emojis(&discord_refs).await?;
    let search_results = search_for_term(&keyword).await?;
    if &search_results.len() == &0 {
        //Can't find any results. Alert user and get out of this function.
        discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).content(format!("Sorry, couldn't find anything when searching for {}", &keyword))?.await?;
        return Ok(());
    } else if &search_results.len() == &1 {
        //Exact match! Start building the embed and send a response
        let embed = build_embed(discord_refs, &search_results).await?;
        discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).embed(embed)?.await?;
        return Ok(());
    } else {
        //Ambiguous. Ask user which of the short list they mean.
        let mut options_string: String = String::from("");
        for option in 0..search_results.len() {
            let mut named_option = Vec::new();
            named_option.push(String::from(&search_results[option]));
            let option_info = extract_info(&named_option).await?;
            options_string = format!("{}\n{} - {}", options_string, REACTIONS[option].to_string(), option_info);
        }
        //Add cancel option
        options_string = format!("{}\n{} - Cancel", options_string, CANCEL.to_string());

        let clarification = discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).content(format!("Found more than one possible term. Please let me know which one to look up by simply reacting to this message with the emoji beside the desired choice.\n{}", options_string))?.await?;
        //Add reactions to allow user to select
        for option in 0..search_results.len() {
            let num = REACTIONS[option].to_string();
            let react = discord_refs.http.create_reaction(clarification.channel_id, clarification.id, RequestReactionType::Unicode { name: num } ).await?;
        }
        //React with CANCEL
        discord_refs.http.create_reaction(clarification.channel_id, clarification.id, RequestReactionType::Unicode {name: CANCEL.to_string()} ).await?;

        //THREAD SLEEP DREAD SLEEP
        for t in 0..20 {
            let mut reaction_list = discord_refs.http.reactions(clarification.channel_id, clarification.id, RequestReactionType::Unicode { name: CANCEL.to_string()}).await?;
            if reaction_list.iter().any(| UserId | UserId == &discord_refs.msg.author) {
                discord_refs.http.delete_message(discord_refs.msg.channel_id, clarification.id).await?;
                break
            }
            for option in 0..search_results.len() {
                let num = REACTIONS[option].to_string();
                reaction_list = discord_refs.http.reactions(clarification.channel_id, clarification.id, RequestReactionType::Unicode { name: num }).await?;
                if reaction_list.iter().any(| UserId | UserId == &discord_refs.msg.author) {
                    let _typing = discord_refs.http.create_typing_trigger(discord_refs.msg.channel_id).await;
                    let response_string = &search_results[option];
                    let mut response_vec: Vec<String> = Vec::new();
                    response_vec.push(response_string.to_string());
                    discord_refs.http.delete_message(discord_refs.msg.channel_id, clarification.id).await?;
                    let embed = build_embed(discord_refs, &response_vec).await?;
                    discord_refs.http.create_message(discord_refs.msg.channel_id).reply(discord_refs.msg.id).embed(embed)?.await?;
                    break
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        discord_refs.http.delete_message(discord_refs.msg.channel_id, clarification.id).await?;

        return Ok(());
    }
}

///extract_id splits a result and extracts the id to be used later
async fn extract_id(result: &Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    let slice = &result[0];
    let start_b_slice = slice.find(">").unwrap()+1;
    let mut id = String::with_capacity(10); 
    id = slice[start_b_slice..].to_string();
    return Ok(id)
}

///extract_info splits a result and extracts the title and subtitle to be used later
 async fn extract_info(result: &Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    let slice = &result[0];
    let end_b_slice = slice.find(">").unwrap();
    let mut info = String::with_capacity(10); 
    info = slice[..end_b_slice].to_string();
    return Ok(info)
}

///split_string splits a string given a starting text and and ending text.
async fn split_string(split_me: &str, start_split: &str, end_split: &str) -> Result<String, Box<dyn std::error::Error>> {
    let skip_length = start_split.len();
    let start_byte = split_me.find(start_split).unwrap_or(0)+skip_length;
    let end_byte = split_me.find(end_split).unwrap_or(split_me.len());
    let output = &split_me[start_byte..end_byte];
    return Ok(output.to_string());
}

///sanitize removes html formatting from a string. It replaces some html formatting with discord syntax, and some emojis.
async fn sanitize(sanitize_me: &str) -> Result<String, Box<dyn std::error::Error>> {
    let iter = sanitize_me.len()/3;
    let mut return_string = sanitize_me.to_string();
    for n in 0..iter {
        if return_string.find("<") == None || return_string.find(">") == None {
            return Ok(return_string)
        }
        let start_byte = return_string.find("<").unwrap(); 
        let mut end_byte = return_string.find(">").unwrap()+1;
        let mut temp_string = &return_string[start_byte+1..]; 
        let mut open_count = 1;
        let mut close_count = 0;
        let mut byte_count = start_byte+1;
        loop {
            if temp_string.find("<") == None || temp_string.find(">") == None {
                if temp_string.find(">") != None {
                    end_byte = return_string.rfind(">").unwrap()+1;
                } else{
                end_byte = byte_count;
                }
                break
            }
            if temp_string.find("<").unwrap() < temp_string.find(">").unwrap() {
                open_count += 1;
                byte_count = byte_count+temp_string.find("<").unwrap()+1;
                temp_string = &temp_string[temp_string.find("<").unwrap()+1..];
            } else {
                close_count += 1;
                byte_count = byte_count+temp_string.find(">").unwrap()+1;
                temp_string = &temp_string[temp_string.find(">").unwrap()+1..];
            }
            if open_count == close_count {
                end_byte = byte_count;
                break
            }
        }
        
        let slice = &return_string[start_byte..end_byte];
        let mut new_slice = "";
        //TODO: use custom emojis like https://github.com/Rapptz/discord.py/issues/390 instead of :one:, :two:, etc.
        //Use http.get_emojis and http.create_emoji to accomplish this in bot setup. Then get the id of the emojis and store them as public consts.
        if &slice == &"<p>" || &slice == &"<p class=\"fancy\">" || slice.contains("/h3") || &slice == &"<tr>" {
            new_slice = "\n";
        } else if &slice == &"</section>" {
            new_slice = "\n------------";
        } else if &slice == &"<strong>" {
            new_slice = " **";
        } else if &slice == &"</strong>" {
            new_slice = "** ";
        } else if &slice == &"<em>" {
            new_slice = " _";
        } else if &slice == &"</em>" {
            new_slice = "_ ";
        } else if &slice == &"</th>" || &slice == &"</td>" {
            new_slice = "|";
        } else if slice.contains("class=\"pf2 action1\"") {
            new_slice = " :1_action: ";
        } else if slice.contains("class=\"pf2 action2\"") {
            new_slice = " :2_actions: ";
        } else if slice.contains("class=\"pf2 action3\"") {
            new_slice = " :3_actions: ";
        } else if slice.contains("class=\"pf2 Reaction\"") {
            new_slice = " :reaction: ";
        } else if slice.contains("class=\"pf2 actionF\"") {
            new_slice = " :free_action: ";
        }
        return_string = return_string.replace(slice, new_slice);
    }
    return Ok(return_string)
}

/// pretty_format Makes some further alterations to strings for the embed. 
/// It does things liks split up types of items and add some line breaks to make spell outcomes easier to read
async fn pretty_format(mut string_to_format: String) -> Result<String, Box<dyn std::error::Error>> {
    string_to_format = str::replace(&string_to_format, "\nType", "\n\nType");
    string_to_format = str::replace(&string_to_format, "**Critical Success**", "\n\n**Critical Success**");
    string_to_format = str::replace(&string_to_format, "**Success**", "\n\n**Success**");
    string_to_format = str::replace(&string_to_format, "**Failure**", "\n\n**Failure**");
    string_to_format = str::replace(&string_to_format, "**Critical Failure**", "\n\n**Critical Failure**");
    return Ok(string_to_format)
}

///search_for_term searches for a term, then adds the top 3 results to a vector in the form of
///["Prescient Planner - GENERAL FEAT 3 > 8462", "Prescient Consumable - GENERAL FEAT 7 > 8461"]
async fn search_for_term(term: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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
            //Some dumb result comes back in the split like "/n/t/t". This if statement handles that.
            if &r.len() > &8 {
                //TODO: Figure out a way to get these to run concurrently
                let one_result_title = split_string(r, "<strong>", "</strong>").await?;
                let one_result_extra = split_string(r, "<small>","</small>").await?;
                let one_result_id = split_string(r, "value='", "' />").await?;
                let one_result = format!("{} - {}>{}", one_result_title, one_result_extra, one_result_id);
                //Add the result to the vector
                search_results.push(one_result);
            }
            i+=1;
            if i > MAX_RESULTS {
                break;
            }
        }
        return Ok(search_results);
    } else{
        return Ok(search_results);
    }
}

///build_embed uses an id to find the specific result. Then builds an embed.
///The embed should use Title, Traits, Details, Description, and URL
async fn build_embed(discord_refs: &DiscordReferences<'_>, result: &Vec<String>) -> Result<Embed, Box<dyn std::error::Error>> {
    let id = extract_id(&result).await?;
    let req_body:String = format!("id={}", id);
    let req_url:String = format!("https://pf2.easytool.es/index.php?id={}", &id);
    let client = reqwest::Client::new();
    //Send the request
    let response = client.get(&req_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(req_body)
        .send()
        .await?;
    //Check the response for a success
    if response.status().is_success() {
        let response_string: String = response.text().await?;
        //println!("{:#?}", response_string);
        //Get title
        let title = extract_info(&result).await?
            .to_case(Case::Upper);
        //Get main description
        let init_description = split_string(&response_string, "description\' content=\'", "\' />").await?;
        //Chop the length of the description if it is too large for the embed
        let mut description: String = "Description Placeholder".to_string();
        description = init_description;
        //If traits exist, get the traits, otherwise send an empty struct
        let mut traits = EmbedField {
            inline: true,
            name: "Empty".to_owned(),
            value: "Empty".to_owned()
        };
        if &response_string.find("class=\"content\"").unwrap_or(0) != &0 {
            let another_description = split_string(&response_string, "class=\"content\">", "</section>\n\t\t\t<footer class").await?;
            description = sanitize(&another_description).await?;
        }

        if &response_string.find("class=\'traits\'>").unwrap_or(0) != &0 {
            let traits_string = split_string(&response_string, "class=\'traits\'>", "</section>\n\t\t\t\t<section class=\'details\'>").await?;
            let mut traits_value = sanitize(&traits_string).await?;
            if traits_value.find("\n\n").unwrap_or(0) != 0 {
                let end_traits = traits_value.find("\n\n").unwrap();
                traits_value = traits_value[..end_traits].to_string();
            }
            //println!("TRAITS: {}", &traits_value);
            traits = EmbedField {
                inline: true,
                name: "Traits".to_owned(),
                value: traits_value.to_owned()
            };
        }
        //If details exist, get the details
        if &response_string.find("class=\'details\'>").unwrap_or(0) != &0 {
            let details_string = split_string(&response_string, "class=\'details\'>", "</section>\n\t\t\t<footer class").await?;
            //Update description to have the detail from the details
            description = sanitize(&details_string).await?;
        }
        //Build fields
        let mut fields_vec: Vec<EmbedField> = [].to_vec();
        if traits.name != "Empty" {
            fields_vec = vec![traits];
        }

        //Replace emojis after all the formatting is said and done
        let one_action = construct_emoji(discord_refs, "1_action".to_string()).await?;
        let two_actions = construct_emoji(discord_refs, "2_actions".to_string()).await?;
        let three_actions = construct_emoji(discord_refs, "3_actions".to_string()).await?;
        let free_action = construct_emoji(discord_refs, "free_action".to_string()).await?;
        let reaction = construct_emoji(discord_refs, "reaction".to_string()).await?;
        
        description = str::replace(&description, ":1_action:", &one_action);
        description = str::replace(&description, ":2_actions:", &two_actions);
        description = str::replace(&description, ":3_actions:", &three_actions);
        description = str::replace(&description, ":free_action:", &free_action);
        description = str::replace(&description, ":reaction:", &reaction);

        //Pretty format for success/failure conditions.
        description = pretty_format(description).await?;
        if &description.len() > &2048 {
            description = format!("{}...[more]({})", &description[..1991], req_url);
        }

        //Finally actually build the embed
        let embed = Embed {
            author: None,
            color: Some(12009742),
            description: Some(description.to_string()), //Uses discord markdown :emoji: **bold** _italic_ __underline__ and ***bold italic***
            fields: fields_vec,
            footer: None,
            image: None,
            kind: "rich".to_owned(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: Some(title.to_owned()),
            url: Some(req_url.to_owned()),
            video: None        
        };
        return Ok(embed);
    } else {
        let embed = Embed {
            author: None,
            color: Some(12009742),
            description: Some("Unable to connect to pf2.easytools".to_owned()),
            fields: vec![EmbedField {
                inline: true,
                name: "Traits".to_owned(),
                value: "This was a lookup operation".to_owned()
                },
                EmbedField {
                inline:true,
                name: "Details".to_owned(),
                value: "Tried to create embed for discord and failed.".to_owned()
                }
            ],
            footer: None,
            image: None,
            kind: "rich".to_owned(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: Some("Error Occured".to_owned()),
            url: Some(req_url.to_owned()),
            video: None        
        };

        return Ok(embed);
    }
}