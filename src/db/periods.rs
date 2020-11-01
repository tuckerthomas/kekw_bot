use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

use crate::models::period::Period;

use chrono::{DateTime, Utc};

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
struct PeriodError;

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl std::fmt::Display for PeriodError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

pub fn get_periods(pool: &Pool<ConnectionManager<SqliteConnection>>) -> Result<Vec<Period>> {
    use crate::schema::periods::dsl::*;

    match periods
        .order(id.desc())
        .limit(10)
        .load::<Period>(&pool.get()?)
        {
            Ok(period) => return Ok(period),
            Err(e) => Err(Box::new(e)),
        }
}

pub fn get_most_recent_period(pool: &Pool<ConnectionManager<SqliteConnection>>) -> Result<Period> {
    use crate::schema::periods::dsl::*;

    match periods
        .order(id.desc())
        .filter(end_day.is_null())
        .first::<Period>(&pool.get()?)
    {
        Ok(period) => return Ok(period),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn get_most_recent_closed_period(
    pool: &Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Period> {
    use crate::schema::periods::dsl::*;

    match periods
        .order(id.desc())
        .filter(end_day.is_not_null())
        .first::<Period>(&pool.get()?)
    {
        Ok(period) => Ok(period),
        Err(e) => Err(Box::new(e)),
    }
}

use crate::models::period::NewPeriod;
pub fn create_period<'a>(pool: &Pool<ConnectionManager<SqliteConnection>>) -> Result<usize> {
    use crate::schema::periods;

    let now: DateTime<Utc> = Utc::now();

    let new_period = NewPeriod {
        start_day: now.timestamp(),
    };

    match diesel::insert_into(periods::table)
        .values(&new_period)
        .execute(&pool.get()?)
    {
        Ok(num_values) => Ok(num_values),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn reopen_period<'a>(
    pool: &Pool<ConnectionManager<SqliteConnection>>,
    mut period_to_end: Period,
) -> Result<usize> {
    use crate::schema::periods;

    period_to_end.end_day = None;

    match diesel::update(periods::table)
        .set(&period_to_end)
        .execute(&pool.get()?)
    {
        Ok(num_values) => Ok(num_values),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn end_period<'a>(
    pool: &Pool<ConnectionManager<SqliteConnection>>,
    mut period_to_end: Period,
) -> Result<usize> {
    use crate::schema::periods;

    period_to_end.end_day = Some(Utc::now().timestamp());

    match diesel::update(periods::table)
        .set(&period_to_end)
        .execute(&pool.get()?)
    {
        Ok(num_values) => Ok(num_values),
        Err(e) => Err(Box::new(e)),
    }
}
