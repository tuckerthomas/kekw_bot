use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{
    ConnectionManager,
    Pool
};
use serenity::prelude::{
    TypeMapKey
};

pub mod moviesubs;

// Setup DB Connection data for Context
pub struct DBConnectionContainer;

impl TypeMapKey for DBConnectionContainer {
    type Value = Pool::<ConnectionManager::<SqliteConnection>>;
}
