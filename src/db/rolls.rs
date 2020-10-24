use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::models::roll::{Roll, NewRoll};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

pub fn get_rolls(conn: &SqliteConnection) -> Vec<Roll> {
    use crate::schema::rolls::dsl::*;

    let results = rolls
        .order(id.desc())
        .limit(5)
        .load::<Roll>(conn)
        .expect("Error loading submissions");

    return results;
}


pub fn get_roll_by_period_id(conn: &SqliteConnection, search_period_id: i32) -> Result<Roll> {
    use crate::schema::rolls::dsl::*;

    match rolls
        .filter(period_id.eq(search_period_id))
        .first::<Roll>(conn) {
            Ok(roll) => Ok(roll),
            Err(e) => Err(Box::new(e)),
        }
}

pub fn create_roll(pool: &Pool<ConnectionManager<SqliteConnection>>, new_period_id: i32, new_selection_1: i32, new_selection_2: i32) -> Result<usize> {
    use crate::schema::rolls;

    let new_roll = NewRoll {
        period_id: new_period_id, selection_1: new_selection_1, selection_2: new_selection_2
    };

    match diesel::insert_into(rolls::table)
        .values(&new_roll)
        .execute(&pool.get()?)
    {
        Ok(num_values) => Ok(num_values),
        Err(e) => Err(Box::new(e)),
    }
}