// Diesel needs this attrubte for macro generation
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

// Pull in local modules
mod commands;
mod db;
mod models;
mod schema;

// Imports
use std::{
    collections::HashSet,
    env,
    sync::Arc,
};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use tracing::{error, info};
use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};


// Diesel
use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{
    ConnectionManager,
    Pool
};

// Use embeded migrations
diesel_migrations::embed_migrations!("./migrations");

// Serenity(Discord)
use commands::{
    math::*,
    movie::*
};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(multiply)]
struct General;

#[group]
#[prefix = "q"]
#[description = "Submit a movie to Movie Night!"]
#[default_command(submit)]
#[commands(getsubs, roll, startperiod, reopenperiod)]
struct Movie;

#[tokio::main]
async fn main() {
    match env::var("PROD") {
        Ok(prod) => {
            info!("Running in production");
        }
        Err(e) => {
            info!("Running in dev");
            dotenv::dotenv().expect("Failted to load .env file");
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
    let pool = establish_connection();
    let new_pool = pool.clone();
    // By default the output is thrown out. If you want to redirect it to stdout, you
    // should call embedded_migrations::run_with_output.
    embedded_migrations::run_with_output(&new_pool.get().unwrap(), &mut std::io::stdout()).unwrap(); 

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN to be set");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c
                   .owners(owners)
                   .prefix("!"))
        .group(&GENERAL_GROUP)
        .group(&MOVIE_GROUP);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());

        use self::db::DBConnectionContainer;
        // Write connection to client data
        data.insert::<DBConnectionContainer>(pool);
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

pub fn establish_connection() -> Pool::<ConnectionManager::<SqliteConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set!");

    Pool::builder().build(ConnectionManager::<SqliteConnection>::new(database_url)).expect("Could not creat pool")
}