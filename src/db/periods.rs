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

pub fn get_most_recent_period(conn: &SqliteConnection) -> Period {
    use crate::schema::periods::dsl::*;

    let results = periods
        .order(id.desc())
        .limit(1)
        .load::<Period>(conn)
        .expect("Error loading submissions");

    return results[0];
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