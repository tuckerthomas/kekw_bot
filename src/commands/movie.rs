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

        let num_added = moviesubs::create_moviesub(&db_pool.get().unwrap(), 1, movie_submission, "test");
        info!("Added {} movie submissions.", num_added);

        let response = format!("You've submitted the movie: {}", movie_submission);
        info!("{}:{} submitted movie {}", msg.author, msg.author.name, movie_submission);

        msg.channel_id.say(&ctx.http, response).await?;
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
    
    let movie_subs = moviesubs::get_moviesubs(&db_pool.get().unwrap());
    info!("Got {} movie submission(s).", movie_subs.len());

    let mut response = String::from("Current movies submitted:\n");
    for movie_sub in movie_subs.clone() {
        response.push_str(&format!("{}: {}", movie_sub.title, movie_sub.dis_user_id));
    }
    
    msg.channel_id.say(&ctx.http, response).await?;

    Ok(())
}