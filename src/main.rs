// Pull in local modules
mod commands;
mod omdb;
mod timed;
mod utils;

// Imports
use std::{collections::HashSet, env, sync::Arc};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::id::GuildId,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use kekw_db::KekPool;

// Serenity(Discord)
use commands::{math::*, movie::*};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Setup DB Connection data for Context
pub struct DBConnectionContainer;

impl TypeMapKey for DBConnectionContainer {
    type Value = KekPool;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Starting Scheduler thread.");

        tokio::spawn(async move {
            let ctx = Arc::new(ctx);
            let ctx1 = Arc::clone(&ctx);
            loop {
                use chrono::prelude::*;
                use chrono::{DateTime, Duration, Utc};
                use chrono_tz::US::Eastern;
                pub const WEEK_IN_SECONDS: i64 = 604800;
                pub fn get_next_movie_selection() -> Duration {
                    //get a hardcoded past reset date / time (17:00 UTC every tuesday)
                    let last_selection = Eastern.ymd(2021, 2, 11).and_hms(18, 0, 0);
                    let now: DateTime<Utc> = Utc::now();
                    //get total seconds between now and the past reset
                    //take the mod of that divided by a week in seconds
                    //subtract that amount from current date / time to find previous reset
                    Duration::seconds(
                        WEEK_IN_SECONDS
                            - ((now - last_selection.with_timezone(&Utc)).num_seconds()
                                % WEEK_IN_SECONDS),
                    )
                }

                let next_movie_selection_duration = get_next_movie_selection();

                info!(
                    "Next movie selection scheduled for {}",
                    Utc::now() + next_movie_selection_duration
                );

                let mut interval_timer =
                    tokio::time::interval(next_movie_selection_duration.to_std().unwrap());

                // Tick once to clear it??
                interval_timer.tick().await;

                // Wait for the next interval tick
                interval_timer.tick().await;
                timed::jobs::select_movie(&ctx1).await;
            }
        })
        .await
        .unwrap(); // For async task
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(multiply)]
struct General;

#[group]
#[prefix = "m"]
#[description = "Submit a movie to Movie Night!"]
#[default_command(submit)]
#[commands(
    deletesub,
    getsubs,
    roll,
    startperiod,
    reopenperiod,
    endperiod,
    listperiods,
    fixdb
)]
struct Movie;

#[tokio::main]
async fn main() {
    match env::var("PROD") {
        Ok(prod) => {
            info!("Running in production");
        }
        Err(e) => {
            info!("Running in dev");
            dotenv::dotenv().expect("Failed to load .env file");
        }
    }

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    // Start sqlite connection
    let pool = kekw_db::establish_connection();

    // Setup Discord bot variables
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN to be set");

    env::var("MOVIE_NOTIFY_ROLE_ID").expect("Expected MOVIE_NOTIFY_ROLE_ID to be set");
    env::var("MOVIE_CHANNEL").expect("Expected MOVIE_CHANNEL to be set");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("!"))
        .group(&GENERAL_GROUP)
        .group(&MOVIE_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());

        // Write connection to client data
        data.insert::<DBConnectionContainer>(pool);
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
