use std::str::FromStr;
use std::env;

use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::model::channel::Message;
use serenity::model::id::MessageId;
use serenity::model::channel::ReactionType;
use tracing::info;

use kekw_db::periods::get_most_recent_closed_period;
use kekw_db::rolls::get_roll_by_period;
use kekw_db::submissions::get_submission_by_id;

use crate::DBConnectionContainer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

fn roll_movies() {
    
}

pub async fn select_movie(ctx: &Context) -> Result<Message>{
    info!("Selecting Movie!");

    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    // Get the most recently close period
    let period = get_most_recent_closed_period(&db_pool)?;

    // Get the rolls associated with that period
    let roll = get_roll_by_period(&db_pool, &period)?;

    // Get the emotes from that roll selection
    let selection_1_emote = ReactionType::from_str(&roll.selection_1_emote.unwrap()).unwrap();
    let selection_2_emote = ReactionType::from_str(&roll.selection_2_emote.unwrap()).unwrap();

    let movie_channel_id = env::var("MOVIE_CHANNEL").unwrap().parse::<u64>().expect("MOVIE_CHANNEL not a correct Discord Channel ID!");
    let movie_channel = ChannelId(movie_channel_id);

    // Load the reactions from the voting message
    let vote_msg: Message = movie_channel.message(&ctx.http, MessageId::from(period.vote_message.unwrap().parse::<u64>().unwrap())).await.unwrap();

    let mut selection_1_count: u64 = 0;
    let mut selection_2_count: u64 = 0;

    for reaction in &vote_msg.reactions {
        if reaction.reaction_type == selection_1_emote {
            selection_1_count = reaction.count;
        } else if reaction.reaction_type == selection_2_emote {
            selection_2_count = reaction.count;
        }
    }

    let submission_1 = get_submission_by_id(&db_pool, roll.selection_1)?;
    let submission_2 = get_submission_by_id(&db_pool, roll.selection_2)?;

    // Compare votes and set message!
    let mut message_str = String::from("");

    if selection_1_count > selection_2_count {
        message_str = format!("{} wins!", submission_1.title);
    } else if selection_2_count > selection_1_count {
        message_str = format!("{} wins!", submission_2.title);
    } else if selection_1_count == selection_2_count {
        message_str = format!("{} and {} tied!", submission_1.title, submission_2.title);
    } else {
        message_str = String::from("Something went wrong.....");
    }

    Ok(movie_channel.say(&ctx.http, message_str).await.unwrap())
}