use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use tracing::{error, info};

use crate::db::{periods, rolls, submissions, DBConnectionContainer};

use crate::models::submission::Submission;

#[command]
pub async fn submit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        if let Err(why) = msg.channel_id.say(&ctx.http, "No movie supplied.").await {
            error!("Error sending message: {:?}", why);
        }
    } else {
        let movie_submission = args.rest();

        // Pull DBConnection from local context
        let db_pool = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<DBConnectionContainer>()
                .expect("Expected DBConnection in TypeMap.")
                .clone()
        };

        match periods::get_most_recent_period(&db_pool) {
            Ok(cur_period) => {
                let movie_subs = submissions::check_prev_sub(
                    &db_pool.get().unwrap(),
                    cur_period.id,
                    &msg.author.id.to_string(),
                );

                let mut response = String::new();
                if movie_subs.len() == 0 {
                    let num_added = submissions::create_moviesub(
                        &db_pool.get().unwrap(),
                        &msg.author.id.to_string(),
                        movie_submission,
                        "test",
                        cur_period.id,
                    );
                    info!("Added {} movie submissions.", num_added);
                    response = format!("You've submitted the movie: {}", movie_submission);
                    info!(
                        "{}:{} submitted movie {}",
                        msg.author, msg.author.name, movie_submission
                    );
                    msg.channel_id.say(&ctx.http, response).await?;
                } else {
                    let conf_message = format!("You've already submitted the movie: {}, would you like to update your submission?", movie_subs[0].title);
                    let yes_msg = String::from("Submission updated!");
                    let no_msg = String::from("Submission not updated.");

                    use crate::utils::Confirmation;

                    match crate::utils::ask_confirmation(
                        &ctx,
                        msg.author.id,
                        msg.channel_id,
                        conf_message,
                        yes_msg,
                        no_msg,
                    )
                    .await
                    {
                        Ok(Confirmation::Yes) => {
                            // TODO: Make update_moviesub
                            let mut updated_moviesub = movie_subs[0].clone();
                            updated_moviesub.title = String::from(movie_submission);
                            submissions::update_moviesub(&db_pool, updated_moviesub)?;
                        }
                        Ok(Confirmation::No) => (),
                        Ok(Confirmation::InvalidConfirmation) => {
                            msg.reply(ctx, "Error").await?;
                        }
                        _ => {
                            msg.reply(ctx, "Error").await?;
                        }
                    }
                }
            }
            Err(NotFound) => {
                msg.channel_id
                    .say(&ctx.http, "No current movie submission periods active.")
                    .await
                    .unwrap();
            }
            Err(e) => {
                error!("Failed to get movie submission");
            }
        }
    }

    Ok(())
}

#[command]
pub async fn getsubs(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    match periods::get_most_recent_period(&db_pool) {
        Ok(cur_period) => {
            // Query DB
            let movie_subs = submissions::get_moviesubs(&db_pool.get().unwrap(), cur_period.id);
            info!("Got {} movie submission(s).", movie_subs.len());

            // Struct to help with information gathered from the DB
            struct DisMovieSub {
                movie_sub: Submission,
                user: User,
                nick: String,
            }

            let mut dis_movie_subs: Vec<DisMovieSub> = Vec::new();

            use std::convert::TryFrom;

            // TODO: Move to closure once async closures are a thing.
            for movie_sub in movie_subs.clone() {
                let user_id: u64 = movie_sub.dis_user_id.parse().unwrap();
                let user = UserId::try_from(user_id)
                    .unwrap()
                    .to_user(&ctx.http)
                    .await?;
                let nick = match user.nick_in(&ctx.http, msg.guild_id.unwrap()).await {
                    Some(nick) => nick,
                    None => String::from(""),
                };
                dis_movie_subs.push(DisMovieSub {
                    movie_sub,
                    user,
                    nick,
                });
            }

            use serenity::model::id::UserId;

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Current Movie Submissions");
                        for dis_movie_sub in dis_movie_subs {
                            if dis_movie_sub.nick.is_empty() {
                                e.field(
                                    format!("{}", dis_movie_sub.user.name),
                                    dis_movie_sub.movie_sub.title,
                                    false,
                                );
                            } else {
                                e.field(
                                    format!("{}({})", dis_movie_sub.nick, dis_movie_sub.user.name),
                                    dis_movie_sub.movie_sub.title,
                                    false,
                                );
                            }
                        }
                        e
                    });
                    m
                })
                .await?;
        }
        Err(NotFound) => {
            msg.channel_id
                .say(&ctx.http, "No current movie submission periods active.")
                .await
                .unwrap();
        }
        Err(e) => {
            error!("Failed to get movie submission");
        }
    }

    Ok(())
}

#[command]
pub async fn deletesub(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    if msg.mentions.len() == 0 {
        msg.reply(
            &ctx.http,
            "Please provide the user(s) you would like to delete the submissions for!",
        )
        .await?;
        return Ok(());
    }

    match periods::get_most_recent_period(&db_pool) {
        Err(_) => {
            msg.reply(&ctx.http, "No current movie submission period exists.")
                .await?;
        }
        Ok(cur_period) => {
            for user in &msg.mentions {
                match submissions::get_submission_by_period_and_user(
                    &db_pool,
                    cur_period,
                    user.id.to_string(),
                ) {
                    Ok(sub) => {
                        submissions::delete_moviesub(&db_pool, &sub);
                        msg.reply(
                            &ctx.http,
                            &format!("Deleted submission {} for {}.", sub.title, user.name),
                        )
                        .await?;
                    }
                    Err(_) => {
                        msg.reply(
                            &ctx.http,
                            &format!("Submission does not exist for {}.", user.name),
                        )
                        .await?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[command]
pub async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    // Get most recent period
    match periods::get_most_recent_period(&db_pool) {
        Ok(cur_period) => {
            // Get movie submissions for the current period
            let movie_subs = submissions::get_moviesubs(&db_pool.get().unwrap(), cur_period.id);
            info!("Got {} movie submission(s).", movie_subs.len());

            if movie_subs.len() >= 2 {
                // Confirm roll
                let conf_message = String::from("Would you like to roll for movie night?");
                let yes_msg = String::from("Starting roll");
                let no_msg = String::from("Cancelling roll");

                use crate::utils::Confirmation;

                let start_roll = match crate::utils::ask_confirmation(
                    &ctx,
                    msg.author.id,
                    msg.channel_id,
                    conf_message,
                    yes_msg,
                    no_msg,
                )
                .await
                {
                    Ok(Confirmation::Yes) => true,
                    Ok(Confirmation::No) => false,
                    _ => false,
                };

                // User decided to start a roll
                if start_roll {
                    // End current submission period
                    periods::end_period(&db_pool, cur_period)?;

                    // Check if a roll already exists
                    if let Ok(cur_roll) = rolls::get_roll_by_period(&db_pool, cur_period) {
                        let conf_message = format!("There already exists a roll for this movie submission period, would you like to roll again?");
                        let yes_msg = String::from("Rolling again!");
                        let no_msg = String::from("Cancelling roll.");
                        use crate::utils::Confirmation;
                        match crate::utils::ask_confirmation(
                            &ctx,
                            msg.author.id,
                            msg.channel_id,
                            conf_message,
                            yes_msg,
                            no_msg,
                        )
                        .await
                        {
                            Ok(Confirmation::Yes) => {
                                rolls::delete_roll(&db_pool.clone(), cur_roll.id)?;
                            }
                            Ok(Confirmation::No) => {
                                return Ok(());
                            }
                            _ => {
                                return Ok(());
                            }
                        };
                    }

                    // Randomization
                    use rand::rngs::SmallRng;
                    use rand::{Rng, SeedableRng};

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

                    // Insert roll into roll table
                    rolls::create_roll(
                        &db_pool,
                        cur_period.id,
                        choice_movie1.id,
                        choice_movie2.id,
                    )?;

                    use std::convert::TryFrom;
                    // Lookup user by discord id
                    let choice_user1 =
                        UserId::try_from(choice_movie1.dis_user_id.parse::<u64>().unwrap())
                            .unwrap()
                            .to_user(&ctx.http)
                            .await
                            .unwrap();
                    let choice_user2 =
                        UserId::try_from(choice_movie2.dis_user_id.parse::<u64>().unwrap())
                            .unwrap()
                            .to_user(&ctx.http)
                            .await
                            .unwrap();

                    // Get current nickname in current guild
                    let choice_nick1 = choice_user1.name;
                    let choice_nick2 = choice_user2.name;

                    // Respond with movie selection message
                    let msg_movie_selection = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Movie Emoji Selections!");
                        e.description("Please react with the two emotes you would like to use for voting. Make sure to only use Emoji's from within this Guild/Server. You have 5 minutes.");
                        e.field(
                            &choice_movie1.title,
                            format!("submitted by {}", choice_nick1),
                            false
                        );
                        e.field(
                            &choice_movie2.title,
                            format!("submitted by {}", choice_nick2),
                            false
                        );

                        e
                    });

                    m
                }).await?;

                    // Collect reactions from previous message
                    use serenity::futures::StreamExt;
                    let reactions: Vec<_> = msg_movie_selection
                        .await_reactions(&ctx)
                        .timeout(std::time::Duration::from_secs(60 * 5))
                        .author_id(msg.author.id)
                        .collect_limit(2)
                        .await
                        .collect()
                        .await;

                    // Check that two reactions were supplied
                    if reactions.len() == 2 {
                        let mut emoji_check = false;

                        for reaction in reactions.clone() {
                            use serenity::model::channel::ReactionType;
                            emoji_check = match reaction.as_inner_ref().emoji.clone() {
                                ReactionType::Custom {
                                    animated: _,
                                    id,
                                    name: _,
                                } => {
                                    // TODO: Dumb way to do this?
                                    match msg
                                        .guild(&ctx.cache)
                                        .await
                                        .unwrap()
                                        .emoji(&ctx.http, id)
                                        .await
                                    {
                                        Ok(_) => true,
                                        Err(_) => false,
                                    }
                                }
                                ReactionType::Unicode(_) => true,
                                _ => false,
                            };
                        }

                        if emoji_check == false {
                            msg_movie_selection.delete(&ctx.http).await.unwrap();
                            msg.reply(&ctx.http, "Cannot use emoji outside of Guild/Server!")
                                .await
                                .unwrap();
                        } else {
                            msg_movie_selection.delete(&ctx.http).await.unwrap();

                            msg.channel_id
                                .send_message(&ctx.http, |m| {
                                    m.embed(|e| {
                                        e.title("Movie Voting!");
                                        e.field(
                                            choice_movie1.title,
                                            format!(
                                                "submitted by {} use {}",
                                                choice_nick1,
                                                reactions[0].clone().as_inner_ref().emoji
                                            ),
                                            false,
                                        );
                                        e.field(
                                            choice_movie2.title,
                                            format!(
                                                "submitted by {} use {}",
                                                choice_nick2,
                                                reactions[1].clone().as_inner_ref().emoji
                                            ),
                                            false,
                                        );

                                        e
                                    });

                                    m
                                })
                                .await?;
                        }
                    } else {
                        msg.reply(&ctx.http, "No reactions supplied, try rolling later.")
                            .await
                            .unwrap();
                    }
                } // End check start roll
            } else {
                msg.reply(&ctx.http, "Not enough movies submitted to choose two!")
                    .await?;
            }
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, "No current movie submission periods active.")
                .await
                .unwrap();
        }
    }

    Ok(())
}

#[command]
pub async fn startperiod(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    match periods::get_most_recent_period(&db_pool) {
        Ok(cur_period) => {
            msg.channel_id.say(&ctx.http, "A submission period has already started, run `!m roll` to finish the current submission period.").await.unwrap();
        }
        Err(NotFound) => {
            periods::create_period(&db_pool)?;
            msg.channel_id
                .say(&ctx.http, "Started new submission period!")
                .await
                .unwrap();
        }
    }

    Ok(())
}

#[command]
pub async fn reopenperiod(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    match periods::get_most_recent_closed_period(&db_pool) {
        Ok(cur_period) => {
            periods::reopen_period(&db_pool, cur_period)?;
            msg.channel_id
                .say(&ctx.http, "Reopened last submission period!")
                .await
                .unwrap();
        }
        Err(e) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Could not find a recently closed submission period.",
                )
                .await
                .unwrap();
        }
    }
    Ok(())
}

#[command]
pub async fn endperiod(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    match periods::get_most_recent_period(&db_pool) {
        Ok(cur_period) => {
            periods::end_period(&db_pool, cur_period)?;
            msg.channel_id
                .say(&ctx.http, "Ended current movie submission without roll!")
                .await
                .unwrap();
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, "No current movie submission period exists.")
                .await
                .unwrap();
        }
    }
    Ok(())
}

#[command]
pub async fn listperiods(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    match periods::get_periods(&db_pool) {
        Ok(movie_periods) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        for movie_period in movie_periods {
                            e.field("Start Date", movie_period.start_day, true);
                            if movie_period.end_day.is_some() {
                                e.field("End Date", movie_period.end_day.unwrap(), true);
                            }

                            if let Ok(movie_roll) =
                                rolls::get_roll_by_period(&db_pool, movie_period)
                            {
                                let movie_roll_1 = submissions::get_submission_by_id(
                                    &db_pool,
                                    movie_roll.selection_1,
                                )
                                .unwrap();
                                let movie_roll_2 = submissions::get_submission_by_id(
                                    &db_pool,
                                    movie_roll.selection_2,
                                )
                                .unwrap();
                                e.field("Choice 1", movie_roll_1.title, false);
                                e.field("Choice 2", movie_roll_2.title, false);
                            } else {
                                e.field("Roll", "No Roll!", false);
                            }
                        }
                        e
                    });

                    m
                })
                .await
                .unwrap();
        }
        Err(e) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Could not find a recently closed submission period.",
                )
                .await
                .unwrap();
        }
    }
    Ok(())
}

#[command]
pub async fn fixdb(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db_pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DBConnectionContainer>()
            .expect("Expected DBConnection in TypeMap.")
            .clone()
    };

    let movie_subs = submissions::get_all_moviesubs(&db_pool);

    for movie_sub in movie_subs {
        let movie_title = movie_sub.title.clone();
        let keys = movie_title.split(" ");

        for key in keys {
            if key.contains("imdb") && key.contains("http") {
                if let Ok(imdb_url) = reqwest::Url::parse(key) {
                    let path_segments = imdb_url.path_segments().ok_or_else(|| "cannot be base")?;

                    for path_segment in path_segments {
                        if path_segment.starts_with("tt") {
                            use crate::omdb;

                            println!("Found imdb link {}", path_segment);

                            let movie = omdb::query_by_id(String::from(path_segment))
                                .await
                                .unwrap()
                                .unwrap();

                            let mut updated_moviesub = movie_sub.clone();

                            println!("Updating movie submission id: {}", updated_moviesub.id);

                            updated_moviesub.title = movie.title;
                            updated_moviesub.link = movie.imdb_id;

                            submissions::update_moviesub(&db_pool, updated_moviesub)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
