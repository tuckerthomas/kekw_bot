use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};

use tracing::{info, error};

use crate::db::{
    DBConnectionContainer,
    submissions
};

use crate::models::submission::Submission;

#[command]
pub async fn submit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        if let Err(why) = msg.channel_id.say(&ctx.http, "No movie supplied.").await {
            error!("Error sending message: {:?}", why);
        }
    } else {
        let movie_submission = args.rest();

        let db_pool = {
            // While data is a RwLock, it's recommended that you always open the lock as read.
            // This is mainly done to avoid Deadlocks for having a possible writer waiting for multiple
            // readers to close.
            let data_read = ctx.data.read().await;

            // Since the CommandCounter Value is wrapped in an Arc, cloning will not duplicate the
            // data, instead the reference is cloned.
            // We wap every value on in an Arc, as to keep the data lock open for the least time possible,
            // to again, avoid deadlocking it.
            data_read.get::<DBConnectionContainer>().expect("Expected DBConnection in TypeMap.").clone()
        };

        let movie_subs = submissions::check_prev_sub(&db_pool.get().unwrap(), &msg.author.id.to_string());

        let mut response = String::new();

        if movie_subs.len() == 0 {
            let num_added = submissions::create_moviesub(&db_pool.get().unwrap(), &msg.author.id.to_string(), movie_submission, "test");
            info!("Added {} movie submissions.", num_added);
    
            response = format!("You've submitted the movie: {}", movie_submission);
            info!("{}:{} submitted movie {}", msg.author, msg.author.name, movie_submission);

            msg.channel_id.say(&ctx.http, response).await?;
        } else {
            // Used to convert character to reaction
            use std::convert::TryFrom;

            let mut reactions: Vec<ReactionType> = Vec::new();
            reactions.push(ReactionType::try_from("✅").unwrap());
            reactions.push(ReactionType::try_from("❎").unwrap());

            response = format!("You've already submitted the movie: {}, would you like to update your submission?", movie_subs[0].title);          

            let mut update_sub_msg = msg.channel_id.send_message(&ctx.http, |m| {
                m.content(response);
                m.reactions(reactions);

                m
            }).await.unwrap();

            if let Some(reaction) = &update_sub_msg.await_reaction(&ctx).timeout(std::time::Duration::from_secs(10)).author_id(msg.author.id).await {
                let emoji = &reaction.as_inner_ref().emoji;

                let _ = match emoji.as_data().as_str() {
                    "✅" => {
                        submissions::delete_moviesub(&db_pool.get().unwrap(), movie_subs[0].id);
                        submissions::create_moviesub(&db_pool.get().unwrap(), &msg.author.id.to_string(), movie_submission, "test");
                        update_sub_msg.edit(ctx, |m| m.content("Submission updated!")).await?;
                        update_sub_msg.delete_reactions(ctx).await?;
                        Ok(update_sub_msg)
                    },
                    "❎" => { 
                        update_sub_msg.edit(ctx, |m| m.content("Submission not updated.")).await?;
                        update_sub_msg.delete_reactions(ctx).await?;
                        Ok(update_sub_msg)
                    },
                    _ => msg.reply(ctx, "Please react with ✅ or ❎").await,
                };
            } else {
                msg.reply(ctx, "No reaction within 10 seconds.").await?;
            }
        }
    }

    Ok(())
}

#[command]
pub async fn getsubs(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let db_pool = {
        // While data is a RwLock, it's recommended that you always open the lock as read.
        // This is mainly done to avoid Deadlocks for having a possible writer waiting for multiple
        // readers to close.
        let data_read = ctx.data.read().await;
        // Since the CommandCounter Value is wrapped in an Arc, cloning will not duplicate the
        // data, instead the reference is cloned.
        // We wap every value on in an Arc, as to keep the data lock open for the least time possible,
        // to again, avoid deadlocking it.
        data_read.get::<DBConnectionContainer>().expect("Expected DBConnection in TypeMap.").clone()
    };
    
    // Query DB
    let movie_subs = submissions::get_moviesubs(&db_pool.get().unwrap());
    info!("Got {} movie submission(s).", movie_subs.len());

    // Struct to help with information gathered from the DB
    struct DisMovieSub {
        movie_sub: Submission,
        user: User,
        nick: String
    }

    let mut dis_movie_subs: Vec<DisMovieSub> = Vec::new();

    use std::convert::TryFrom;

    // TODO: Move to closure once async closures are a thing.
    for movie_sub in movie_subs.clone() {
        let user_id: u64 = movie_sub.dis_user_id.parse().unwrap();
        let user = UserId::try_from(user_id).unwrap().to_user(&ctx.http).await?;
        let nick = user.nick_in(&ctx.http, msg.guild_id.unwrap()).await.unwrap();
        dis_movie_subs.push(DisMovieSub {movie_sub, user, nick});
    }

    use serenity::model::id::UserId;

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|mut e| {
            e.title("Current Movie Submissions");
            for dis_movie_sub in dis_movie_subs {
                e.field(
                    format!("{}({})", dis_movie_sub.nick, dis_movie_sub.user.name),
                    dis_movie_sub.movie_sub.title,
                    false
                );
            }

            e
        });

        m
    }).await?;

    Ok(())
}

#[command]
pub async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DBConnectionContainer>().expect("Expected DBConnection in TypeMap.").clone()
    };

    let movie_subs = submissions::get_moviesubs(&db_pool.get().unwrap());
    info!("Got {} movie submission(s).", movie_subs.len());

    if movie_subs.len() < 2 {
        msg.reply(&ctx.http, "Not enough movies submitted to choose two!").await?;
    }

    // Randomization
    use rand::{Rng, SeedableRng};
    use rand::rngs::SmallRng;

    let mut rng = SmallRng::from_entropy();

    let choice_1: usize = rng.gen_range(0, movie_subs.len());

    let choice_2: usize = {
        let mut temp: usize = rng.gen_range(0, movie_subs.len());
        loop {
            if temp != choice_1 {
                break;
            }

            // Regenerate another choice.
            temp = rng.gen_range(0, movie_subs.len());
        }

        temp
    };

    let choice_movie1 = movie_subs[choice_1].clone();
    let choice_movie2 = movie_subs[choice_2].clone();

    // Lookup user by discord id
    use std::convert::TryFrom;
    let choice_user1 = UserId::try_from(choice_movie1.dis_user_id.parse::<u64>().unwrap()).unwrap().to_user(&ctx.http).await.unwrap();
    let choice_user2 = UserId::try_from(choice_movie2.dis_user_id.parse::<u64>().unwrap()).unwrap().to_user(&ctx.http).await.unwrap();

    // Get current nickname in current guild
    let choice_nick1 = choice_user1.nick_in(&ctx.http, msg.guild_id.unwrap()).await.unwrap();
    let choice_nick2 = choice_user2.nick_in(&ctx.http, msg.guild_id.unwrap()).await.unwrap();

    // Respond with movie selection message
    let msg_movie_selection = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|mut e| {

            e.title("Movie Selections!");
            e.description("Please react with the two emotes you would like to use for voting. Make sure to only use Emoji's from within this Guild/Server. You have 5 minutes.");
            e.field(
                "Choice 1",
                format!("{} submitted by {}", choice_movie1.title, choice_nick1),
                false
            );
            e.field(
                "Choice 2",
                format!("{} submitted by {}", choice_movie2.title, choice_nick2),
                false
            );
            
            e
        });

        m
    }).await?;

    // Collect reactions from previous message
    use serenity::futures::StreamExt;
    let reactions: Vec<_> = msg_movie_selection.await_reactions(&ctx).timeout(std::time::Duration::from_secs(60 * 5)).author_id(msg.author.id).collect_limit(2).await.collect().await;

    // Check that two reactions were supplied
    if reactions.len() < 2 {
        msg.reply(&ctx.http, "No reactions supplied, try rolling later.").await.unwrap();
    }

    let mut emoji_check = false;

    for reaction in reactions.clone() {
        use serenity::model::channel::ReactionType;
        emoji_check = match reaction.as_inner_ref().emoji.clone() {
            ReactionType::Custom {animated: _, id, name: _}=> {
                // TODO: Dumb way to do this?
                match msg.guild(&ctx.cache).await.unwrap().emoji(&ctx.http, id).await {
                    Ok(_) => true,
                    Err(_e) => false
                }
            },
            ReactionType::Unicode(_string) => true,
            _ => false
        };
    }

    if emoji_check == false {
        msg_movie_selection.delete(&ctx.http).await.unwrap();
        msg.reply(&ctx.http, "Cannot use emoji outside of Guild/Server!").await.unwrap();
    } else {
        msg_movie_selection.delete(&ctx.http).await.unwrap();

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|mut e| {
    
                e.title("Movie Voting!");
                e.field(
                    "Choice 1",
                    format!("{} submitted by {} use {}", choice_movie1.title, choice_nick1, reactions[0].clone().as_inner_ref().emoji),
                    false
                );
                e.field(
                    "Choice 2",
                    format!("{} submitted by {} use {}", choice_movie2.title, choice_nick2, reactions[1].clone().as_inner_ref().emoji),
                    false
                );
                
                e
            });
    
            m
        }).await?;
    }

    Ok(())
}