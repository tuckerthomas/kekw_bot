use diesel::prelude::*;

use crate::models::period::Period;

use chrono::{DateTime, Utc};

pub fn get_period(conn: &SqliteConnection) -> Vec<Period> {
    use crate::schema::periods::dsl::*;

    let results = periods
        .order(id.desc())
        .limit(5)
        .load::<Period>(conn)
        .expect("Error loading submissions");

    return results;
}

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


pub fn get_most_recent_period(conn: &SqliteConnection) -> Vec<Period> {
    use crate::schema::periods::dsl::*;

    let results = periods
        .order(id.desc())
        .filter(end_day.is_null())
        .limit(1)
        .load::<Period>(conn)
        .expect("Error loading periods");

    return results;
}

use crate::models::period::NewPeriod;

pub fn create_period<'a>(conn: &SqliteConnection) -> usize {
    use crate::schema::periods;

    let now: DateTime<Utc> = Utc::now();

    let new_post = NewPeriod {
        start_day: now.timestamp()
    };

    diesel::insert_into(periods::table)
        .values(&new_post)
        .execute(conn)
        .expect("Error saving new submission")
}

pub fn end_period<'a>(conn: &SqliteConnection, mut period_to_end: Period) -> usize {
    use crate::schema::periods;

    period_to_end.end_day = Some(Utc::now().timestamp());

    diesel::update(periods::table)
        .set(&period_to_end)
        .execute(conn)
        .expect("Error saving new submission")
}