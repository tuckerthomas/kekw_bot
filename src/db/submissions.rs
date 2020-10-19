use diesel::prelude::*;

use crate::models::submission::Submission;

pub fn get_moviesubs(conn: &SqliteConnection, cur_period_id: i32) -> Vec<Submission> {
    use crate::schema::submissions::dsl::*;

    let results = submissions
        .filter(period_id.eq(cur_period_id))
        .load::<Submission>(conn)
        .expect("Error loading submissions");

    return results;
}

use crate::models::submission::NewSubmission;
pub fn create_moviesub<'a>(conn: &SqliteConnection, dis_user_id: &'a str, title: &'a str, link: &'a str, period_id: i32) -> usize {
    use crate::schema::submissions;

    let new_post = NewSubmission {
        dis_user_id: dis_user_id,
        title: title,
        link: link,
        period_id: period_id
    };

    diesel::insert_into(submissions::table)
        .values(&new_post)
        .execute(conn)
        .expect("Error saving new submission")
}

pub fn delete_moviesub<'a>(conn: &SqliteConnection, del_id: i32) -> usize {
    use crate::schema::submissions;
    use crate::schema::submissions::dsl::*;

    diesel::delete(submissions::table.filter(id.eq(del_id)))
        .execute(conn)
        .expect("Error deleting submission")
}

pub fn check_prev_sub<'a>(conn: &SqliteConnection, check_dis_user_id: &'a str) -> Vec<Submission> {
    use crate::schema::submissions::dsl::*;

    let results = submissions
        .filter(dis_user_id.eq(check_dis_user_id))
        .limit(5)
        .load::<Submission>(conn)
        .expect("Error loading submissions");

    return results;
}