use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};

use tracing::{info, error};

use crate::db::{
    DBConnectionContainer,
    moviesubs
};

use crate::models::MovieSub;

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

        let movie_subs = moviesubs::check_prev_sub(&db_pool.get().unwrap(), &msg.author.id.to_string());

        let mut response = String::new();

        if movie_subs.len() == 0 {
            let num_added = moviesubs::create_moviesub(&db_pool.get().unwrap(), &msg.author.id.to_string(), movie_submission, "test");
            info!("Added {} movie submissions.", num_added);
    
            response = format!("You've submitted the movie: {}", movie_submission);
            info!("{}:{} submitted movie {}", msg.author, msg.author.name, movie_submission);

            msg.channel_id.say(&ctx.http, response).await?;
        } else {
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
                        moviesubs::delete_moviesub(&db_pool.get().unwrap(), movie_subs[0].id);
                        moviesubs::create_moviesub(&db_pool.get().unwrap(), &msg.author.id.to_string(), movie_submission, "test");
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
    let movie_subs = moviesubs::get_moviesubs(&db_pool.get().unwrap());
    info!("Got {} movie submission(s).", movie_subs.len());

    // Struct to help with information gathered from the DB
    struct DisMovieSub {
        movie_sub: MovieSub,
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