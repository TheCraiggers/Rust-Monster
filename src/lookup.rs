use twilight_http::Client as HttpClient;
use twilight_model::gateway::{payload::MessageCreate};
use twilight_model::channel::embed::{Embed, EmbedField};
use twilight_standby::Standby;
use anyhow::{Result};
use convert_case::{Case, Casing};
use reqwest;

//TODO: Abstract the discord api methods. Like "build_embed_from_struct" and "send_text_message" and "send_embed_message"
pub async fn lookup(http: &HttpClient, msg: &Box<MessageCreate>, keyword: String) -> Result<(), Box<dyn std::error::Error>> {
    let search_results = search_for_term(&keyword).await?;
    if &search_results.len() == &0 {
        //Can't find any results. Alert user and get out of this function.
        http.create_message(msg.channel_id).reply(msg.id).content(format!("Sorry, couldn't find anything when searching for {}", &keyword))?.await?;
        return Ok(());
    } else if &search_results.len() == &1 {
        //Exact match! Start building the embed and send a response
        println!("Exact match! {:?}", &search_results);
        let embed = build_embed(&search_results).await?;
        http.create_message(msg.channel_id).reply(msg.id).embed(embed)?.await?;
    } else {
        //Ambiguous. Ask user which of the short list they mean.
        println!("A lot of matches {:?}", &search_results);
        let mut options_string: String = String::from("");
        for option in 0..search_results.len() {
            let mut named_option = Vec::new();
            named_option.push(String::from(&search_results[option]));
            let option_info = extract_info(&named_option).await?;
            options_string = format!("{}\n[{}] {}", options_string, option+1, option_info);
        }
        //Add cancel option
        options_string = format!("{}\n[{}] Cancel", options_string, search_results.len()+1);

        http.create_message(msg.channel_id).reply(msg.id).content(format!("Found more than one possible term. Please let me know which one to look up by simply responding with the number shown beside the desired choice.\n{}", options_string))?.await?;
        let standby = Standby::new();
        let message = msg.to_owned();
        let decision = standby.wait_for_message(message.channel_id, move |event: &MessageCreate| {
            event.author.id == message.author.id && (event.content == "1" || event.content == "2" || event.content == "3" || event.content == "4")
        }).await?;
        println!("DECISION? {:?}", decision);
        //Convert user choice to usize, then respond with choice
        let user_response = decision.content.parse::<usize>().unwrap();
        if user_response == search_results.len()+1 {
            println!("Canceled");
        } else {
            println!("Do something");
            let response_string = &search_results[user_response];
            let mut response_vec: Vec<String> = Vec::new();
            response_vec.push(response_string.to_string());
            let embed = build_embed(&response_vec).await?;
            http.create_message(msg.channel_id).reply(msg.id).embed(embed)?.await?;
        }
    }
    Ok(())
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

///sanitize removes html formatting from a string. It replaces <p> with new lines and h3 with spaces
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
                    end_byte = return_string.find(">").unwrap()+1;
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
        if &slice == &"<p>" || &slice == &"<p class=\"fancy\">" || slice.contains("/h3") {
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
        } else if slice.contains("class=\"pf2 action1\"") {
            new_slice = " :one: ";
        } else if slice.contains("class=\"pf2 action2\"") {
            new_slice = " :two: ";
        } else if slice.contains("class=\"pf2 action3\"") {
            new_slice = " :three: ";
        } else if slice.contains("class=\"pf2 Reaction\"") {
            new_slice = " :arrow_heading_down: ";
        } else if slice.contains("class=\"pf2 actionF\"") {
            new_slice = " :free: ";
        }
        return_string = return_string.replace(slice, new_slice);
    }
    return Ok(return_string)
}

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
            if i > 3 {
                println!("to Cancel.");
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
async fn build_embed(result: &Vec<String>) -> Result<Embed, Box<dyn std::error::Error>> {
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
        //Get title
        let title = extract_info(&result).await?
            .to_case(Case::Upper);
        //Get main description
        let init_description = split_string(&response_string, "description\' content=\'", "\' />").await?;
        //Chop the length of the description if it is too large for the embed
        let mut description: String = "Description Placeholder".to_string();
        //Pretty format for success/failure conditions.
        description = pretty_format(description).await?;

        if &init_description.len() > &2047 {
            description = format!("{}{}", &init_description[..2019], "... click the title for more");
        } else {
            description = init_description;
        }
        //If traits exist, get the traits, otherwise send an empty struct
        let mut traits = EmbedField {
            inline: true,
            name: "Empty".to_owned(),
            value: "Empty".to_owned()
        };

        if &response_string.find("class=\'traits\'>").unwrap_or(0) != &0 {
            let traits_string = split_string(&response_string, "class=\'traits\'>", "</section>\n\t\t\t\t<section class=\'details\'>").await?;
            let traits_value = sanitize(&traits_string).await?;
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
            let mut description_value = sanitize(&details_string).await?;
            description_value = pretty_format(description_value).await?;
            println!("DESCRIPTION: {}", description_value);
            if &description_value.len() > &2047 {
                description = format!("{}{}", &description_value[..2019], "... click the title for more");
            } else {
                description = description_value;
            }
        }
        //println!("{:#?}", response_string);
        //Build fields
        let mut fields_vec: Vec<EmbedField> = [].to_vec();
        if traits.name != "Empty" {
            fields_vec = vec![traits];
        }

        let embed = Embed {
            author: None,
            color: Some(12009742),
            description: Some(description.to_string()), //emoji can be placed in here as :emoji_name: "Something &lt;strong>bold</strong> here"
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