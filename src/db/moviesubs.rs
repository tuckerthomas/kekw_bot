use diesel::prelude::*;

use crate::models::MovieSub;

pub fn get_moviesubs(conn: &SqliteConnection) -> Vec<MovieSub> {
    use crate::schema::moviesubs::dsl::*;

    let results = moviesubs
        .limit(5)
        .load::<MovieSub>(conn)
        .expect("Error loading submissions");

    return results;
}

use crate::models::NewMovieSub;
pub fn create_moviesub<'a>(conn: &SqliteConnection, dis_user_id: &'a str, title: &'a str, link: &'a str) -> usize {
    use crate::schema::moviesubs;

    let new_post = NewMovieSub {
        dis_user_id: dis_user_id,
        title: title,
        link: link,
    };

    diesel::insert_into(moviesubs::table)
        .values(&new_post)
        .execute(conn)
        .expect("Error saving new submission")
}

pub fn delete_moviesub<'a>(conn: &SqliteConnection, del_id: i32) -> usize {
    use crate::schema::moviesubs;
    use crate::schema::moviesubs::dsl::*;

    diesel::delete(moviesubs::table.filter(id.eq(del_id)))
        .execute(conn)
        .expect("Error deleting submission")
}

pub fn check_prev_sub<'a>(conn: &SqliteConnection, check_dis_user_id: &'a str) -> Vec<MovieSub> {
    use crate::schema::moviesubs::dsl::*;

    let results = moviesubs
        .filter(dis_user_id.eq(check_dis_user_id))
        .limit(5)
        .load::<MovieSub>(conn)
        .expect("Error loading submissions");

    return results;
}